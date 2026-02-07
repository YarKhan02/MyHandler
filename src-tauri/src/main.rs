// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod error;
mod structs;
mod helpers;
mod services;
mod commands;
mod thirdparty;

use commands::{
  create_task, 
  get_tasks_by_date, 
  get_tasks_by_date_not_completed, 
  start_task, 
  pause_task, 
  resume_task, 
  complete_task, 
  delete_task, 
  get_task_by_id,
  update_task,
  get_settings,
  update_settings,
  start_calendar_auth,
  get_calendar_status,
  disconnect_calendar
};

fn main() {
  // Load environment variables from .env file
  dotenv::dotenv().ok();
  
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
    .invoke_handler(tauri::generate_handler![
      create_task, 
      get_tasks_by_date, 
      get_tasks_by_date_not_completed, 
      start_task, 
      pause_task, 
      resume_task, 
      complete_task, 
      delete_task, 
      get_task_by_id,
      update_task,
      get_settings,
      update_settings,
      start_calendar_auth,
      get_calendar_status,
      disconnect_calendar
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
