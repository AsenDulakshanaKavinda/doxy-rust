use crate::db::models::ChatDocument;
use crate::db::pool::DbPool;
use crate::db::schema::chat_document;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use diesel::prelude::Insertable;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde::Deserialize;
use tokio::task;
use uuid::Uuid;

/// Shape of the incoming JSON body when attaching a document to a chat.
/// `position` is optional — if omitted, the document is appended to the
/// end of the chat's current document list.
#[derive(Deserialize)]
pub struct AddDocumentToChatRequest {
    pub document_id: String,
    pub position: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = chat_document)]
pub struct NewChatDocument {
    pub chat_id: String,
    pub document_id: String,
    pub position: i32,
}

fn validate_ids(chat_id: &str, document_id: &str) -> Result<(), (StatusCode, String)> {
    if Uuid::parse_str(chat_id).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            "chat_id is not a valid uuid".to_string(),
        ));
    }
    if Uuid::parse_str(document_id).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            "document_id is not a valid uuid".to_string(),
        ));
    }
    Ok(())
}

/// Attach a document to a chat at a given (or auto-computed) position.
pub async fn add_document_to_chat(
    State(pool): State<DbPool>,
    Path(chat_id): Path<String>,
    Json(payload): Json<AddDocumentToChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    validate_ids(&chat_id, &payload.document_id)?;

    let document_id = payload.document_id;
    let requested_position = payload.position;

    let entry = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        // If no explicit position was given, append after the current
        // highest position for this chat.
        let position = match requested_position {
            Some(p) => p,
            None => {
                let max_position: Option<i32> = chat_document::table
                    .filter(chat_document::chat_id.eq(&chat_id))
                    .select(diesel::dsl::max(chat_document::position))
                    .first(&mut conn)
                    .map_err(|e| {
                        tracing::error!("failed to compute next position: {e}");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "internal error".to_string(),
                        )
                    })?;
                max_position.map_or(0, |p| p + 1)
            }
        };

        let new_entry = NewChatDocument {
            chat_id,
            document_id,
            position,
        };

        diesel::insert_into(chat_document::table)
            .values(&new_entry)
            .get_result::<ChatDocument>(&mut conn)
            .map_err(|e| match e {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                    tracing::warn!("document already attached to chat: {}", info.message());
                    (
                        StatusCode::CONFLICT,
                        "this document is already attached to the chat".to_string(),
                    )
                }
                DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, info) => {
                    tracing::warn!("fk violation attaching document to chat: {}", info.message());
                    (
                        StatusCode::BAD_REQUEST,
                        "chat_id or document_id does not exist".to_string(),
                    )
                }
                other => {
                    tracing::error!("failed to attach document to chat: {other}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    )
                }
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("add_document_to_chat task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok((StatusCode::CREATED, Json(entry)))
}

/// Fetch a single chat/document link.
pub async fn get_chat_document(
    State(pool): State<DbPool>,
    Path((chat_id, document_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    validate_ids(&chat_id, &document_id)?;

    let entry = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        chat_document::table
            .filter(chat_document::chat_id.eq(&chat_id))
            .filter(chat_document::document_id.eq(&document_id))
            .first::<ChatDocument>(&mut conn)
            .map_err(|e| match e {
                DieselError::NotFound => (
                    StatusCode::NOT_FOUND,
                    "document is not attached to this chat".to_string(),
                ),
                other => {
                    tracing::error!("failed to fetch chat_document: {other}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    )
                }
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("get_chat_document task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok((StatusCode::OK, Json(entry)))
}

/// List all documents attached to a chat, ordered by position.
pub async fn list_documents_for_chat(
    State(pool): State<DbPool>,
    Path(chat_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if Uuid::parse_str(&chat_id).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            "chat_id is not a valid uuid".to_string(),
        ));
    }

    let entries = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        chat_document::table
            .filter(chat_document::chat_id.eq(&chat_id))
            .order(chat_document::position.asc())
            .load::<ChatDocument>(&mut conn)
            .map_err(|e| {
                tracing::error!("failed to list documents for chat: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("list_documents_for_chat task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok((StatusCode::OK, Json(entries)))
}

#[derive(Deserialize)]
pub struct UpdateChatDocumentPositionRequest {
    pub position: i32,
}

#[derive(AsChangeset)]
#[diesel(table_name = chat_document)]
pub struct UpdateChatDocumentPositionPayload {
    pub position: i32,
}

/// Update the position of a document within a chat's document list.
pub async fn update_chat_document_position(
    State(pool): State<DbPool>,
    Path((chat_id, document_id)): Path<(String, String)>,
    Json(payload): Json<UpdateChatDocumentPositionRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    validate_ids(&chat_id, &document_id)?;

    if payload.position < 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "position must not be negative".to_string(),
        ));
    }

    let changes = UpdateChatDocumentPositionPayload {
        position: payload.position,
    };

    let entry = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        diesel::update(
            chat_document::table
                .filter(chat_document::chat_id.eq(&chat_id))
                .filter(chat_document::document_id.eq(&document_id)),
        )
        .set(&changes)
        .get_result::<ChatDocument>(&mut conn)
        .map_err(|e| match e {
            DieselError::NotFound => (
                StatusCode::NOT_FOUND,
                "document is not attached to this chat".to_string(),
            ),
            other => {
                tracing::error!("failed to update chat_document position: {other}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            }
        })
    })
    .await
    .map_err(|e| {
        tracing::error!("update_chat_document_position task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok((StatusCode::OK, Json(entry)))
}

/// Detach a document from a chat.
pub async fn remove_document_from_chat(
    State(pool): State<DbPool>,
    Path((chat_id, document_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    validate_ids(&chat_id, &document_id)?;

    task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        let affected = diesel::delete(
            chat_document::table
                .filter(chat_document::chat_id.eq(&chat_id))
                .filter(chat_document::document_id.eq(&document_id)),
        )
        .execute(&mut conn)
        .map_err(|e| {
            tracing::error!("failed to detach document from chat: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        if affected == 0 {
            return Err((
                StatusCode::NOT_FOUND,
                "document is not attached to this chat".to_string(),
            ));
        }

        Ok(())
    })
    .await
    .map_err(|e| {
        tracing::error!("remove_document_from_chat task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok(StatusCode::NO_CONTENT)
}