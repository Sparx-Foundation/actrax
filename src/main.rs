use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

mod app_state;
mod core;
mod web;


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing!("ACTRAX_SERVER_LOG");

    info!("Starting ACTRAX_SERVER...");

    let app = Arc::new(app_state::AppState::default().await);
    
    println!("{}", app.config);

    web::web_main(app).await.expect("webserver startup failed");

    Ok(())
}
