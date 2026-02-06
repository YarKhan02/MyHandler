-- Drop existing settings table if exists (for development)
DROP TABLE IF EXISTS settings;

-- Settings table for user preferences
-- Single row table (id always = 1) for desktop app settings
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    dark_mode BOOLEAN NOT NULL DEFAULT 0,
    notifications_enabled BOOLEAN NOT NULL DEFAULT 1,
    default_reminder_frequency VARCHAR(20) NOT NULL DEFAULT 'none' CHECK (
        default_reminder_frequency IN ('none', 'hourly', 'every-3-hours', 'daily')
    ),
    calendar_integration_enabled BOOLEAN NOT NULL DEFAULT 0,
    calendar_email VARCHAR(255),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default settings row
INSERT INTO settings (id) VALUES (1);
