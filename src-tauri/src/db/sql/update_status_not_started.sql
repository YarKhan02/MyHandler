-- Reset a task: set status to not-started, clear all timestamps
UPDATE tasks 
SET status = ?1,
    started_at = NULL,
    paused_at = NULL,
    completed_at = NULL,
    updated_at = ?2
WHERE id = ?3
