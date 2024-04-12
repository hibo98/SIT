use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskBundle {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: i32,
    pub task: Value,
    pub time_start: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub id: i32,
    pub time_downloaded: Option<DateTime<Utc>>,
    pub task_status: TaskStatus,
    pub task_result: Option<Value>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Created,
    Downloaded,
    Running,
    Successful,
    Failed,
}
