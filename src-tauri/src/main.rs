// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod error;
mod structs;

use tauri::State;
use serde::Deserialize;
use chrono::Utc;

use crate::db::insert;
use crate::structs::task_struct::Task;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskData {
  title: String,
  created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DateQuery {
  date: String,
}

#[derive(Deserialize)]
struct TaskId {
  id: String,
}

#[tauri::command]
fn create_task(payload: TaskData, db: State<db::Database>) -> Result<Task, String> {
  
  // Parse ISO 8601 datetime string
  let created_at = payload.created_at.parse::<chrono::DateTime<Utc>>()
    .map_err(|e| format!("Invalid datetime format: {}", e))?;
  
  // Use the global database connection
  let conn = db.get_connection();

  let task = Task::new(&payload.title, created_at, None);
  insert(&conn, &task).map_err(|e| format!("Failed to insert task: {}", e))?;
  
  Ok(task)
}

#[tauri::command]
fn get_tasks_by_date(payload: DateQuery, db: State<db::Database>) -> Result<Vec<Task>, String> {

  // Parse ISO 8601 datetime string and get the date
  let date_time = payload.date.parse::<chrono::DateTime<Utc>>()
    .map_err(|e| format!("Invalid datetime format: {}", e))?;
  
  // Get date at start of day (00:00:00) and end of day (23:59:59)
  let start_of_day = date_time.date_naive().and_hms_opt(0, 0, 0)
    .ok_or("Failed to create start of day")?
    .and_utc();
  let end_of_day = date_time.date_naive().and_hms_opt(23, 59, 59)
    .ok_or("Failed to create end of day")?
    .and_utc();
  
  let conn = db.get_connection();
  let tasks = db::query_tasks_by_date_range(&conn, start_of_day, end_of_day)
    .map_err(|e| format!("Failed to query tasks: {}", e))?;
  
  Ok(tasks)
}

#[tauri::command]
fn delete_task(payload: TaskId, db: State<db::Database>) -> Result<(), String> {
  let conn = db.get_connection();
  db::delete_task_by_id(&conn, &payload.id)
    .map_err(|e| format!("Failed to delete task: {}", e))?;
  Ok(())
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      match db::init_db(&app.handle()) {
        Ok(_) => {
          println!("Database initialized successfully");
          Ok(())
        }
        Err(e) => {
          eprintln!("Failed to initialize database: {}", e);
          eprintln!("App will continue but database features may not work");
          // Don't crash the app, just log the error
          Ok(())
        }
      }
    })
    .invoke_handler(tauri::generate_handler![create_task, get_tasks_by_date, delete_task])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
