use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarCredentials {
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_expiry: DateTime<Utc>,
}
