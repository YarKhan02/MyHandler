-- Tasks table - main table for storing task information        
CREATE TABLE IF NOT EXISTS tasks (
    id BLOB PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    notes TEXT,
    status VARCHAR(20) NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    deadline DATETIME,
    has_calendar_integration BOOLEAN NOT NULL DEFAULT 0,
    calendar_email VARCHAR(255),
    reminder_frequency VARCHAR(20) NOT NULL DEFAULT 'none',
    started_at DATETIME,
    paused_at DATETIME,
    completed_at DATETIME
);

CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);