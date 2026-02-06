use chrono::Utc;
use crate::db::{self, Database, insert};
use crate::structs::dto::{TaskData, DateQuery, TaskId};
use crate::structs::task_struct::{Task, Status};
use crate::helpers::parse_date::parse_date_range;

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
    let conn = db.get_connection();
    
    db::update_task_status(&conn, &payload.id, Status::Paused)
        .map_err(|e| format!("Failed to pause task: {}", e))
}

pub fn resume_task(payload: TaskId, db: &Database) -> Result<Task, String> {
    let conn = db.get_connection();
    
    db::update_task_status(&conn, &payload.id, Status::Ongoing)
        .map_err(|e| format!("Failed to resume task: {}", e))
}

pub fn complete_task(payload: TaskId, db: &Database) -> Result<Task, String> {
    let conn = db.get_connection();
    
    db::update_task_status(&conn, &payload.id, Status::Completed)
        .map_err(|e| format!("Failed to complete task: {}", e))
}

pub fn delete_task(payload: TaskId, db: &Database) -> Result<(), String> {
    let conn = db.get_connection();
    
    let deleted = db::delete_task_by_id(&conn, &payload.id)
        .map_err(|e| format!("Failed to delete task: {}", e))?;
    
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

pub fn update_task(payload: crate::structs::task_update::TaskUpdate, db: &Database) -> Result<Task, String> {
    use crate::structs::task_update::TaskUpdateParsed;
    
    let conn = db.get_connection();
    
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
    
    let update_data = TaskUpdateParsed {
        title: payload.data.title,
        notes,
        deadline,
        has_calendar_integration: payload.data.has_calendar_integration,
        calendar_email,
        reminder_frequency: payload.data.reminder_frequency,
        updated_at: chrono::Utc::now(),
    };
    
    db::update_task(&conn, &payload.id, &update_data)
        .map_err(|e| format!("Failed to update task: {}", e))
}
