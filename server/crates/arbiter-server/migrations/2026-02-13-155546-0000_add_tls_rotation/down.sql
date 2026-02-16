-- Rollback TLS rotation tables

-- Удалить добавленную колонку из arbiter_settings
ALTER TABLE arbiter_settings DROP COLUMN current_cert_id;

-- Удалить таблицы в обратном порядке
DROP TABLE IF EXISTS tls_rotation_history;
DROP TABLE IF EXISTS rotation_client_acks;
DROP TABLE IF EXISTS tls_rotation_state;
DROP INDEX IF EXISTS idx_tls_certificates_active;
DROP TABLE IF EXISTS tls_certificates;
