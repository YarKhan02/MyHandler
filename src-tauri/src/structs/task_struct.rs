use db_macros::{Insertable, Queryable};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::{Uuid, Timestamp};
use rusqlite::types::{ToSql, ToSqlOutput, FromSql, FromSqlResult, ValueRef};

use crate::db::Insertable;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReminderFrequency {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "hourly")]
    Hourly,
    #[serde(rename = "every-3-hours")]
    Every3Hours,
    #[serde(rename = "daily")]
    Daily,
}

impl Default for ReminderFrequency {
    fn default() -> Self {
        ReminderFrequency::None
    }
}

impl From<ReminderFrequency> for String {
    fn from(freq: ReminderFrequency) -> Self {
        match freq {
            ReminderFrequency::None => "none".to_string(),
            ReminderFrequency::Hourly => "hourly".to_string(),
            ReminderFrequency::Every3Hours => "every-3-hours".to_string(),
            ReminderFrequency::Daily => "daily".to_string(),
        }
    }
}

impl From<&str> for ReminderFrequency {
    fn from(s: &str) -> Self {
        match s {
            "hourly" => ReminderFrequency::Hourly,
            "every-3-hours" => ReminderFrequency::Every3Hours,
            "daily" => ReminderFrequency::Daily,
            _ => ReminderFrequency::None,
        }
    }
}

impl From<String> for ReminderFrequency {
    fn from(s: String) -> Self {
        ReminderFrequency::from(s.as_str())
    }
}

impl ToSql for ReminderFrequency {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let s = String::from(self.clone());
        Ok(ToSqlOutput::from(s))
    }
}

impl FromSql for ReminderFrequency {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(ReminderFrequency::from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    #[serde(rename = "not-started")]
    NotStarted,
    #[serde(rename = "ongoing")]
    Ongoing,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "completed")]
    Completed,
}

impl Default for Status {
    fn default() -> Self {
        Status::NotStarted
    }
}

impl From<Status> for String {
    fn from(status: Status) -> Self {
        match status {
            Status::NotStarted => "not-started".to_string(),
            Status::Ongoing => "ongoing".to_string(),
            Status::Paused => "paused".to_string(),
            Status::Completed => "completed".to_string(),
        }
    }
}

impl From<&str> for Status {
    fn from(s: &str) -> Self {
        match s {
            "not-started" => Status::NotStarted,
            "ongoing" => Status::Ongoing,
            "paused" => Status::Paused,
            "completed" => Status::Completed,
            _ => Status::NotStarted,
        }
    }
}

impl From<String> for Status {
    fn from(s: String) -> Self {
        Status::from(s.as_str())
    }
}

impl ToSql for Status {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let s = String::from(self.clone());
        Ok(ToSqlOutput::from(s))
    }
}

impl FromSql for Status {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(Status::from)
    }
}

#[derive(Debug, Insertable, Queryable, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[table_name = "tasks"]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub notes: Option<String>,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub has_calendar_integration: bool,
    pub calendar_email: Option<String>,
    pub reminder_frequency: ReminderFrequency,
    pub started_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(title: &str, created_at: DateTime<Utc>, notes: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v7(Timestamp::now(uuid::timestamp::context::NoContext)),
            title: title.to_string(),
            notes: notes.map(|s| s.to_string()),
            status: Status::default(),
            created_at,
            updated_at: created_at,
            deadline: None,
            has_calendar_integration: false,
            calendar_email: None,
            reminder_frequency: ReminderFrequency::default(),
            started_at: None,
            paused_at: None,
            completed_at: None,
        }
    }
}
