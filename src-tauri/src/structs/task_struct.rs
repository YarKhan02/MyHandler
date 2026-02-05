use db_macros::Insertable;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Serialize, Deserialize};
use uuid::{Uuid, Timestamp};
use rusqlite::types::{ToSql, ToSqlOutput, FromSql, FromSqlResult, ValueRef};

use crate::db::Insertable;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    #[serde(rename = "not started")]
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
            Status::NotStarted => "not started".to_string(),
            Status::Ongoing => "ongoing".to_string(),
            Status::Paused => "paused".to_string(),
            Status::Completed => "completed".to_string(),
        }
    }
}

impl From<&str> for Status {
    fn from(s: &str) -> Self {
        match s {
            "not started" => Status::NotStarted,
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

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name = "tasks"]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub notes: Option<String>,
    pub status: Status,
    pub task_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(title: &str, task_date: NaiveDate, notes: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v7(Timestamp::now(uuid::timestamp::context::NoContext)),
            title: title.to_string(),
            notes: notes.map(|s| s.to_string()),
            status: Status::default(),
            task_date,
            created_at: Utc::now(),
            started_at: None,
            paused_at: None,
            completed_at: None,
        }
    }
}
