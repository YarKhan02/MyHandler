use chrono::{DateTime, Utc};
use db_macros::{Queryable, Updatable};
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};
use rusqlite::Result as RusqliteResult;
use serde::{Deserialize, Serialize};

// ReminderFrequency enum for settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ReminderFrequency {
    None,
    Hourly,
    #[serde(rename = "every-3-hours")]
    Every3Hours,
    Daily,
}

impl ToSql for ReminderFrequency {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput<'_>> {
        let s = match self {
            ReminderFrequency::None => "none",
            ReminderFrequency::Hourly => "hourly",
            ReminderFrequency::Every3Hours => "every-3-hours",
            ReminderFrequency::Daily => "daily",
        };
        Ok(ToSqlOutput::from(s))
    }
}

impl FromSql for ReminderFrequency {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        String::column_result(value).and_then(|s| match s.as_str() {
            "none" => Ok(ReminderFrequency::None),
            "hourly" => Ok(ReminderFrequency::Hourly),
            "every-3-hours" => Ok(ReminderFrequency::Every3Hours),
            "daily" => Ok(ReminderFrequency::Daily),
            _ => Err(FromSqlError::InvalidType),
        })
    }
}

// Settings struct
#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub id: i32,
    pub dark_mode: bool,
    pub notifications_enabled: bool,
    pub default_reminder_frequency: ReminderFrequency,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTO for updating settings from frontend
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsUpdateData {
    pub dark_mode: Option<bool>,
    pub notifications_enabled: Option<bool>,
    pub default_reminder_frequency: Option<String>,
}

// Parsed update data with Updatable derive
#[derive(Debug, Updatable)]
#[table_name = "settings"]
pub struct SettingsUpdateParsed {
    pub dark_mode: Option<bool>,
    pub notifications_enabled: Option<bool>,
    pub default_reminder_frequency: Option<ReminderFrequency>,
}

impl SettingsUpdateData {
    pub fn parse(self) -> Result<SettingsUpdateParsed, String> {
        let default_reminder_frequency = match self.default_reminder_frequency {
            Some(freq_str) => {
                let freq = match freq_str.as_str() {
                    "none" => ReminderFrequency::None,
                    "hourly" => ReminderFrequency::Hourly,
                    "every-3-hours" => ReminderFrequency::Every3Hours,
                    "daily" => ReminderFrequency::Daily,
                    _ => return Err(format!("Invalid reminder frequency: {}", freq_str)),
                };
                Some(freq)
            }
            None => None,
        };

        Ok(SettingsUpdateParsed {
            dark_mode: self.dark_mode,
            notifications_enabled: self.notifications_enabled,
            default_reminder_frequency,
        })
    }
}
