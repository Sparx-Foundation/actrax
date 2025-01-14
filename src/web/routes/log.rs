use crate::core::log::LogLevel;

use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::Deserialize;
use std::sync::Arc;
use crate::app_state::AppState;

#[derive(Deserialize)]
pub(crate) struct LogRequest {
    log_level: LogLevel,
    message: String,
    user_name: String,
}

pub(crate) async fn log_handler(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Json(payload): Json<LogRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state
        .log
        .log(
            payload.log_level,
            &payload.message,
            user_id,
            &payload.user_name,
        )
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Failed to log message. Error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to log message".to_string()))
        }
    }
}

