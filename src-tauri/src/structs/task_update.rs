use serde::Deserialize;
use chrono::{DateTime, Utc};
use db_macros::Updatable;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdateData {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub deadline: Option<String>,
    pub has_calendar_integration: Option<bool>,
    pub calendar_email: Option<String>,
    pub reminder_frequency: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdate {
    pub id: String,
    pub data: TaskUpdateData,
}

// Parsed version with actual types for database operations
#[derive(Updatable)]
#[table_name = "tasks"]
pub struct TaskUpdateParsed {
    pub title: Option<String>,
    pub notes: Option<Option<String>>,
    pub deadline: Option<Option<DateTime<Utc>>>,
    pub has_calendar_integration: Option<bool>,
    pub calendar_email: Option<Option<String>>,
    pub reminder_frequency: Option<String>,
    pub updated_at: DateTime<Utc>,
}
