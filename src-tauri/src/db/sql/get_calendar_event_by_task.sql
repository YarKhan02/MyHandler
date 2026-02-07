-- Get Google Calendar event ID for a task
SELECT google_event_id FROM calendar_events WHERE task_id = ?;
