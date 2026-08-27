// Diesel models for the Doxy schema.


use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::db::schema::*;



// ── enums ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::DocumentStatus"]
pub enum DocumentStatus {
    Processing,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::MessageRole"]
pub enum MessageRoleDb {
    User,
    Assistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::MessageFeedback"]
pub enum MessageFeedbackDb {
    Up,
    Down,
}

// ── Better Auth tables ──────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = user, primary_key(id), check_for_backend(diesel::pg::Pg))]
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

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user)]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub email: &'a str,
    pub email_verified: bool,
    pub image: Option<&'a str>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = session, primary_key(id), belongs_to(User, foreign_key = user_id), check_for_backend(diesel::pg::Pg))]
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
    pub impersonated_by: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = session)]
pub struct NewSession<'a> {
    pub id: &'a str,
    pub expires_at: DateTime<Utc>,
    pub token: &'a str,
    pub ip_address: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub user_id: &'a str,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = account, primary_key(id), belongs_to(User, foreign_key = user_id), check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub id: String,
    pub account_id: String,
    pub provider_id: String,
    pub user_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub access_token_expires_at: Option<DateTime<Utc>>,
    pub refresh_token_expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = account)]
pub struct NewAccount<'a> {
    pub id: &'a str,
    pub account_id: &'a str,
    pub provider_id: &'a str,
    pub user_id: &'a str,
    pub password: Option<&'a str>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = verification, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct Verification {
    pub id: String,
    pub identifier: String,
    pub value: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = verification)]
pub struct NewVerification<'a> {
    pub id: &'a str,
    pub identifier: &'a str,
    pub value: &'a str,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = rate_limit, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct RateLimit {
    pub id: String,
    pub key: String,
    pub count: i32,
    pub last_request: i64,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = rate_limit)]
pub struct NewRateLimit<'a> {
    pub id: &'a str,
    pub key: &'a str,
    pub count: i32,
    pub last_request: i64,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = organization, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = organization)]
pub struct NewOrganization<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub slug: &'a str,
    pub logo: Option<&'a str>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<&'a str>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = member, primary_key(id), belongs_to(Organization, foreign_key = organization_id), belongs_to(User, foreign_key = user_id), check_for_backend(diesel::pg::Pg))]
pub struct Member {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = member)]
pub struct NewMember<'a> {
    pub id: &'a str,
    pub organization_id: &'a str,
    pub user_id: &'a str,
    pub role: &'a str,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = invitation, primary_key(id), belongs_to(Organization, foreign_key = organization_id), check_for_backend(diesel::pg::Pg))]
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

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = invitation)]
pub struct NewInvitation<'a> {
    pub id: &'a str,
    pub organization_id: &'a str,
    pub email: &'a str,
    pub role: Option<&'a str>,
    pub expires_at: DateTime<Utc>,
    pub inviter_id: &'a str,
}

// ── app-owned tables ────────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = avatar, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct Avatar {
    pub id: String,
    pub user_id: String,
    pub content_type: String,
    #[serde(skip_serializing)]
    pub data: Vec<u8>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = avatar)]
pub struct NewAvatar<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub content_type: &'a str,
    pub data: &'a [u8],
}

/// Append-only; there is deliberately no update/changeset for this table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = admin_log, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct AdminLog {
    pub id: String,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub description: String,
    pub target_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = admin_log)]
pub struct NewAdminLog<'a> {
    pub id: &'a str,
    pub actor_id: Option<&'a str>,
    pub actor_name: Option<&'a str>,
    pub action: &'a str,
    pub description: &'a str,
    pub target_id: Option<&'a str>,
}

/// Single-row table; use `AppSetting::get_or_default` in the repo layer
/// rather than an `Insertable`, since the row is seeded by migration/upsert.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = app_setting, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct AppSetting {
    pub id: String,
    pub allow_sign_ups: bool,
    pub enforce_two_factor: bool,
    pub maintenance_mode: bool,
    pub chat_retention_months: Option<i32>,
    pub updated_by_user_id: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = subscription, primary_key(id), check_for_backend(diesel::pg::Pg))]
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
    /// `"stripe"` when money moved, `"admin"` when comped from the console.
    pub source: String,
    pub granted_by_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = subscription)]
pub struct NewSubscription<'a> {
    pub id: &'a str,
    pub organization_id: &'a str,
    pub stripe_customer_id: Option<&'a str>,
    pub source: &'a str,
    pub granted_by_user_id: Option<&'a str>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = document, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct Document {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub name: String,
    pub content_type: String,
    pub size_bytes: i32,
    pub page_count: Option<i32>,
    #[serde(skip_serializing)]
    pub data: Vec<u8>,
    pub text: Option<String>,
    pub status: DocumentStatus,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = document)]
pub struct NewDocument<'a> {
    pub id: &'a str,
    pub organization_id: &'a str,
    pub user_id: &'a str,
    pub name: &'a str,
    pub content_type: &'a str,
    pub size_bytes: i32,
    pub page_count: Option<i32>,
    pub data: &'a [u8],
    pub status: DocumentStatus,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = document)]
pub struct DocumentProcessed<'a> {
    pub text: Option<&'a str>,
    pub page_count: Option<i32>,
    pub status: DocumentStatus,
    pub failure_reason: Option<&'a str>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = chat, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct Chat {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = chat)]
pub struct NewChat<'a> {
    pub id: &'a str,
    pub organization_id: &'a str,
    pub user_id: &'a str,
    pub title: &'a str,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = chat_document, primary_key(chat_id, document_id), belongs_to(Chat, foreign_key = chat_id), belongs_to(Document, foreign_key = document_id), check_for_backend(diesel::pg::Pg))]
pub struct ChatDocument {
    pub chat_id: String,
    pub document_id: String,
    /// The citation index — the `[n]` markers in an answer resolve through this.
    pub position: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = chat_document)]
pub struct NewChatDocument<'a> {
    pub chat_id: &'a str,
    pub document_id: &'a str,
    pub position: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = message, primary_key(id), belongs_to(Chat, foreign_key = chat_id), check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub role: MessageRoleDb,
    pub content: String,
    /// Citations Claude returned, resolved to `{ index, document, page }`.
    pub sources: Option<Json>,
    /// The seeded brief-analysis request: sent for context, never rendered.
    pub hidden: bool,
    pub feedback: Option<MessageFeedbackDb>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = message)]
pub struct NewMessage<'a> {
    pub id: &'a str,
    pub chat_id: &'a str,
    pub role: MessageRoleDb,
    pub content: &'a str,
    pub sources: Option<Json>,
    pub hidden: bool,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = message)]
pub struct MessageFeedbackUpdate {
    pub feedback: Option<MessageFeedbackDb>,
}

/// One row per question the workspace is charged for. Nothing cascades into
/// this table, so it outlives deleted chats and cleared history on purpose.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = question_event, primary_key(id), check_for_backend(diesel::pg::Pg))]
pub struct QuestionEvent {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub chat_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = question_event)]
pub struct NewQuestionEvent<'a> {
    pub id: &'a str,
    pub organization_id: &'a str,
    pub user_id: &'a str,
    pub chat_id: Option<&'a str>,
}