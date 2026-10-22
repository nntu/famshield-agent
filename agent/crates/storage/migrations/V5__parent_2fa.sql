-- Two-Factor Authentication (2FA TOTP RFC 6238) for Parents
CREATE TABLE IF NOT EXISTS parent_2fa_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    enabled INTEGER NOT NULL DEFAULT 0,
    encrypted_secret TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_verified_at TEXT
);

CREATE TABLE IF NOT EXISTS parent_backup_codes (
    code_hash TEXT PRIMARY KEY,
    used INTEGER NOT NULL DEFAULT 0,
    used_at TEXT
);
