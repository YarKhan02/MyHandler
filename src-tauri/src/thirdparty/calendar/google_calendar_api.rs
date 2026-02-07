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
    
    // Only add reminders if reminder_frequency is not empty (empty = paused/completed)
    if !reminder_frequency.is_empty() {
        // Calculate time until deadline
        let now = Utc::now();
        let duration_until_deadline = deadline.signed_duration_since(now);
        let hours_until_deadline = duration_until_deadline.num_hours();
        
        // Add popup reminders from now until deadline based on frequency
        match reminder_frequency {
            "hourly" => {
                // Add reminders every hour from now until deadline
                let max_popup_reminders = 4; // Reserve 1 slot for email reminder (Google limit is 5 total)
                let reminder_count = std::cmp::min(hours_until_deadline.max(0) as usize, max_popup_reminders);
                
                for i in 0..reminder_count {
                    let minutes_before = ((i as i64 + 1) * 60) as i32; // 1h, 2h, 3h, etc. before deadline
                    reminders.push(ReminderOverride {
                        method: "popup".to_string(),
                        minutes: minutes_before,
                    });
                }
            }
            "every-3-hours" => {
                // Add reminders every 3 hours from now until deadline
                let max_popup_reminders = 4;
                let reminder_count = std::cmp::min((hours_until_deadline / 3).max(0) as usize, max_popup_reminders);
                
                for i in 0..reminder_count {
                    let minutes_before = ((i as i64 + 1) * 180) as i32; // 3h, 6h, 9h, etc. before deadline
                    reminders.push(ReminderOverride {
                        method: "popup".to_string(),
                        minutes: minutes_before,
                    });
                }
            }
            "daily" => {
                // Add reminders every day from now until deadline
                let days_until_deadline = duration_until_deadline.num_days();
                let max_popup_reminders = 4;
                let reminder_count = std::cmp::min(days_until_deadline.max(0) as usize, max_popup_reminders);
                
                for i in 0..reminder_count {
                    let minutes_before = ((i as i64 + 1) * 1440) as i32; // 1 day, 2 days, etc. before deadline
                    reminders.push(ReminderOverride {
                        method: "popup".to_string(),
                        minutes: minutes_before,
                    });
                }
            }
            _ => {} // "none"
        }
        
        // Always add email reminder 1 hour before deadline (even if no popup reminders)
        reminders.push(ReminderOverride {
            method: "email".to_string(),
            minutes: 60,
        });
    }
    
    // Create event that ends at deadline (not extends beyond it)
    let event = CalendarEvent {
        summary: title.to_string(),
        description: notes.map(|s| s.to_string()),
        start: EventDateTime {
            date_time: (deadline - chrono::Duration::hours(1)).to_rfc3339(),
            time_zone: "UTC".to_string(),
        },
        end: EventDateTime {
            date_time: deadline.to_rfc3339(),
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
    
    // Only add reminders if reminder_frequency is not empty (empty = paused/completed)
    if !reminder_frequency.is_empty() {
        // Calculate time until deadline
        let now = Utc::now();
        let duration_until_deadline = deadline.signed_duration_since(now);
        let hours_until_deadline = duration_until_deadline.num_hours();
        
        // Add popup reminders from now until deadline based on frequency
        match reminder_frequency {
            "hourly" => {
                // Add reminders every hour from now until deadline
                let max_popup_reminders = 4; // Reserve 1 slot for email reminder (Google limit is 5 total)
                let reminder_count = std::cmp::min(hours_until_deadline.max(0) as usize, max_popup_reminders);
                
                for i in 0..reminder_count {
                    let minutes_before = ((i as i64 + 1) * 60) as i32; // 1h, 2h, 3h, etc. before deadline
                    reminders.push(ReminderOverride {
                        method: "popup".to_string(),
                        minutes: minutes_before,
                    });
                }
            }
            "every-3-hours" => {
                // Add reminders every 3 hours from now until deadline
                let max_popup_reminders = 4;
                let reminder_count = std::cmp::min((hours_until_deadline / 3).max(0) as usize, max_popup_reminders);
                
                for i in 0..reminder_count {
                    let minutes_before = ((i as i64 + 1) * 180) as i32; // 3h, 6h, 9h, etc. before deadline
                    reminders.push(ReminderOverride {
                        method: "popup".to_string(),
                        minutes: minutes_before,
                    });
                }
            }
            "daily" => {
                // Add reminders every day from now until deadline
                let days_until_deadline = duration_until_deadline.num_days();
                let max_popup_reminders = 4;
                let reminder_count = std::cmp::min(days_until_deadline.max(0) as usize, max_popup_reminders);
                
                for i in 0..reminder_count {
                    let minutes_before = ((i as i64 + 1) * 1440) as i32; // 1 day, 2 days, etc. before deadline
                    reminders.push(ReminderOverride {
                        method: "popup".to_string(),
                        minutes: minutes_before,
                    });
                }
            }
            _ => {} // "none"
        }
        
        // Always add email reminder 1 hour before deadline (even if no popup reminders)
        reminders.push(ReminderOverride {
            method: "email".to_string(),
            minutes: 60,
        });
    }
    
    let event = CalendarEvent {
        summary: title.to_string(),
        description: notes.map(|s| s.to_string()),
        start: EventDateTime {
            date_time: (deadline - chrono::Duration::hours(1)).to_rfc3339(),
            time_zone: "UTC".to_string(),
        },
        end: EventDateTime {
            date_time: deadline.to_rfc3339(),
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
    
    let status = response.status();
    
    // 404 (Not Found) or 410 (Gone) means event was deleted externally
    if status.as_u16() == 404 || status.as_u16() == 410 {
        println!("Calendar event {} not found - may have been deleted externally", event_id);
        return Err("EVENT_NOT_FOUND".to_string());
    }
    
    if !status.is_success() {
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
    
    let status = response.status();
    
    // 404 (Not Found) or 410 (Gone) means event already deleted - this is OK
    if status.as_u16() == 404 || status.as_u16() == 410 {
        println!("Calendar event {} already deleted or not found", event_id);
        return Ok(());
    }
    
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("Failed to delete event: {} - {}", status, error_body));
    }
    
    Ok(())
}
