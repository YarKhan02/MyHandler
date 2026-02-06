-- Pause a task: set status to paused, set paused_at timestamp
UPDATE tasks 
SET status = ?1,
    paused_at = ?2,
    updated_at = ?3
WHERE id = ?4
