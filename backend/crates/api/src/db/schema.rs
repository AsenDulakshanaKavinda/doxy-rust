// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "document_status"))]
    pub struct DocumentStatus;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "message_feedback"))]
    pub struct MessageFeedback;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "message_role"))]
    pub struct MessageRole;
}

diesel::table! {
    account (id) {
        id -> Text,
        account_id -> Text,
        provider_id -> Text,
        user_id -> Text,
        access_token -> Nullable<Text>,
        refresh_token -> Nullable<Text>,
        id_token -> Nullable<Text>,
        access_token_expires_at -> Nullable<Timestamptz>,
        refresh_token_expires_at -> Nullable<Timestamptz>,
        scope -> Nullable<Text>,
        password -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    admin_log (id) {
        id -> Text,
        actor_id -> Nullable<Text>,
        actor_name -> Nullable<Text>,
        action -> Text,
        description -> Text,
        target_id -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    app_setting (id) {
        id -> Text,
        allow_sign_ups -> Bool,
        enforce_two_factor -> Bool,
        maintenance_mode -> Bool,
        chat_retention_months -> Nullable<Int4>,
        updated_by_user_id -> Nullable<Text>,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    avatar (id) {
        id -> Text,
        user_id -> Text,
        content_type -> Text,
        data -> Bytea,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    chat (id) {
        id -> Text,
        organization_id -> Text,
        user_id -> Text,
        title -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    chat_document (chat_id, document_id) {
        chat_id -> Text,
        document_id -> Text,
        position -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DocumentStatus;

    document (id) {
        id -> Text,
        organization_id -> Text,
        user_id -> Text,
        name -> Text,
        content_type -> Text,
        size_bytes -> Int4,
        page_count -> Nullable<Int4>,
        data -> Bytea,
        text -> Nullable<Text>,
        status -> DocumentStatus,
        failure_reason -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    invitation (id) {
        id -> Text,
        organization_id -> Text,
        email -> Text,
        role -> Nullable<Text>,
        status -> Text,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        inviter_id -> Text,
    }
}

diesel::table! {
    member (id) {
        id -> Text,
        organization_id -> Text,
        user_id -> Text,
        role -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MessageRole;
    use super::sql_types::MessageFeedback;

    message (id) {
        id -> Text,
        chat_id -> Text,
        role -> MessageRole,
        content -> Text,
        sources -> Nullable<Jsonb>,
        hidden -> Bool,
        feedback -> Nullable<MessageFeedback>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    organization (id) {
        id -> Text,
        name -> Text,
        slug -> Text,
        logo -> Nullable<Text>,
        created_at -> Timestamptz,
        metadata -> Nullable<Text>,
    }
}

diesel::table! {
    question_event (id) {
        id -> Text,
        organization_id -> Text,
        user_id -> Text,
        chat_id -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    rate_limit (id) {
        id -> Text,
        key -> Text,
        count -> Int4,
        last_request -> Int8,
    }
}

diesel::table! {
    session (id) {
        id -> Text,
        expires_at -> Timestamptz,
        token -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        user_id -> Text,
        active_organization_id -> Nullable<Text>,
        impersonated_by -> Nullable<Text>,
    }
}

diesel::table! {
    subscription (id) {
        id -> Text,
        organization_id -> Text,
        stripe_customer_id -> Nullable<Text>,
        stripe_subscription_id -> Nullable<Text>,
        status -> Nullable<Text>,
        plan_id -> Text,
        price_id -> Nullable<Text>,
        interval -> Nullable<Text>,
        current_period_end -> Nullable<Timestamptz>,
        cancel_at_period_end -> Bool,
        card_brand -> Nullable<Text>,
        card_last4 -> Nullable<Text>,
        card_exp_month -> Nullable<Int4>,
        card_exp_year -> Nullable<Int4>,
        source -> Text,
        granted_by_user_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user (id) {
        id -> Text,
        name -> Text,
        email -> Text,
        email_verified -> Bool,
        image -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        role -> Nullable<Text>,
        banned -> Nullable<Bool>,
        ban_reason -> Nullable<Text>,
        ban_expires -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    verification (id) {
        id -> Text,
        identifier -> Text,
        value -> Text,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(account -> user (user_id));
diesel::joinable!(chat_document -> chat (chat_id));
diesel::joinable!(chat_document -> document (document_id));
diesel::joinable!(invitation -> organization (organization_id));
diesel::joinable!(invitation -> user (inviter_id));
diesel::joinable!(member -> organization (organization_id));
diesel::joinable!(member -> user (user_id));
diesel::joinable!(message -> chat (chat_id));
diesel::joinable!(session -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    admin_log,
    app_setting,
    avatar,
    chat,
    chat_document,
    document,
    invitation,
    member,
    message,
    organization,
    question_event,
    rate_limit,
    session,
    subscription,
    user,
    verification,
);
