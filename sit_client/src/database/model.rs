use diesel::prelude::*;
use serde::Serialize;

use crate::database::schema::*;

#[derive(Clone, Debug, Queryable, Serialize, Insertable)]
#[diesel(table_name = client_task)]
pub struct Task {
    pub id: i32,
    pub task: String,
    pub time_start: Option<i64>,
    pub time_download: Option<i64>,
    pub task_status: i32,
    pub task_result: Option<String>,
}
