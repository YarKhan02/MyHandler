-- Calendar OAuth credentials (single row)

CREATE TABLE IF NOT EXISTS calendar_credentials (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    email VARCHAR(255) NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    token_expiry DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Initialize with placeholder (will be updated on first auth)
INSERT OR IGNORE INTO calendar_credentials (id, email, access_token, refresh_token, token_expiry)
VALUES (1, '', '', '', CURRENT_TIMESTAMP);
