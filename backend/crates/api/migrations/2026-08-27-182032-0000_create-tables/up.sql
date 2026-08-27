-- Your SQL goes here
-- Doxy (Rust rebuild) — initial schema
--
-- Mirrors the Prisma schema. Better Auth owns: user, session, account,
-- verification, rate_limit, organization, member, invitation.
-- App-owned tables: avatar, admin_log, app_setting, subscription, document,
-- chat, chat_document, message, question_event.
--
-- FK constraints are only added where the Prisma schema declared an explicit
-- `@relation` — the deliberately-denormalised tables (avatar, admin_log,
-- app_setting, subscription, document, chat, question_event) keep their
-- organization_id/user_id/actor_id/etc. as plain columns with no FK, exactly
-- as commented in the Prisma source.

-- ── enums ───────────────────────────────────────────────────────────────
CREATE TYPE document_status AS ENUM ('PROCESSING', 'READY', 'FAILED');
CREATE TYPE message_role AS ENUM ('USER', 'ASSISTANT');
CREATE TYPE message_feedback AS ENUM ('UP', 'DOWN');

-- ── Better Auth tables ──────────────────────────────────────────────────
CREATE TABLE "user" (
    id             TEXT PRIMARY KEY,
    name           TEXT NOT NULL,
    email          TEXT NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    image          TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    role           TEXT,
    banned         BOOLEAN DEFAULT FALSE,
    ban_reason     TEXT,
    ban_expires    TIMESTAMPTZ,
    CONSTRAINT user_email_key UNIQUE (email)
);

CREATE TABLE session (
    id                     TEXT PRIMARY KEY,
    expires_at             TIMESTAMPTZ NOT NULL,
    token                  TEXT NOT NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    ip_address             TEXT,
    user_agent             TEXT,
    user_id                TEXT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    active_organization_id TEXT,
    impersonated_by        TEXT,
    CONSTRAINT session_token_key UNIQUE (token)
);
CREATE INDEX session_user_id_idx ON session (user_id);

CREATE TABLE account (
    id                       TEXT PRIMARY KEY,
    account_id               TEXT NOT NULL,
    provider_id              TEXT NOT NULL,
    user_id                  TEXT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    access_token             TEXT,
    refresh_token            TEXT,
    id_token                 TEXT,
    access_token_expires_at  TIMESTAMPTZ,
    refresh_token_expires_at TIMESTAMPTZ,
    scope                    TEXT,
    password                 TEXT,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX account_user_id_idx ON account (user_id);

CREATE TABLE verification (
    id         TEXT PRIMARY KEY,
    identifier TEXT NOT NULL,
    value      TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX verification_identifier_idx ON verification (identifier);

CREATE TABLE rate_limit (
    id           TEXT PRIMARY KEY,
    key          TEXT NOT NULL,
    count        INTEGER NOT NULL,
    last_request BIGINT NOT NULL,
    CONSTRAINT rate_limit_key_key UNIQUE (key)
);

CREATE TABLE organization (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    slug       TEXT NOT NULL,
    logo       TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    metadata   TEXT,
    CONSTRAINT organization_slug_key UNIQUE (slug)
);

CREATE TABLE member (
    id              TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL REFERENCES organization(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    role            TEXT NOT NULL DEFAULT 'member',
    created_at      TIMESTAMPTZ NOT NULL
);
CREATE INDEX member_organization_id_idx ON member (organization_id);
CREATE INDEX member_user_id_idx ON member (user_id);

CREATE TABLE invitation (
    id              TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL REFERENCES organization(id) ON DELETE CASCADE,
    email           TEXT NOT NULL,
    role            TEXT,
    status          TEXT NOT NULL DEFAULT 'pending',
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    inviter_id      TEXT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE
);
CREATE INDEX invitation_organization_id_idx ON invitation (organization_id);
CREATE INDEX invitation_email_idx ON invitation (email);

-- ── app-owned tables ────────────────────────────────────────────────────
CREATE TABLE avatar (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL,
    content_type TEXT NOT NULL,
    data         BYTEA NOT NULL,
    updated_at   TIMESTAMPTZ NOT NULL,
    CONSTRAINT avatar_user_id_key UNIQUE (user_id)
);

CREATE TABLE admin_log (
    id          TEXT PRIMARY KEY,
    actor_id    TEXT,
    actor_name  TEXT,
    action      TEXT NOT NULL,
    description TEXT NOT NULL,
    target_id   TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX admin_log_created_at_idx ON admin_log (created_at);

CREATE TABLE app_setting (
    id                    TEXT PRIMARY KEY DEFAULT 'app',
    allow_sign_ups        BOOLEAN NOT NULL DEFAULT TRUE,
    enforce_two_factor    BOOLEAN NOT NULL DEFAULT FALSE,
    maintenance_mode      BOOLEAN NOT NULL DEFAULT FALSE,
    chat_retention_months INTEGER DEFAULT 12,
    updated_by_user_id    TEXT,
    updated_at            TIMESTAMPTZ NOT NULL
);

CREATE TABLE subscription (
    id                      TEXT PRIMARY KEY,
    organization_id         TEXT NOT NULL,
    stripe_customer_id      TEXT,
    stripe_subscription_id  TEXT,
    status                  TEXT,
    plan_id                 TEXT NOT NULL DEFAULT 'free',
    price_id                TEXT,
    interval                TEXT,
    current_period_end      TIMESTAMPTZ,
    cancel_at_period_end    BOOLEAN NOT NULL DEFAULT FALSE,
    card_brand              TEXT,
    card_last4              TEXT,
    card_exp_month          INTEGER,
    card_exp_year           INTEGER,
    source                  TEXT NOT NULL DEFAULT 'stripe',
    granted_by_user_id      TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT subscription_organization_id_key UNIQUE (organization_id),
    CONSTRAINT subscription_stripe_customer_id_key UNIQUE (stripe_customer_id),
    CONSTRAINT subscription_stripe_subscription_id_key UNIQUE (stripe_subscription_id)
);

CREATE TABLE document (
    id              TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    user_id         TEXT NOT NULL,
    name            TEXT NOT NULL,
    content_type    TEXT NOT NULL,
    size_bytes      INTEGER NOT NULL,
    page_count      INTEGER,
    data            BYTEA NOT NULL,
    text            TEXT,
    status          document_status NOT NULL DEFAULT 'PROCESSING',
    failure_reason  TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX document_organization_id_created_at_idx ON document (organization_id, created_at);

CREATE TABLE chat (
    id              TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    user_id         TEXT NOT NULL,
    title           TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX chat_organization_id_updated_at_idx ON chat (organization_id, updated_at);

CREATE TABLE chat_document (
    chat_id     TEXT NOT NULL REFERENCES chat(id) ON DELETE CASCADE,
    document_id TEXT NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    PRIMARY KEY (chat_id, document_id)
);
CREATE INDEX chat_document_document_id_idx ON chat_document (document_id);

CREATE TABLE message (
    id         TEXT PRIMARY KEY,
    chat_id    TEXT NOT NULL REFERENCES chat(id) ON DELETE CASCADE,
    role       message_role NOT NULL,
    content    TEXT NOT NULL,
    sources    JSONB,
    hidden     BOOLEAN NOT NULL DEFAULT FALSE,
    feedback   message_feedback,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX message_chat_id_created_at_idx ON message (chat_id, created_at);

CREATE TABLE question_event (
    id              TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    user_id         TEXT NOT NULL,
    chat_id         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX question_event_organization_id_created_at_idx ON question_event (organization_id, created_at);