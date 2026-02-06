use crate::db::{self, Database};
use crate::structs::calendar::CalendarCredentials;
use crate::thirdparty::calendar;

pub async fn start_oauth_flow(db: &Database) -> Result<CalendarCredentials, String> {
    // Start OAuth flow and get credentials
    let credentials = calendar::start_oauth_flow().await?;
    
    // Save to database
    save_credentials(db, &credentials)?;
    
    Ok(credentials)
}

pub fn save_credentials(db: &Database, creds: &CalendarCredentials) -> Result<(), String> {
    let conn = db.get_connection();
    
    db::save_calendar_credentials(&conn, creds)
        .map_err(|e| format!("Failed to save credentials: {}", e))
}

pub fn get_credentials(db: &Database) -> Result<Option<CalendarCredentials>, String> {
    let conn = db.get_connection();
    
    db::get_calendar_credentials(&conn)
        .map_err(|e| format!("Failed to get credentials: {}", e))
}

pub fn disconnect_calendar(db: &Database) -> Result<(), String> {
    let conn = db.get_connection();
    
    db::clear_calendar_credentials(&conn)
        .map_err(|e| format!("Failed to disconnect calendar: {}", e))
}
