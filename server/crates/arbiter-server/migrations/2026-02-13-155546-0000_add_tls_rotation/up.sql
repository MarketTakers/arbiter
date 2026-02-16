-- История всех сертификатов
CREATE TABLE IF NOT EXISTS tls_certificates (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    cert BLOB NOT NULL,              -- DER-encoded
    cert_key BLOB NOT NULL,          -- PEM-encoded
    not_before INTEGER NOT NULL,     -- Unix timestamp
    not_after INTEGER NOT NULL,      -- Unix timestamp
    created_at INTEGER NOT NULL DEFAULT(unixepoch('now')),
    is_active BOOLEAN NOT NULL DEFAULT 0  -- Только один active=1
) STRICT;

CREATE INDEX idx_tls_certificates_active ON tls_certificates(is_active, not_after);

-- Tracking процесса ротации
CREATE TABLE IF NOT EXISTS tls_rotation_state (
    id INTEGER NOT NULL PRIMARY KEY CHECK(id = 1),  -- Singleton
    state TEXT NOT NULL DEFAULT('normal') CHECK(state IN ('normal', 'initiated', 'waiting_acks', 'ready')),
    new_cert_id INTEGER REFERENCES tls_certificates(id),
    initiated_at INTEGER,
    timeout_at INTEGER  -- Таймаут для ожидания ACKs (initiated_at + 7 дней)
) STRICT;

-- Tracking ACKs от клиентов
CREATE TABLE IF NOT EXISTS rotation_client_acks (
    rotation_id INTEGER NOT NULL,    -- Ссылка на new_cert_id
    client_key TEXT NOT NULL,        -- Публичный ключ клиента (hex)
    ack_received_at INTEGER NOT NULL DEFAULT(unixepoch('now')),
    PRIMARY KEY (rotation_id, client_key)
) STRICT;

-- Audit trail событий ротации
CREATE TABLE IF NOT EXISTS tls_rotation_history (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    cert_id INTEGER NOT NULL REFERENCES tls_certificates(id),
    event_type TEXT NOT NULL CHECK(event_type IN ('created', 'rotation_initiated', 'acks_complete', 'activated', 'timeout')),
    timestamp INTEGER NOT NULL DEFAULT(unixepoch('now')),
    details TEXT  -- JSON с доп. информацией
) STRICT;

-- Миграция существующего сертификата
INSERT INTO tls_certificates (id, cert, cert_key, not_before, not_after, is_active, created_at)
SELECT
    1,
    cert,
    cert_key,
    unixepoch('now') as not_before,
    unixepoch('now') + (90 * 24 * 60 * 60) as not_after,  -- 90 дней
    1 as is_active,
    unixepoch('now')
FROM arbiter_settings WHERE id = 1;

-- Инициализация rotation_state
INSERT INTO tls_rotation_state (id, state) VALUES (1, 'normal');

-- Добавить ссылку на текущий сертификат
ALTER TABLE arbiter_settings ADD COLUMN current_cert_id INTEGER REFERENCES tls_certificates(id);
UPDATE arbiter_settings SET current_cert_id = 1 WHERE id = 1;
