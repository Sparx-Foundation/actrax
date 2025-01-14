pub(crate) mod client;
pub(crate) mod feeds;
pub(crate) mod log;
pub(crate) mod tasks;

use crate::{Claims};
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::{header, HeaderMap, Request};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Json;
use jsonwebtoken::{decode, errors::ErrorKind, DecodingKey, Validation};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, trace};
use crate::app_state::AppState;

pub(crate) async fn client_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer "));

    let refresh_token_path = state.refresh_token_path.lock().await.clone();

    match token {
        Some(token) => {
            let validation = Validation::default();
            let key = DecodingKey::from_secret(&state.jwt_secret_bytes().await);
            match decode::<Claims>(token, &key, &validation) {
                Ok(decoded) => {
                    req.extensions_mut().insert(decoded.claims.sub);

                    trace!("finsihed middlewastre continue to handler");
                    next.run(req).await
                }
                Err(err) => match *err.kind() {
                    ErrorKind::ExpiredSignature => {
                        let mut headers = HeaderMap::new();
                        let err_msg = &format!(
                                "Bearer error=\"invalid_token\", error_description=\"The access token expired\", refresh_url=\"{}\"",
                                refresh_token_path
                            );

                        error!(err_msg);
                        headers.insert(
                            header::WWW_AUTHENTICATE,
                            header::HeaderValue::from_str(err_msg).unwrap(),
                        );

                        (
                            StatusCode::UNAUTHORIZED,
                            headers,
                            Json(json!({
                                "error": "token_expired",
                                "message": "Your token has expired. Please refresh the token.",
                                "refresh_url": refresh_token_path
                            })),
                        )
                            .into_response()
                    }
                    _ => {
                        error!("The provided token is invalid.");

                        let body = Json(json!({
                            "error": "invalid_token",
                            "message": "The provided token is invalid."
                        }));
                        (StatusCode::UNAUTHORIZED, body).into_response()
                    }
                },
            }
        }
        None => {
            error!("Authorization token is missing");

            let body = Json(json!({
                "error": "missing_token",
                "message": "Authorization token is missing."
            }));
            (StatusCode::UNAUTHORIZED, body).into_response()
        }
    }
}
