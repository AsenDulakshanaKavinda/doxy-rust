use crate::db::models::User;
use crate::db::pool::DbPool;
use crate::db::schema::user;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use diesel::prelude::Insertable;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde::Deserialize;
use tokio::task;
use uuid::Uuid;

/// Shape of the incoming JSON body. Kept separate from `CreateUserPayload`
/// so the API surface (what a client may send) doesn't drift together with
/// the DB row shape (what actually gets inserted).
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Insertable)]
#[diesel(table_name = user)]
pub struct CreateUserPayload {
    pub name: String,
    pub email: String,

    // optional
    pub id: Option<String>,
    pub email_verified: Option<bool>,
    pub image: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub role: Option<String>,
    pub banned: Option<bool>,
    pub ban_reason: Option<String>,
    pub ban_expires: Option<DateTime<Utc>>,
}

impl CreateUserPayload {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id: Some(Uuid::new_v4().to_string()),
            name: name.into(),
            email: email.into(),
            email_verified: Some(false),
            image: None,
            created_at: Some(Utc::now()),
            updated_at: None,
            role: Some(String::from("user")),
            banned: None,
            ban_reason: None,
            ban_expires: None,
        }
    }
}

pub async fn get_user_by_id(
    State(pool): State<DbPool>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if Uuid::parse_str(&user_id).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            "id is not a valid uuid".to_string(),
        ));
    }

    let user = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        user::table
            .filter(user::id.eq(&user_id))
            .first::<User>(&mut conn)
            .map_err(|e| match e {
                DieselError::NotFound => (StatusCode::NOT_FOUND, "user not found".to_string()),
                other => {
                    tracing::error!("failed to fetch user by id: {other}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    )
                }
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("get_user_by_id task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok((StatusCode::OK, Json(user)))
}

pub async fn create_user(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Basic validation before touching the DB.
    let name = payload.name.trim();
    let email = payload.email.trim();

    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name must not be empty".into()));
    }
    if email.is_empty() || !email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "email is not valid".into()));
    }

    let new_user = CreateUserPayload::new(name, email);

    // Spawn blocking task to prevent stalling async execution.
    let user = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        diesel::insert_into(user::table)
            .values(&new_user)
            .get_result::<User>(&mut conn)
            .map_err(|e| match e {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                    tracing::warn!("duplicate email on create_user: {}", info.message());
                    (
                        StatusCode::CONFLICT,
                        "a user with that email already exists".to_string(),
                    )
                }
                other => {
                    tracing::error!("failed to insert user: {other}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    )
                }
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("create_user task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    // Return created User as JSON with 201 Created status.
    Ok((StatusCode::CREATED, Json(user)))
}

/// Shape of the incoming JSON body for updates. All fields optional so
/// callers can send a partial patch (e.g. just `email`).
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    // pub email: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = user)]
pub struct UpdateUserPayload {
    pub name: Option<String>,
    // pub email: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub async fn update_user_by_id(
    State(pool): State<DbPool>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if Uuid::parse_str(&user_id).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            "id is not a valid uuid".to_string(),
        ));
    }

    let name = payload.name.map(|n| n.trim().to_string());
    if let Some(ref n) = name {
        if n.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "name must not be empty".into()));
        }
    }

    if name.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            "at least one field (name or email) must be provided".to_string(),
        ));
    }

    let changes = UpdateUserPayload {
        name,
        updated_at: Some(Utc::now()),
    };

    let user = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        diesel::update(user::table.filter(user::id.eq(&user_id)))
            .set(&changes)
            .get_result::<User>(&mut conn)
            .map_err(|e| match e {
                DieselError::NotFound => (StatusCode::NOT_FOUND, "user not found".to_string()),
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                    tracing::warn!("duplicate email on update_user_by_id: {}", info.message());
                    (
                        StatusCode::CONFLICT,
                        "a user with that email already exists".to_string(),
                    )
                }
                other => {
                    tracing::error!("failed to update user: {other}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    )
                }
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("update_user_by_id task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok((StatusCode::OK, Json(user)))
}

pub async fn delete_user_by_id(
    State(pool): State<DbPool>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if Uuid::parse_str(&user_id).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            "id is not a valid uuid".to_string(),
        ));
    }

    task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        let affected = diesel::delete(user::table.filter(user::id.eq(&user_id)))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("failed to delete user: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            })?;

        if affected == 0 {
            return Err((StatusCode::NOT_FOUND, "user not found".to_string()));
        }

        Ok(())
    })
    .await
    .map_err(|e| {
        tracing::error!("delete_user_by_id task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok(StatusCode::NO_CONTENT)
}
