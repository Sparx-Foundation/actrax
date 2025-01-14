pub(crate) mod log;
pub(crate) mod client;
pub(crate) mod token;
pub(crate) mod tasks;

#[macro_export]
macro_rules! init_tracing {
    ($env_var:expr) => {{
        use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

        let env_filter = if let Ok(env) = std::env::var($env_var) {
            EnvFilter::new(env)
        } else if cfg!(debug_assertions) {
            EnvFilter::new(tracing::Level::TRACE.to_string())
        } else {
            EnvFilter::new(tracing::Level::INFO.to_string())
        };

        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(env_filter)
            .init();

        if cfg!(debug_assertions) {
            tracing::error!("This is an error message");
            tracing::warn!("This is a warning message");
            tracing::info!("This is an info message");
            tracing::debug!("This is a debug message");
            tracing::trace!("This is a trace message");
        }
    }};
}
