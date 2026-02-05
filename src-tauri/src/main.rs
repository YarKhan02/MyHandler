// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod error;
mod structs;

use tauri::State;
use serde::Deserialize;
use chrono::NaiveDate;

use crate::db::insert;
use crate::structs::task_struct::Task;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskData {
  title: String,
  task_date: String,
}

#[tauri::command]
fn create_task(payload: TaskData, db: State<db::Database>) -> Result<(), String> {
  println!("Received task: {} on {}", payload.title, payload.task_date);
  
  // Parse the date string to NaiveDate
  let task_date = NaiveDate::parse_from_str(&payload.task_date, "%Y-%m-%d")
    .map_err(|e| format!("Invalid date format: {}", e))?;
  
  // Use the global database connection
  let conn = db.get_connection();

  let task = Task::new(&payload.title, task_date, None);
  insert(&conn, &task).map_err(|e| format!("Failed to insert task: {}", e))?;
  println!("Inserted task: {:?}", task);

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
    .invoke_handler(tauri::generate_handler![create_task])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
