use super::{model::*, schema::*};
use anyhow::Result;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::Serialize;
use serde_json::{json, Value};
use sit_lib::task::TaskUpdate;

#[derive(Clone, Debug, Serialize)]
pub struct TaskOptions {
    pub name: String,
    pub parameters: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskResult {
    pub result: String,
}

pub struct TaskManager {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl TaskManager {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> TaskManager {
        TaskManager { pool }
    }

    pub fn delete_user_profile(&self, client_id: i32, sid_string: String) -> Result<()> {
        let mut conn = self.pool.get()?;
        let task_options = TaskOptions {
            name: "delete-user-profile".to_owned(),
            parameters: json!({"sid": sid_string}),
        };
        let task = NewTask {
            client_id,
            task: serde_json::to_value(task_options)?,
            time_start: None,
            time_download: None,
            task_status: Some(TaskStatus::Created),
            task_result: None,
        };
        diesel::insert_into(client_task::table)
            .values(task)
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn update_task_status(&self, client_id: i32, task_update: TaskUpdate) -> Result<()> {
        let mut conn = self.pool.get()?;

        if let Some(time_downloaded) = task_update.time_downloaded {
            diesel::update(client_task::table)
                .set(client_task::time_download.eq(time_downloaded.naive_utc()))
                .filter(client_task::client_id.eq(client_id))
                .execute(&mut conn)?;
        }
        if let Some(task_result) = task_update.task_result {
            diesel::update(client_task::table)
                .set(client_task::task_result.eq(task_result))
                .filter(client_task::client_id.eq(client_id))
                .execute(&mut conn)?;
        }
        diesel::update(client_task::table)
            .set(client_task::task_status.eq(Self::convert_task_status(task_update.task_status)))
            .filter(client_task::client_id.eq(client_id))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn get_new_tasks_for_client(&self, client_id: i32) -> Result<Vec<Task>> {
        let mut conn = self.pool.get()?;

        Ok(client_task::table
            .filter(client_task::client_id.eq(client_id))
            .filter(client_task::task_status.eq(TaskStatus::Created))
            .load(&mut conn)?)
    }

    fn convert_task_status(task_status: sit_lib::task::TaskStatus) -> TaskStatus {
        match task_status {
            sit_lib::task::TaskStatus::Created => TaskStatus::Created,
            sit_lib::task::TaskStatus::Downloaded => TaskStatus::Downloaded,
            sit_lib::task::TaskStatus::Running => TaskStatus::Running,
            sit_lib::task::TaskStatus::Successful => TaskStatus::Successful,
            sit_lib::task::TaskStatus::Failed => TaskStatus::Failed,
        }
    }
}
