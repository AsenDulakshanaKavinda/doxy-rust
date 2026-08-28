// src/db/models.rs

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::schema::*;

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::db::schema::sql_types::DocumentStatus"]
pub enum DocumentStatus {
    PROCESSING,
    READY,
    FAILED,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::db::schema::sql_types::MessageRole"]
pub enum MessageRole {
    USER,
    ASSISTANT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::db::schema::sql_types::MessageFeedback"]
pub enum MessageFeedback {
    UP,
    DOWN,
}

// --- Struct Models ---

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: Option<String>,
    pub banned: Option<bool>,
    pub ban_reason: Option<String>,
    pub ban_expires: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = session)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub expires_at: DateTime<Utc>,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub user_id: String,
    pub active_organization_id: Option<String>,
    pub impersonated_byy: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = account)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub id: String,
    pub account_id: String,
    pub provider_id: String,
    pub user_id: String,
    pub access_token: Option<String>,
    pub refreshToken: Option<String>,
    pub id_token: Option<String>,
    pub access_token_expires_at: Option<DateTime<Utc>>,
    pub refresh_token_expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
    pub password: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = verification)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Verification {
    pub id: String,
    pub identifier: String,
    pub value: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = rate_limit)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RateLimit {
    pub id: String,
    pub key: String,
    pub count: i32,
    pub last_request: i64,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = organization)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = member)]
#[diesel(belongs_to(Organization, foreign_key = organization_id))]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Member {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = invitation)]
#[diesel(belongs_to(Organization, foreign_key = organization_id))]
#[diesel(belongs_to(User, foreign_key = inviter_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Invitation {
    pub id: String,
    pub organization_id: String,
    pub email: String,
    pub role: Option<String>,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub inviter_id: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = avatar)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Avatar {
    pub id: String,
    pub user_id: String,
    pub content_type: String,
    pub data: Vec<u8>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = admin_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AdminLog {
    pub id: String,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub description: String,
    pub target_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = app_setting)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AppSetting {
    pub id: String,
    pub allow_sign_ups: bool,
    pub enforce_two_factor: bool,
    pub maintenance_mode: bool,
    pub chat_retention_months: Option<i32>,
    pub updated_by_user_id: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = subscription)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Subscription {
    pub id: String,
    pub organization_id: String,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub status: Option<String>,
    pub plan_id: String,
    pub price_id: Option<String>,
    pub interval: Option<String>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub card_brand: Option<String>,
    pub card_last4: Option<String>,
    pub card_exp_month: Option<i32>,
    pub card_exp_year: Option<i32>,
    pub source: String,
    pub granted_by_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = document)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Document {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub name: String,
    pub content_type: String,
    pub size_bytes: i32,
    pub page_count: Option<i32>,
    pub data: Vec<u8>,
    pub text: Option<String>,
    pub status: DocumentStatus,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chat {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = chat_document)]
#[diesel(primary_key(chat_id, document_id))]
#[diesel(belongs_to(Chat, foreign_key = chat_id))]
#[diesel(belongs_to(Document, foreign_key = document_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatDocument {
    pub chat_id: String,
    pub document_id: String,
    pub position: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = message)]
#[diesel(belongs_to(Chat, foreign_key = chat_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub role: MessageRole,
    pub content: String,
    pub sources: Option<Json>,
    pub hidden: bool,
    pub feedback: Option<MessageFeedback>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = question_event)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuestionEvent {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub chat_id: Option<String>,
    pub created_at: DateTime<Utc>,
}