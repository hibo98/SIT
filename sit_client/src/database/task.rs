use crate::server::Server;

use super::{model::*, schema::*};
use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde_json::Value;
use sit_lib::task::{TaskStatus, TaskUpdate};

pub struct TaskManager {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl TaskManager {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>) -> TaskManager {
        TaskManager { pool }
    }

    pub fn add_new_task(&self, task: sit_lib::task::Task) -> Result<()> {
        let mut conn = self.pool.get()?;
        let t = Task {
            id: task.id,
            task: task.task.to_string(),
            time_start: task.time_start.map(|time| time.timestamp()),
            time_download: None,
            task_status: sit_lib::task::TaskStatus::Downloaded as i32,
            task_result: None,
        };
        diesel::insert_into(client_task::table)
            .values(t)
            .execute(&mut conn)?;
        let task_update = TaskUpdate {
            id: task.id,
            time_downloaded: Some(Utc::now()),
            task_status: TaskStatus::Downloaded,
            task_result: None,
        };
        Server::update_task(&task_update).unwrap();
        Ok(())
    }

    pub fn get_pending_tasks(&self) -> Result<Vec<sit_lib::task::Task>> {
        let mut conn = self.pool.get()?;
        let current_time = Utc::now();
        let tasks: Vec<Task> = client_task::table
            .filter(client_task::time_start.le(current_time.timestamp()))
            .filter(client_task::task_status.eq_any(vec![1]))
            .load(&mut conn)?;
        Ok(tasks
            .iter()
            .map(|t| sit_lib::task::Task {
                id: t.id,
                task: serde_json::from_str(&t.task).unwrap(),
                time_start: t
                    .time_start
                    .map(|time| DateTime::from_timestamp(time, 0).unwrap()),
            })
            .collect())
    }

    pub fn task_update_running(&self, task: &sit_lib::task::Task) {
        let task_update = TaskUpdate {
            id: task.id,
            time_downloaded: None,
            task_status: TaskStatus::Running,
            task_result: None,
        };
        Server::update_task(&task_update).unwrap();
    }

    pub fn task_update_failed(&self, task: &sit_lib::task::Task, task_result: Option<Value>) {
        let task_update = TaskUpdate {
            id: task.id,
            time_downloaded: None,
            task_status: TaskStatus::Failed,
            task_result,
        };
        Server::update_task(&task_update).unwrap();
    }

    pub fn task_update_successful(&self, task: &sit_lib::task::Task, task_result: Option<Value>) {
        let task_update = TaskUpdate {
            id: task.id,
            time_downloaded: None,
            task_status: TaskStatus::Successful,
            task_result,
        };
        Server::update_task(&task_update).unwrap();
    }
}
