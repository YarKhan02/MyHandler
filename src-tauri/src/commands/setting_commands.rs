use tauri::State;
use crate::db;
use crate::structs::settings::{Settings, SettingsUpdateData};
use crate::services::settings_service;

#[tauri::command]
pub fn get_settings(db: State<db::Database>) -> Result<Settings, String> {
  settings_service::get_settings(&db)
}

#[tauri::command]
pub fn update_settings(payload: SettingsUpdateData, db: State<db::Database>) -> Result<Settings, String> {
  settings_service::update_settings(&db, payload)
}