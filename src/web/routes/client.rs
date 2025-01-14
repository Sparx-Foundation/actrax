use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use crate::app_state::AppState;

#[derive(Deserialize)]
pub(crate) struct CreateClientRequest {
    name: String,
    uid: String,
}

pub(crate) async fn create_client_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateClientRequest>,
) -> impl IntoResponse {
    match state
        .user
        .create_client(&payload.uid, Some(&payload.name))
        .await
    {
        Ok(client_id) => match state.token.generate_refresh_token(&client_id).await {
            Ok(token) => (
                StatusCode::OK,
                Json(
                    json!({"client_id": client_id.to_string(), "refresh_token":token.to_string() }),
                ),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize)]
pub struct GetTokenRequest {
    refresh_token: String,
    client_id: String,
}

pub(crate) async fn get_token_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GetTokenRequest>,
) -> impl IntoResponse {
    let is_valid = state
        .token
        .validate_refresh_token(&payload.client_id.parse().unwrap(), &payload.refresh_token)
        .await;
    if !is_valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "invalid_refresh_token",
                "message": "The provided refresh token is invalid."
            })),
        )
            .into_response();
    }

    let new_access_token = match state
        .token
        .generate_refresh_token(&payload.client_id.parse().unwrap())
        .await
    {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "invalid_token"})),
            )
                .into_response()
        }
    };
    let new_session_token = state
        .token
        .generate_session_token(&payload.client_id.parse().unwrap());

    (
        StatusCode::OK,
        Json(json!({"refresh_token": new_access_token, "session_token": new_session_token})),
    )
        .into_response()
}
