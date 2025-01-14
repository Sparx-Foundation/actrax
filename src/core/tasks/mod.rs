pub(crate) mod task;

use crate::core::tasks::task::{Task, TaskStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

/// A manager for handling tasks.
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<u64, Task>>>,
}
#[allow(dead_code)]
impl TaskManager {
    pub async fn new() -> Self {
        TaskManager {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, task: Task) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;

        if tasks.contains_key(&task.id) {
            return Err("Task with the given ID already exists".to_string());
        }

        tasks.insert(task.id, task);
        Ok(())
    }

    /// Helper method to retrieve and validate a task by its ID and expected status.
    async fn get_task_and_validate_status<F>(
        &self,
        task_id: u64,
        expected_status: TaskStatus,
        action: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&mut Task),
    {
        let mut tasks: MutexGuard<HashMap<u64, Task>> = self.tasks.lock().await;

        if let Some(task) = tasks.get_mut(&task_id) {
            if task.status == expected_status {
                action(task);
                Ok(())
            } else {
                Err(format!(
                    "Task is in an invalid state for this operation: {:?}",
                    task.status
                ))
            }
        } else {
            Err("Task not found".to_string())
        }
    }
    /// Marks a task as "working on."
    pub async fn mark_working_on(&self, task_id: u64) -> Result<(), String> {
        self.get_task_and_validate_status(task_id, TaskStatus::Pending, |task| {
            task.status = TaskStatus::WorkingOn;
        })
        .await
    }

    /// Marks a task as completed.
    pub async fn mark_completed(&self, task_id: u64) -> Result<(), String> {
        self.get_task_and_validate_status(task_id, TaskStatus::WorkingOn, |task| {
            task.status = TaskStatus::Completed;
        })
        .await
    }

    /// Marks a task as failed.
    pub async fn mark_failed(&self, task_id: u64) -> Result<(), String> {
        self.get_task_and_validate_status(task_id, TaskStatus::WorkingOn, |task| {
            task.status = TaskStatus::Failed;
        })
        .await
    }

    /// Retrieves all tasks associated with a specific client ID.
    /// Returns an error if no tasks are found for the given client ID.
    pub async fn get_tasks_by_client_id(&self, client_id: i32) -> Result<Vec<Task>, String> {
        let tasks = self.tasks.lock().await;
        let client_tasks: Vec<Task> = tasks
            .values()
            .filter(|task| task.client_id == client_id)
            .cloned()
            .collect();

        if client_tasks.is_empty() {
            Err(format!("No tasks found for client_id: {}", client_id))
        } else {
            Ok(client_tasks)
        }
    }

    /// Lists all tasks with their statuses.
    pub async fn list_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.tasks.lock().await;
        Ok(tasks.values().cloned().collect())
    }
}
