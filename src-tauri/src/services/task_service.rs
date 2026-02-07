use chrono::Utc;
use crate::services::calendar_service;
use crate::db::{self, Database, insert};
use crate::structs::task_struct::{Task, Status};
use crate::helpers::parse_date::parse_date_range;
use crate::structs::dto::{TaskData, DateQuery, TaskId};

pub fn create_task(payload: TaskData, db: &Database) -> Result<Task, String> {
    // Parse ISO 8601 datetime string
    let created_at = payload.created_at.parse::<chrono::DateTime<Utc>>()
        .map_err(|e| format!("Invalid datetime format: {}", e))?;
    
    // Use the global database connection
    let conn = db.get_connection();

    let task = Task::new(&payload.title, created_at, None);
    insert(&conn, &task).map_err(|e| format!("Failed to insert task: {}", e))?;
    
    Ok(task)
}

pub fn get_tasks_by_date(payload: DateQuery, db: &Database) -> Result<Vec<Task>, String> {
    let (start_of_day, end_of_day) = parse_date_range(&payload.date)?;
    
    let sql = include_str!("../db/sql/get_tasks_by_date.sql");
    let conn = db.get_connection();
    let tasks = db::query_tasks_by_date_range(&conn, start_of_day, end_of_day, sql)
        .map_err(|e| format!("Failed to query tasks: {}", e))?;
    
    Ok(tasks)
}

pub fn get_tasks_by_date_not_completed(payload: DateQuery, db: &Database) -> Result<Vec<Task>, String> {
    let (start_of_day, end_of_day) = parse_date_range(&payload.date)?;
    
    let sql = include_str!("../db/sql/get_tasks_by_date_not_completed.sql");
    let conn = db.get_connection();
    let tasks = db::query_tasks_by_date_range(&conn, start_of_day, end_of_day, sql)
        .map_err(|e| format!("Failed to query tasks: {}", e))?;
    
    Ok(tasks)
}

pub fn start_task(payload: TaskId, db: &Database) -> Result<Task, String> {
    let conn = db.get_connection();
    
    db::update_task_status(&conn, &payload.id, Status::Ongoing)
        .map_err(|e| format!("Failed to start task: {}", e))
}

pub fn pause_task(payload: TaskId, db: &Database) -> Result<Task, String> {
    let (task, event_id) = {
        let conn = db.get_connection();
        
        let task = db::update_task_status(&conn, &payload.id, Status::Paused)
            .map_err(|e| format!("Failed to pause task: {}", e))?;
        
        let event_id = db::get_task_google_event_id(&conn, &payload.id)
            .map_err(|e| format!("Failed to get calendar event: {}", e))?;
        
        (task, event_id)
    }; // DB lock released here
    
    // If task has calendar event and deadline, remove reminders (pause alarms)
    if let Some(event_id) = event_id {
        if let Some(deadline) = task.deadline {
            println!("Pausing calendar reminders for task: {}", task.id);
            match tokio::runtime::Runtime::new()
                .map_err(|e| format!("Failed to create runtime: {}", e))?
                .block_on(calendar_service::update_task_calendar_event(
                    db,
                    &event_id,
                    &task.title,
                    task.notes.as_deref(),
                    deadline,
                    "", // Empty reminder_frequency to remove all reminders
                )) {
                Ok(_) => println!("Calendar reminders paused"),
                Err(e) if e == "EVENT_NOT_FOUND" => {
                    println!("Calendar event was deleted externally, clearing from database");
                    let conn = db.get_connection();
                    let _ = db::clear_task_google_event_id(&conn, &payload.id);
                }
                Err(e) => eprintln!("Warning: Failed to pause calendar reminders: {}", e),
            }
        }
    }
    
    Ok(task)
}

pub fn resume_task(payload: TaskId, db: &Database) -> Result<Task, String> {
    let (task, event_id) = {
        let conn = db.get_connection();
        
        let task = db::update_task_status(&conn, &payload.id, Status::Ongoing)
            .map_err(|e| format!("Failed to resume task: {}", e))?;
        
        let event_id = db::get_task_google_event_id(&conn, &payload.id)
            .map_err(|e| format!("Failed to get calendar event: {}", e))?;
        
        (task, event_id)
    }; // DB lock released here
    
    // If task has calendar event and deadline, restore reminders
    if let Some(event_id) = event_id {
        if let Some(deadline) = task.deadline {
            println!("Resuming calendar reminders for task: {}", task.id);
            let reminder_freq_str = String::from(task.reminder_frequency.clone());
            match tokio::runtime::Runtime::new()
                .map_err(|e| format!("Failed to create runtime: {}", e))?
                .block_on(calendar_service::update_task_calendar_event(
                    db,
                    &event_id,
                    &task.title,
                    task.notes.as_deref(),
                    deadline,
                    &reminder_freq_str, // Restore reminders from task settings
                )) {
                Ok(_) => println!("Calendar reminders resumed"),
                Err(e) if e == "EVENT_NOT_FOUND" => {
                    println!("Calendar event was deleted externally, clearing from database");
                    let conn = db.get_connection();
                    let _ = db::clear_task_google_event_id(&conn, &payload.id);
                }
                Err(e) => eprintln!("Warning: Failed to resume calendar reminders: {}", e),
            }
        }
    }
    
    Ok(task)
}

pub fn complete_task(payload: TaskId, db: &Database) -> Result<Task, String> {
    let (task, event_id) = {
        let conn = db.get_connection();
        
        let task = db::update_task_status(&conn, &payload.id, Status::Completed)
            .map_err(|e| format!("Failed to complete task: {}", e))?;
        
        let event_id = db::get_task_google_event_id(&conn, &payload.id)
            .map_err(|e| format!("Failed to get calendar event: {}", e))?;
        
        (task, event_id)
    }; // DB lock released here
    
    // If task has calendar event, delete it (task is completed)
    if let Some(event_id) = event_id {
        println!("Deleting calendar event for completed task: {}", task.id);
        if let Err(e) = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?
            .block_on(calendar_service::delete_task_calendar_event(db, &event_id)) 
        {
            eprintln!("Warning: Failed to delete calendar event: {}", e);
        } else {
            // Clear event ID from database
            let conn = db.get_connection();
            let _ = db::clear_task_google_event_id(&conn, &payload.id);
        }
    }
    
    Ok(task)
}

pub fn delete_task(payload: TaskId, db: &Database) -> Result<(), String> {
    // Scope 1: Get calendar event ID and release lock
    let event_id = {
        let conn = db.get_connection();
        
        db::get_task_google_event_id(&conn, &payload.id)
            .map_err(|e| format!("Failed to get calendar event: {}", e))?
    }; // DB lock released here
    
    // Delete calendar event from Google if exists
    if let Some(event_id) = event_id {
        println!("Deleting calendar event: {}", event_id);
        if let Err(e) = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?
            .block_on(calendar_service::delete_task_calendar_event(db, &event_id)) 
        {
            eprintln!("Warning: Failed to delete calendar event: {}", e);
        }
    }
    
    // Scope 2: Delete from database (get fresh connection)
    let deleted = {
        let conn = db.get_connection();
        db::delete_task_by_id(&conn, &payload.id)
            .map_err(|e| format!("Failed to delete task: {}", e))?
    };
    
    if deleted == 0 {
        Err("Task not found".to_string())
    } else {
        Ok(())
    }
}

pub fn get_task_by_id(payload: TaskId, db: &Database) -> Result<Task, String> {
    let conn = db.get_connection();
    
    db::get_task_by_id(&conn, &payload.id)
        .map_err(|e| format!("Failed to get task by ID: {}", e))
}

pub async fn update_task(payload: crate::structs::task_update::TaskUpdate, db: &Database) -> Result<Task, String> {
    use crate::structs::task_update::TaskUpdateParsed;
    
    println!("Updating task: {:?}", payload.id);
    
    // Scope 1: Get current state and update task in DB
    let (_current_task, current_event_id, updated_task, calendar_enabled, new_deadline, reminder_freq_for_event) = {
        let conn = db.get_connection();
        
        // Get current task and calendar event
        let current_task = db::get_task_by_id(&conn, &payload.id)
            .map_err(|e| format!("Failed to get current task: {}", e))?;
        let current_event_id = db::get_task_google_event_id(&conn, &payload.id)
            .map_err(|e| format!("Failed to get calendar event: {}", e))?;
        
        println!("Current task found, has event: {}", current_event_id.is_some());
        
        // Parse deadline if provided
        let deadline = if let Some(ref deadline_str) = payload.data.deadline {
            Some(Some(
                deadline_str.parse::<chrono::DateTime<chrono::Utc>>()
                    .map_err(|e| format!("Invalid deadline format: {}", e))?
            ))
        } else {
            None
        };
        
        // Convert notes - if provided, wrap in Some(Some) or Some(None)
        let notes = payload.data.notes.map(|n| {
            if n.is_empty() {
                None
            } else {
                Some(n)
            }
        });
        
        // Convert calendar_email
        let calendar_email = payload.data.calendar_email.map(|e| {
            if e.is_empty() {
                None
            } else {
                Some(e)
            }
        });
        
        // Get reminder frequency for later use (before moving payload.data)
        let default_freq = String::from(current_task.reminder_frequency.clone());
        let reminder_freq_for_event = payload.data.reminder_frequency.clone().unwrap_or(default_freq);
        
        let update_data = TaskUpdateParsed {
            title: payload.data.title,
            notes,
            deadline,
            has_calendar_integration: payload.data.has_calendar_integration,
            calendar_email,
            reminder_frequency: payload.data.reminder_frequency,
            updated_at: chrono::Utc::now(),
        };
        
        let updated_task = db::update_task(&conn, &payload.id, &update_data)
            .map_err(|e| format!("Failed to update task: {}", e))?;
        
        println!("Task updated in DB");
        
        // Calculate calendar state
        let calendar_enabled = payload.data.has_calendar_integration.unwrap_or(current_task.has_calendar_integration);
        let new_deadline = if let Some(Some(d)) = deadline { Some(d) } else { current_task.deadline };
        
        (current_task, current_event_id, updated_task, calendar_enabled, new_deadline, reminder_freq_for_event)
    }; // Connection dropped here!
    
    println!("Calendar enabled: {}, has deadline: {}", calendar_enabled, new_deadline.is_some());
    
    if calendar_enabled && new_deadline.is_some() {
        if let Some(existing_event_id) = current_event_id {
            // Event already exists, try to UPDATE it
            println!("Updating existing calendar event: {}", existing_event_id);
            match calendar_service::update_task_calendar_event(
                db,
                &existing_event_id,
                &updated_task.title,
                updated_task.notes.as_deref(),
                new_deadline.unwrap(),
                &reminder_freq_for_event,
            ).await {
                Ok(_) => {
                    println!("Calendar event updated successfully");
                }
                Err(e) if e == "EVENT_NOT_FOUND" => {
                    // Event was deleted externally, clear it from database to stay in sync
                    println!("Calendar event was deleted externally, clearing from database");
                    let conn = db.get_connection();
                    let _ = db::clear_task_google_event_id(&conn, &payload.id);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to update calendar event: {}", e);
                }
            }
        } else {
            // No event exists, CREATE new one
            println!("Creating new calendar event...");
            match calendar_service::create_task_calendar_event(
                db,
                &updated_task.title,
                updated_task.notes.as_deref(),
                new_deadline.unwrap(),
                &reminder_freq_for_event,
            ).await {
                Ok(event_id) => {
                    println!("Calendar event created: {}", event_id);
                    // Save event ID in calendar_events table (get fresh connection)
                    let conn = db.get_connection();
                    let _ = db::update_task_google_event_id(&conn, &payload.id, &event_id);
                }
                Err(e) => {
                    eprintln!("Failed to create calendar event: {}", e);
                    return Err(format!("Failed to create calendar event: {}", e));
                }
            }
        }
    } else if !calendar_enabled && current_event_id.is_some() {
        // Calendar disabled, delete existing event
        if let Some(event_id) = current_event_id {
            if let Err(e) = calendar_service::delete_task_calendar_event(db, &event_id).await {
                eprintln!("Warning: Failed to delete calendar event: {}", e);
            }
            let conn = db.get_connection();
            let _ = db::clear_task_google_event_id(&conn, &payload.id);
        }
    }
    
    // Return refreshed task (get fresh connection)
    let conn = db.get_connection();
    db::get_task_by_id(&conn, &payload.id)
        .map_err(|e| format!("Failed to get updated task: {}", e))
}
