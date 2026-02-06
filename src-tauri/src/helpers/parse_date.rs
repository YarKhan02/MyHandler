use chrono::{DateTime, Utc};

/// Parse ISO 8601 datetime string and return start and end of day timestamps
pub fn parse_date_range(date_str: &str) -> Result<(DateTime<Utc>, DateTime<Utc>), String> {
    // Parse ISO 8601 datetime string and get the date
    let date_time = date_str.parse::<DateTime<Utc>>()
        .map_err(|e| format!("Invalid datetime format: {}", e))?;
    
    // Get date at start of day (00:00:00) and end of day (23:59:59)
    let start_of_day = date_time.date_naive().and_hms_opt(0, 0, 0)
        .ok_or("Failed to create start of day")?
        .and_utc();
    let end_of_day = date_time.date_naive().and_hms_opt(23, 59, 59)
        .ok_or("Failed to create end of day")?
        .and_utc();
    
    Ok((start_of_day, end_of_day))
}
