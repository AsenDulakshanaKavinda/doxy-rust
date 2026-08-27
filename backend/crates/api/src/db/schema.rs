// src/db/schema.rs

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "DocumentStatus"))]
    pub struct DocumentStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "MessageRole"))]
    pub struct MessageRole;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "MessageFeedback"))]
    pub struct MessageFeedback;
}

diesel::table! {
    use diesel::sql_types::*;

    user (id) {
        id -> Text,
        name -> Text,
        email -> Text,
        emailVerified -> Bool,
        image -> Nullable<Text>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
        role -> Nullable<Text>,
        banned -> Nullable<Bool>,
        banReason -> Nullable<Text>,
        banExpires -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    session (id) {
        id -> Text,
        expiresAt -> Timestamptz,
        token -> Text,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
        ipAddress -> Nullable<Text>,
        userAgent -> Nullable<Text>,
        userId -> Text,
        activeOrganizationId -> Nullable<Text>,
        impersonatedBy -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    account (id) {
        id -> Text,
        accountId -> Text,
        providerId -> Text,
        userId -> Text,
        accessToken -> Nullable<Text>,
        refreshToken -> Nullable<Text>,
        idToken -> Nullable<Text>,
        accessTokenExpiresAt -> Nullable<Timestamptz>,
        refreshTokenExpiresAt -> Nullable<Timestamptz>,
        scope -> Nullable<Text>,
        password -> Nullable<Text>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    verification (id) {
        id -> Text,
        identifier -> Text,
        value -> Text,
        expiresAt -> Timestamptz,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    rateLimit (id) {
        id -> Text,
        key -> Text,
        count -> Int4,
        lastRequest -> BigInt,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    organization (id) {
        id -> Text,
        name -> Text,
        slug -> Text,
        logo -> Nullable<Text>,
        createdAt -> Timestamptz,
        metadata -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    member (id) {
        id -> Text,
        organizationId -> Text,
        userId -> Text,
        role -> Text,
        createdAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    invitation (id) {
        id -> Text,
        organizationId -> Text,
        email -> Text,
        role -> Nullable<Text>,
        status -> Text,
        expiresAt -> Timestamptz,
        createdAt -> Timestamptz,
        inviterId -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    avatar (id) {
        id -> Text,
        userId -> Text,
        contentType -> Text,
        data -> Bytea,
        updatedAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    adminLog (id) {
        id -> Text,
        actorId -> Nullable<Text>,
        actorName -> Nullable<Text>,
        action -> Text,
        description -> Text,
        targetId -> Nullable<Text>,
        createdAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    appSetting (id) {
        id -> Text,
        allowSignUps -> Bool,
        enforceTwoFactor -> Bool,
        maintenanceMode -> Bool,
        chatRetentionMonths -> Nullable<Int4>,
        updatedByUserId -> Nullable<Text>,
        updatedAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    subscription (id) {
        id -> Text,
        organizationId -> Text,
        stripeCustomerId -> Nullable<Text>,
        stripeSubscriptionId -> Nullable<Text>,
        status -> Nullable<Text>,
        planId -> Text,
        priceId -> Nullable<Text>,
        interval -> Nullable<Text>,
        currentPeriodEnd -> Nullable<Timestamptz>,
        cancelAtPeriodEnd -> Bool,
        cardBrand -> Nullable<Text>,
        cardLast4 -> Nullable<Text>,
        cardExpMonth -> Nullable<Int4>,
        cardExpYear -> Nullable<Int4>,
        source -> Text,
        grantedByUserId -> Nullable<Text>,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DocumentStatus;

    document (id) {
        id -> Text,
        organizationId -> Text,
        userId -> Text,
        name -> Text,
        contentType -> Text,
        sizeBytes -> Int4,
        pageCount -> Nullable<Int4>,
        data -> Bytea,
        text -> Nullable<Text>,
        status -> DocumentStatus,
        failureReason -> Nullable<Text>,
        createdAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    chat (id) {
        id -> Text,
        organizationId -> Text,
        userId -> Text,
        title -> Text,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    chatDocument (chatId, documentId) {
        chatId -> Text,
        documentId -> Text,
        position -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::{MessageRole, MessageFeedback};

    message (id) {
        id -> Text,
        chatId -> Text,
        role -> MessageRole,
        content -> Text,
        sources -> Nullable<Jsonb>,
        hidden -> Bool,
        feedback -> Nullable<MessageFeedback>,
        createdAt -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    questionEvent (id) {
        id -> Text,
        organizationId -> Text,
        userId -> Text,
        chatId -> Nullable<Text>,
        createdAt -> Timestamptz,
    }
}

diesel::joinable!(session -> user (userId));
diesel::joinable!(account -> user (userId));
diesel::joinable!(member -> organization (organizationId));
diesel::joinable!(member -> user (userId));
diesel::joinable!(invitation -> organization (organizationId));
diesel::joinable!(invitation -> user (inviterId));
diesel::joinable!(chatDocument -> chat (chatId));
diesel::joinable!(chatDocument -> document (documentId));
diesel::joinable!(message -> chat (chatId));

diesel::allow_tables_to_appear_in_same_query!(
    user,
    session,
    account,
    verification,
    rateLimit,
    organization,
    member,
    invitation,
    avatar,
    adminLog,
    appSetting,
    subscription,
    document,
    chat,
    chatDocument,
    message,
    questionEvent,
);