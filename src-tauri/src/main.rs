// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod error;
mod structs;
mod helpers;
mod services;

use tauri::State;
use crate::structs::dto::{TaskData, DateQuery, TaskId};
use crate::structs::task_update::TaskUpdate;
use crate::structs::task_struct::Task;
use crate::services::task_service;

#[tauri::command]
fn create_task(payload: TaskData, db: State<db::Database>) -> Result<Task, String> {
  task_service::create_task(payload, &db)
}

#[tauri::command]
fn get_tasks_by_date(payload: DateQuery, db: State<db::Database>) -> Result<Vec<Task>, String> {
  task_service::get_tasks_by_date(payload, &db)
}

#[tauri::command]
fn get_tasks_by_date_not_completed(payload: DateQuery, db: State<db::Database>) -> Result<Vec<Task>, String> {
  task_service::get_tasks_by_date_not_completed(payload, &db)
}

#[tauri::command]
fn start_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::start_task(payload, &db)
}

#[tauri::command]
fn pause_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::pause_task(payload, &db)
}

#[tauri::command]
fn resume_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::resume_task(payload, &db)
}

#[tauri::command]
fn complete_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::complete_task(payload, &db)
}

#[tauri::command]
fn delete_task(payload: TaskId, db: State<db::Database>) -> Result<(), String> {
  task_service::delete_task(payload, &db)
}

#[tauri::command]
fn get_task_by_id(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::get_task_by_id(payload, &db)
}

#[tauri::command]
fn update_task(payload: TaskUpdate, db: State<db::Database>) -> Result<Task, String> {
  task_service::update_task(payload, &db)
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
      update_task
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
