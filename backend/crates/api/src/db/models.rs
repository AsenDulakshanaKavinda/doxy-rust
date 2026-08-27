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
    pub emailVerified: bool,
    pub image: Option<String>,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
    pub role: Option<String>,
    pub banned: Option<bool>,
    pub banReason: Option<String>,
    pub banExpires: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = session)]
#[diesel(belongs_to(User, foreign_key = userId))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub expiresAt: DateTime<Utc>,
    pub token: String,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
    pub ipAddress: Option<String>,
    pub userAgent: Option<String>,
    pub userId: String,
    pub activeOrganizationId: Option<String>,
    pub impersonatedBy: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = account)]
#[diesel(belongs_to(User, foreign_key = userId))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub id: String,
    pub accountId: String,
    pub providerId: String,
    pub userId: String,
    pub accessToken: Option<String>,
    pub refreshToken: Option<String>,
    pub idToken: Option<String>,
    pub accessTokenExpiresAt: Option<DateTime<Utc>>,
    pub refreshTokenExpiresAt: Option<DateTime<Utc>>,
    pub scope: Option<String>,
    pub password: Option<String>,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = verification)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Verification {
    pub id: String,
    pub identifier: String,
    pub value: String,
    pub expiresAt: DateTime<Utc>,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = rateLimit)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RateLimit {
    pub id: String,
    pub key: String,
    pub count: i32,
    pub lastRequest: i64,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = organization)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub createdAt: DateTime<Utc>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = member)]
#[diesel(belongs_to(Organization, foreign_key = organizationId))]
#[diesel(belongs_to(User, foreign_key = userId))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Member {
    pub id: String,
    pub organizationId: String,
    pub userId: String,
    pub role: String,
    pub createdAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = invitation)]
#[diesel(belongs_to(Organization, foreign_key = organizationId))]
#[diesel(belongs_to(User, foreign_key = inviterId))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Invitation {
    pub id: String,
    pub organizationId: String,
    pub email: String,
    pub role: Option<String>,
    pub status: String,
    pub expiresAt: DateTime<Utc>,
    pub createdAt: DateTime<Utc>,
    pub inviterId: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = avatar)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Avatar {
    pub id: String,
    pub userId: String,
    pub contentType: String,
    pub data: Vec<u8>,
    pub updatedAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = adminLog)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AdminLog {
    pub id: String,
    pub actorId: Option<String>,
    pub actorName: Option<String>,
    pub action: String,
    pub description: String,
    pub targetId: Option<String>,
    pub createdAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = appSetting)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AppSetting {
    pub id: String,
    pub allowSignUps: bool,
    pub enforceTwoFactor: bool,
    pub maintenanceMode: bool,
    pub chatRetentionMonths: Option<i32>,
    pub updatedByUserId: Option<String>,
    pub updatedAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = subscription)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Subscription {
    pub id: String,
    pub organizationId: String,
    pub stripeCustomerId: Option<String>,
    pub stripeSubscriptionId: Option<String>,
    pub status: Option<String>,
    pub planId: String,
    pub priceId: Option<String>,
    pub interval: Option<String>,
    pub currentPeriodEnd: Option<DateTime<Utc>>,
    pub cancelAtPeriodEnd: bool,
    pub cardBrand: Option<String>,
    pub cardLast4: Option<String>,
    pub cardExpMonth: Option<i32>,
    pub cardExpYear: Option<i32>,
    pub source: String,
    pub grantedByUserId: Option<String>,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = document)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Document {
    pub id: String,
    pub organizationId: String,
    pub userId: String,
    pub name: String,
    pub contentType: String,
    pub sizeBytes: i32,
    pub pageCount: Option<i32>,
    pub data: Vec<u8>,
    pub text: Option<String>,
    pub status: DocumentStatus,
    pub failureReason: Option<String>,
    pub createdAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = chat)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chat {
    pub id: String,
    pub organizationId: String,
    pub userId: String,
    pub title: String,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = chatDocument)]
#[diesel(primary_key(chatId, documentId))]
#[diesel(belongs_to(Chat, foreign_key = chatId))]
#[diesel(belongs_to(Document, foreign_key = documentId))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatDocument {
    pub chatId: String,
    pub documentId: String,
    pub position: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = message)]
#[diesel(belongs_to(Chat, foreign_key = chatId))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: String,
    pub chatId: String,
    pub role: MessageRole,
    pub content: String,
    pub sources: Option<Json>,
    pub hidden: bool,
    pub feedback: Option<MessageFeedback>,
    pub createdAt: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = questionEvent)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuestionEvent {
    pub id: String,
    pub organizationId: String,
    pub userId: String,
    pub chatId: Option<String>,
    pub createdAt: DateTime<Utc>,
}