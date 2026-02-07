use crate::db::{self, Database};
use crate::structs::calendar::CalendarCredentials;
use crate::thirdparty::calendar;
use chrono::{DateTime, Utc, Duration};

pub async fn start_oauth_flow(db: &Database) -> Result<CalendarCredentials, String> {
    // Start OAuth flow and get credentials
    let credentials = calendar::start_oauth_flow().await?;
    
    // Save to database
    save_credentials(db, &credentials)?;
    
    Ok(credentials)
}

pub fn save_credentials(db: &Database, creds: &CalendarCredentials) -> Result<(), String> {
    println!("save_credentials: Starting...");
    let conn = db.get_connection();
    println!("save_credentials: Got connection, saving...");
    
    let result = db::save_calendar_credentials(&conn, creds)
        .map_err(|e| format!("Failed to save credentials: {}", e));
    
    println!("save_credentials: Save completed");
    result
}

pub fn get_credentials(db: &Database) -> Result<Option<CalendarCredentials>, String> {
    println!("get_credentials: Getting DB connection...");
    let conn = db.get_connection();
    println!("get_credentials: Got connection, querying credentials...");
    
    let result = db::get_calendar_credentials(&conn)
        .map_err(|e| format!("Failed to get credentials: {}", e));
    
    println!("get_credentials: Query completed");
    result
}

pub fn disconnect_calendar(db: &Database) -> Result<(), String> {
    let conn = db.get_connection();
    
    db::clear_calendar_credentials(&conn)
        .map_err(|e| format!("Failed to disconnect calendar: {}", e))
}

// Get valid access token, refreshing if needed
pub async fn get_valid_access_token(db: &Database) -> Result<String, String> {
    println!("get_valid_access_token: Starting...");
    println!("get_valid_access_token: Calling get_credentials...");
    
    let mut creds = get_credentials(db)?
        .ok_or_else(|| "No calendar credentials found".to_string())?;
    
    println!("get_valid_access_token: Credentials loaded successfully");
    
    println!("Credentials loaded, checking expiry...");
    
    // Check if token needs refresh (5 minute buffer)
    let now = Utc::now();
    let buffer = Duration::minutes(5);
    
    if creds.token_expiry - buffer < now {
        println!("Token expired, refreshing...");
        // Token expired or about to expire, refresh it
        let (new_access_token, expires_in) = 
            calendar::refresh_access_token(&creds.refresh_token).await?;
        println!("Token refresh completed");
        
        // Update credentials
        creds.access_token = new_access_token.clone();
        creds.token_expiry = Utc::now() + Duration::seconds(expires_in);
        
        // Save updated credentials
        save_credentials(db, &creds)?;
        
        Ok(new_access_token)
    } else {
        println!("Token still valid, using existing one");
        Ok(creds.access_token)
    }
}

// Create calendar event for a task
pub async fn create_task_calendar_event(
    db: &Database,
    title: &str,
    notes: Option<&str>,
    deadline: DateTime<Utc>,
    reminder_frequency: &str,
) -> Result<String, String> {
    println!("Getting access token for calendar...");
    let access_token = get_valid_access_token(db).await?;
    println!("Access token obtained, creating event...");
    
    let result = calendar::create_calendar_event(
        &access_token,
        title,
        notes,
        deadline,
        reminder_frequency,
    ).await;
    
    match &result {
        Ok(event_id) => println!("Successfully created calendar event: {}", event_id),
        Err(e) => eprintln!("Failed to create calendar event: {}", e),
    }
    
    result
}

// Update calendar event
pub async fn update_task_calendar_event(
    db: &Database,
    event_id: &str,
    title: &str,
    notes: Option<&str>,
    deadline: DateTime<Utc>,
    reminder_frequency: &str,
) -> Result<(), String> {
    println!("Updating calendar event: {}", event_id);
    let access_token = get_valid_access_token(db).await?;
    
    let result = calendar::update_calendar_event(
        &access_token,
        event_id,
        title,
        notes,
        deadline,
        reminder_frequency,
    ).await;
    
    match &result {
        Ok(_) => println!("Successfully updated calendar event"),
        Err(e) => eprintln!("Failed to update calendar event: {}", e),
    }
    
    result
}

// Delete calendar event
pub async fn delete_task_calendar_event(
    db: &Database,
    event_id: &str,
) -> Result<(), String> {
    let access_token = get_valid_access_token(db).await?;
    
    calendar::delete_calendar_event(&access_token, event_id).await
}
