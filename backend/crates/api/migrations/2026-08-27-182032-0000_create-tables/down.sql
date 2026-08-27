-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS question_event;
DROP TABLE IF EXISTS message;
DROP TABLE IF EXISTS chat_document;
DROP TABLE IF EXISTS chat;
DROP TABLE IF EXISTS document;
DROP TABLE IF EXISTS subscription;
DROP TABLE IF EXISTS app_setting;
DROP TABLE IF EXISTS admin_log;
DROP TABLE IF EXISTS avatar;
DROP TABLE IF EXISTS invitation;
DROP TABLE IF EXISTS member;
DROP TABLE IF EXISTS organization;
DROP TABLE IF EXISTS rate_limit;
DROP TABLE IF EXISTS verification;
DROP TABLE IF EXISTS account;
DROP TABLE IF EXISTS session;
DROP TABLE IF EXISTS "user";

DROP TYPE IF EXISTS message_feedback;
DROP TYPE IF EXISTS message_role;
DROP TYPE IF EXISTS document_status;