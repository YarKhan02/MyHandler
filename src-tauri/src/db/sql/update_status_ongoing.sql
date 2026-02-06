-- Start or resume a task: set status to ongoing, preserve started_at, clear paused_at
UPDATE tasks 
SET status = ?1, 
    started_at = COALESCE(started_at, ?2),
    paused_at = NULL,
    updated_at = ?3
WHERE id = ?4
