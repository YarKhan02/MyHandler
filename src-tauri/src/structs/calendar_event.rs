use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct CalendarEvent {
    pub summary: String,
    pub description: Option<String>,
    pub start: EventDateTime,
    pub end: EventDateTime,
    pub reminders: EventReminders,
}

#[derive(Serialize)]
pub struct EventDateTime {
    #[serde(rename = "dateTime")]
    pub date_time: String,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
}

#[derive(Serialize)]
pub struct EventReminders {
    #[serde(rename = "useDefault")]
    pub use_default: bool,
    pub overrides: Vec<ReminderOverride>,
}

#[derive(Serialize)]
pub struct ReminderOverride {
    pub method: String,
    pub minutes: i32,
}

#[derive(Deserialize)]
pub struct EventResponse {
    pub id: String,
}
