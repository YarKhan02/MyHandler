// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod error;
mod structs;
mod helpers;

use tauri::State;
use serde::Deserialize;
use chrono::Utc;

use crate::db::insert;
use crate::structs::task_struct::Task;
use crate::helpers::parse_date::parse_date_range;

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
  let (start_of_day, end_of_day) = parse_date_range(&payload.date)?;
  
  let sql = include_str!("../db/sql/get_tasks_by_date.sql");
  let conn = db.get_connection();
  let tasks = db::query_tasks_by_date_range(&conn, start_of_day, end_of_day, sql)
    .map_err(|e| format!("Failed to query tasks: {}", e))?;
  
  Ok(tasks)
}

#[tauri::command]
fn get_tasks_by_date_not_completed(payload: DateQuery, db: State<db::Database>) -> Result<Vec<Task>, String> {
  let (start_of_day, end_of_day) = parse_date_range(&payload.date)?;
  
  let sql = include_str!("../db/sql/get_tasks_by_date_not_completed.sql");
  let conn = db.get_connection();
  let tasks = db::query_tasks_by_date_range(&conn, start_of_day, end_of_day, sql)
    .map_err(|e| format!("Failed to query tasks: {}", e))?;
  
  Ok(tasks)
}

#[tauri::command]
fn start_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  let conn = db.get_connection();
  
  db::update_task_status(&conn, &payload.id, structs::task_struct::Status::Ongoing)
    .map_err(|e| format!("Failed to start task: {}", e))
}

#[tauri::command]
fn pause_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  let conn = db.get_connection();
  
  db::update_task_status(&conn, &payload.id, structs::task_struct::Status::Paused)
    .map_err(|e| format!("Failed to pause task: {}", e))
}

#[tauri::command]
fn resume_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  let conn = db.get_connection();
  
  db::update_task_status(&conn, &payload.id, structs::task_struct::Status::Ongoing)
    .map_err(|e| format!("Failed to resume task: {}", e))
}

#[tauri::command]
fn complete_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  let conn = db.get_connection();
  
  db::update_task_status(&conn, &payload.id, structs::task_struct::Status::Completed)
    .map_err(|e| format!("Failed to complete task: {}", e))
}

#[tauri::command]
fn delete_task(payload: TaskId, db: State<db::Database>) -> Result<(), String> {
  let conn = db.get_connection();
  
  let deleted = db::delete_task_by_id(&conn, &payload.id)
    .map_err(|e| format!("Failed to delete task: {}", e))?;
  
  if deleted == 0 {
    return Err("Task not found".to_string());
  }
  
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
    .invoke_handler(tauri::generate_handler![create_task, get_tasks_by_date, start_task, pause_task, resume_task, complete_task, delete_task])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
