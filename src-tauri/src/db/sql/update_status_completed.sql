-- Complete a task: set status to completed, set completed_at timestamp
UPDATE tasks 
SET status = ?1,
    completed_at = ?2,
    updated_at = ?3
WHERE id = ?4
