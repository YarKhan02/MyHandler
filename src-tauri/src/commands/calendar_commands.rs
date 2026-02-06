use tauri::State;
use crate::db;
use crate::services::calendar_service;
use crate::structs::calendar::CalendarCredentials;

#[tauri::command]
pub async fn start_calendar_auth(db: State<'_, db::Database>) -> Result<CalendarCredentials, String> {
    calendar_service::start_oauth_flow(&db).await
}

#[tauri::command]
pub fn get_calendar_status(db: State<'_, db::Database>) -> Result<Option<CalendarCredentials>, String> {
    calendar_service::get_credentials(&db)
}

#[tauri::command]
pub fn disconnect_calendar(db: State<'_, db::Database>) -> Result<(), String> {
    calendar_service::disconnect_calendar(&db)
}
