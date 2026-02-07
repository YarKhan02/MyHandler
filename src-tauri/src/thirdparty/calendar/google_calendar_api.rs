use reqwest::Client;
use chrono::{DateTime, Utc};
use crate::structs::calendar_event::{CalendarEvent, EventDateTime, EventReminders, ReminderOverride, EventResponse};

pub async fn create_calendar_event(
    access_token: &str,
    title: &str,
    notes: Option<&str>,
    deadline: DateTime<Utc>,
    reminder_frequency: &str,
) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    
    // Create reminder list based on frequency
    let mut reminders = Vec::new();
    
    // Always add email reminder
    reminders.push(ReminderOverride {
        method: "email".to_string(),
        minutes: 60, // 1 hour before
    });
    
    // Add popup reminders based on frequency
    match reminder_frequency {
        "hourly" => {
            reminders.push(ReminderOverride {
                method: "popup".to_string(),
                minutes: 60,
            });
        }
        "every-3-hours" => {
            reminders.push(ReminderOverride {
                method: "popup".to_string(),
                minutes: 180,
            });
        }
        "daily" => {
            reminders.push(ReminderOverride {
                method: "popup".to_string(),
                minutes: 1440, // 24 hours
            });
        }
        _ => {} // "none" - just email
    }
    
    // Create event with deadline as end time
    let event = CalendarEvent {
        summary: title.to_string(),
        description: notes.map(|s| s.to_string()),
        start: EventDateTime {
            date_time: deadline.to_rfc3339(),
            time_zone: "UTC".to_string(),
        },
        end: EventDateTime {
            date_time: (deadline + chrono::Duration::hours(1)).to_rfc3339(),
            time_zone: "UTC".to_string(),
        },
        reminders: EventReminders {
            use_default: false,
            overrides: reminders,
        },
    };
    
    let response = client
        .post("https://www.googleapis.com/calendar/v3/calendars/primary/events")
        .bearer_auth(access_token)
        .json(&event)
        .send()
        .await
        .map_err(|e| format!("Failed to create calendar event: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("Failed to create event: {} - {}", status, error_body));
    }
    
    let event_response: EventResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse event response: {}", e))?;
    
    Ok(event_response.id)
}

pub async fn update_calendar_event(
    access_token: &str,
    event_id: &str,
    title: &str,
    notes: Option<&str>,
    deadline: DateTime<Utc>,
    reminder_frequency: &str,
) -> Result<(), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    
    // Create reminder list based on frequency
    let mut reminders = Vec::new();
    
    // Always add email reminder
    reminders.push(ReminderOverride {
        method: "email".to_string(),
        minutes: 60,
    });
    
    // Add popup reminders based on frequency
    match reminder_frequency {
        "hourly" => {
            reminders.push(ReminderOverride {
                method: "popup".to_string(),
                minutes: 60,
            });
        }
        "every-3-hours" => {
            reminders.push(ReminderOverride {
                method: "popup".to_string(),
                minutes: 180,
            });
        }
        "daily" => {
            reminders.push(ReminderOverride {
                method: "popup".to_string(),
                minutes: 1440,
            });
        }
        _ => {}
    }
    
    let event = CalendarEvent {
        summary: title.to_string(),
        description: notes.map(|s| s.to_string()),
        start: EventDateTime {
            date_time: deadline.to_rfc3339(),
            time_zone: "UTC".to_string(),
        },
        end: EventDateTime {
            date_time: (deadline + chrono::Duration::hours(1)).to_rfc3339(),
            time_zone: "UTC".to_string(),
        },
        reminders: EventReminders {
            use_default: false,
            overrides: reminders,
        },
    };
    
    let response = client
        .patch(&format!(
            "https://www.googleapis.com/calendar/v3/calendars/primary/events/{}",
            event_id
        ))
        .bearer_auth(access_token)
        .json(&event)
        .send()
        .await
        .map_err(|e| format!("Failed to update calendar event: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("Failed to update event: {} - {}", status, error_body));
    }
    
    Ok(())
}

pub async fn delete_calendar_event(
    access_token: &str,
    event_id: &str,
) -> Result<(), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    
    let response = client
        .delete(&format!(
            "https://www.googleapis.com/calendar/v3/calendars/primary/events/{}",
            event_id
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to delete calendar event: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("Failed to delete event: {} - {}", status, error_body));
    }
    
    Ok(())
}
