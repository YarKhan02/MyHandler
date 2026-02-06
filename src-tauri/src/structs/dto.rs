use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskData {
    pub title: String,
    pub created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateQuery {
    pub date: String,
}

#[derive(Deserialize)]
pub struct TaskId {
    pub id: String,
}
