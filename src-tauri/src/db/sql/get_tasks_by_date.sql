SELECT id, title, notes, status, created_at, updated_at, deadline, 
       has_calendar_integration, calendar_email, reminder_frequency, 
       started_at, paused_at, completed_at 
FROM tasks 
WHERE created_at >= ?1 AND created_at <= ?2 
ORDER BY created_at DESC