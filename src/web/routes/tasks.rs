use crate::app_state::AppState;
use crate::core::tasks::task::{OperationType, Task, TaskStatus};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use std::sync::Arc;

pub(crate) async fn get_tasks(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
) -> Result<(StatusCode, Json<Vec<Task>>), (StatusCode, String)> {
    match state.tasks.get_tasks_by_client_id(user_id).await {
        Ok(tasks) => Ok((StatusCode::OK, Json(tasks))),
        Err(e) => {
            tracing::error!("Failed to get tasks for this Client. Error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get Tasks".to_string(),
            ))
        }
    }
}

/// Struct for the incoming JSON payload to create a new task.
#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub client_id: i32,
    pub description: String,
    pub operation: OperationType,
}

/// The handler to add a task.
pub(crate) async fn add_task(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let new_task = Task {
        id: rand::random::<u64>(),
        description: payload.description,
        status: TaskStatus::Pending,
        client_id: payload.client_id,
        operation: payload.operation,
    };

    match state.tasks.add_task(new_task).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            tracing::error!("Failed to add task. Error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to add task".to_string(),
            ))
        }
    }
}
