-- Insert or replace calendar event for a task
INSERT OR REPLACE INTO calendar_events (task_id, google_event_id, updated_at, synced_at) 
VALUES (?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
