mod routes;

use std::sync::Arc;
use std::time::Duration;

use crate::app_state::AppState;
use crate::web::routes::client_middleware;
use axum::http;
use axum::routing::{get, post};
use axum::{middleware, Router};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

pub async fn web_main(core: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        &core.config.server.host, &core.config.server.port
    ))
    .await
    .unwrap();

    tracing::info!("Server Listening on:  {}", &core.config.server.port);

    let origins = AllowOrigin::any();
    let app = configure_routes(origins, core);

    axum::serve(listener, app)
        .await
        .expect("Failed to run Axum server");

    Ok(())
}

pub fn configure_routes<T>(origins: T, state: Arc<AppState>) -> Router
where
    T: Into<AllowOrigin>,
{
    Router::new()
        .route("/client", post(routes::client::create_client_handler))
        .route("/token", get(routes::client::get_token_handler))
        .route(
            "/log",
            post(routes::log::log_handler).layer(middleware::from_fn_with_state(
                state.clone(),
                client_middleware,
            )),
        )
        .route(
            "/tasks",
            get(routes::tasks::get_tasks).layer(middleware::from_fn_with_state(
                state.clone(),
                client_middleware,
            )),
        )
        .route("/tasks", post(routes::tasks::add_task))
        .route("/feed/all", get(routes::feeds::all))
        .route("/feed/log", get(routes::feeds::log))
        .with_state(state)
        .layer(TimeoutLayer::new(Duration::from_secs(90))) // abort request after 90sec
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_headers([http::header::AUTHORIZATION])
                .allow_methods([http::Method::GET, http::Method::POST, http::Method::PUT]),
        )
        .layer(TraceLayer::new_for_http())
}
