use crate::db::models::Organization;
use crate::db::pool::DbPool;
use crate::db::schema::organization;
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

/// Shape of the incoming JSON body for creation. Kept separate from
/// `CreateOrganizationPayload` so the API surface doesn't drift together
/// with the DB row shape.
#[derive(Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = organization)]
pub struct CreateOrganizationPayload {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<String>,
}

impl CreateOrganizationPayload {
    pub fn new(
        name: impl Into<String>,
        slug: impl Into<String>,
        logo: Option<String>,
        metadata: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            slug: slug.into(),
            logo,
            created_at: Utc::now(),
            metadata,
        }
    }
}

/// Basic slug validation: lowercase letters, digits, and hyphens only,
/// no leading/trailing/double hyphens. Adjust to match whatever your
/// frontend actually generates.
fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub async fn create_organization(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateOrganizationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let name = payload.name.trim().to_string();
    let slug = payload.slug.trim().to_lowercase();

    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name must not be empty".into()));
    }
    if !is_valid_slug(&slug) {
        return Err((
            StatusCode::BAD_REQUEST,
            "slug must be lowercase alphanumeric with single hyphens".into(),
        ));
    }

    let new_org = CreateOrganizationPayload::new(name, slug, payload.logo, payload.metadata);

    let org = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        diesel::insert_into(organization::table)
            .values(&new_org)
            .get_result::<Organization>(&mut conn)
            .map_err(|e| match e {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                    tracing::warn!("duplicate slug on create_organization: {}", info.message());
                    (
                        StatusCode::CONFLICT,
                        "an organization with that slug already exists".to_string(),
                    )
                }
                other => {
                    tracing::error!("failed to insert organization: {other}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    )
                }
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("create_organization task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok((StatusCode::CREATED, Json(org)))
}

pub async fn get_organization_by_id(
    State(pool): State<DbPool>,
    Path(org_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if Uuid::parse_str(&org_id).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            "id is not a valid uuid".to_string(),
        ));
    }

    let org = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        organization::table
            .filter(organization::id.eq(&org_id))
            .first::<Organization>(&mut conn)
            .map_err(|e| match e {
                DieselError::NotFound => {
                    (StatusCode::NOT_FOUND, "organization not found".to_string())
                }
                other => {
                    tracing::error!("failed to fetch organization by id: {other}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    )
                }
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("get_organization_by_id task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok((StatusCode::OK, Json(org)))
}

/// Shape of the incoming JSON body for updates. All fields optional so
/// callers can send a partial patch.
#[derive(Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub logo: Option<String>,
    pub metadata: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = organization)]
pub struct UpdateOrganizationPayload {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub logo: Option<String>,
    pub metadata: Option<String>,
}

pub async fn update_organization_by_id(
    State(pool): State<DbPool>,
    Path(org_id): Path<String>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if Uuid::parse_str(&org_id).is_err() {
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

    let slug = payload.slug.map(|s| s.trim().to_lowercase());
    if let Some(ref s) = slug {
        if !is_valid_slug(s) {
            return Err((
                StatusCode::BAD_REQUEST,
                "slug must be lowercase alphanumeric with single hyphens".into(),
            ));
        }
    }

    if name.is_none() && slug.is_none() && payload.logo.is_none() && payload.metadata.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            "at least one field must be provided".to_string(),
        ));
    }

    let changes = UpdateOrganizationPayload {
        name,
        slug,
        logo: payload.logo,
        metadata: payload.metadata,
    };

    let org = task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| {
            tracing::error!("failed to get db connection: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            )
        })?;

        diesel::update(organization::table.filter(organization::id.eq(&org_id)))
            .set(&changes)
            .get_result::<Organization>(&mut conn)
            .map_err(|e| match e {
                DieselError::NotFound => {
                    (StatusCode::NOT_FOUND, "organization not found".to_string())
                }
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                    tracing::warn!(
                        "duplicate slug on update_organization_by_id: {}",
                        info.message()
                    );
                    (
                        StatusCode::CONFLICT,
                        "an organization with that slug already exists".to_string(),
                    )
                }
                other => {
                    tracing::error!("failed to update organization: {other}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    )
                }
            })
    })
    .await
    .map_err(|e| {
        tracing::error!("update_organization_by_id task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok((StatusCode::OK, Json(org)))
}

pub async fn delete_organization_by_id(
    State(pool): State<DbPool>,
    Path(org_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if Uuid::parse_str(&org_id).is_err() {
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

        let affected = diesel::delete(organization::table.filter(organization::id.eq(&org_id)))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("failed to delete organization: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            })?;

        if affected == 0 {
            return Err((StatusCode::NOT_FOUND, "organization not found".to_string()));
        }

        Ok(())
    })
    .await
    .map_err(|e| {
        tracing::error!("delete_organization_by_id task panicked: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        )
    })??;

    Ok(StatusCode::NO_CONTENT)
}