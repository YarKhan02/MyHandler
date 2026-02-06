use tauri::State;
use crate::db;
use crate::structs::dto::{TaskData, DateQuery, TaskId};
use crate::structs::task_update::TaskUpdate;
use crate::structs::task_struct::Task;
use crate::services::task_service;

#[tauri::command]
pub fn create_task(payload: TaskData, db: State<db::Database>) -> Result<Task, String> {
  task_service::create_task(payload, &db)
}

#[tauri::command]
pub fn get_tasks_by_date(payload: DateQuery, db: State<db::Database>) -> Result<Vec<Task>, String> {
  task_service::get_tasks_by_date(payload, &db)
}

#[tauri::command]
pub fn get_tasks_by_date_not_completed(payload: DateQuery, db: State<db::Database>) -> Result<Vec<Task>, String> {
  task_service::get_tasks_by_date_not_completed(payload, &db)
}

#[tauri::command]
pub fn start_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::start_task(payload, &db)
}

#[tauri::command]
pub fn pause_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::pause_task(payload, &db)
}

#[tauri::command]
pub fn resume_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::resume_task(payload, &db)
}

#[tauri::command]
pub fn complete_task(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::complete_task(payload, &db)
}

#[tauri::command]
pub fn delete_task(payload: TaskId, db: State<db::Database>) -> Result<(), String> {
  task_service::delete_task(payload, &db)
}

#[tauri::command]
pub fn get_task_by_id(payload: TaskId, db: State<db::Database>) -> Result<Task, String> {
  task_service::get_task_by_id(payload, &db)
}

#[tauri::command]
pub fn update_task(payload: TaskUpdate, db: State<db::Database>) -> Result<Task, String> {
  task_service::update_task(payload, &db)
}