use crate::db::{self, Database};
use crate::structs::settings::{Settings, SettingsUpdateData};

pub fn get_settings(db: &Database) -> Result<Settings, String> {
    let conn = db.get_connection();
    
    db::get_settings(&conn)
        .map_err(|e| format!("Failed to fetch settings: {}", e))
}

pub fn update_settings(db: &Database, data: SettingsUpdateData) -> Result<Settings, String> {
    // Parse and validate the update data
    let parsed = data.parse()?;
    
    // Update settings in database
    let conn = db.get_connection();
    db::update_settings(&conn, &parsed)
        .map_err(|e| format!("Failed to update settings: {}", e))
}
