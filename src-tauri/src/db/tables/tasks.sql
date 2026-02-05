CREATE TABLE IF NOT EXISTS tasks (
    id BLOB PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    notes TEXT,
    status VARCHAR(20) NOT NULL,
    task_date DATE NOT NULL,
    created_at DATETIME NOT NULL,
    started_at DATETIME,
    paused_at DATETIME,
    completed_at DATETIME
);

CREATE INDEX IF NOT EXISTS idx_tasks_task_date ON tasks(task_date);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);