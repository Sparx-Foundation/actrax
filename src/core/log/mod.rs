pub mod database;

use crate::core::log::database::{add_log_entry, initialize_schema, LogEntry};
use serde::Deserialize;
use sqlx::{Error as SqlxError, PgPool, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio::time::Instant;
use tracing::{error, info, instrument};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Deserialize)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
            LogLevel::CRITICAL => "CRITICAL",
        };

        write!(f, "{:^10}", text)
    }
}

impl std::str::FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Ok(LogLevel::DEBUG),
            "INFO" => Ok(LogLevel::INFO),
            "WARN" => Ok(LogLevel::WARN),
            "ERROR" => Ok(LogLevel::ERROR),
            "CRITICAL" => Ok(LogLevel::CRITICAL),
            _ => Err(()),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Logging {
    pub db_pool: Arc<PgPool>,
    /// (Event, Data)
    pub sender: Sender<(String, String)>,
}

impl Logging {
    #[instrument()]
    pub async fn new(db_pool: Arc<Pool<Postgres>>) -> Result<Self, sqlx::Error> {
        let db_pool_clone = db_pool.clone();

        let (sender, _receiver) = broadcast::channel(10);

        initialize_schema(&db_pool).await?;

        Ok(Self {
            db_pool: db_pool_clone,
            sender,
        })
    }

    #[instrument(skip(self, msg))]
    pub async fn log(
        &self,
        log_level: LogLevel,
        msg: &str,
        user_id: i32,
        user_name: &str,
    ) -> Result<String, String> {
        info!(
            "Logging message: {} | user_id: {} | user_name {}",
            log_level, user_id, user_name
        );

        let start = Instant::now();

        match add_log_entry(
            &self.db_pool,
            LogEntry {
                level: log_level.to_string(),
                message: msg.to_string(),
                user_id,
            },
        )
        .await
        {
            Ok(_) => {
                let duration = start.elapsed();

                info!(
                    "{} | {} | user_id: {:?} | Time taken: {:.2?}",
                    log_level, msg, user_id, duration
                );

                let event_msg = format!(
                    "log_level={}, msg={}, user_id={}, username={}",
                    log_level, msg, user_id, user_name
                );
                if let Err(e) = self
                    .sender
                    .send(("log_added".to_string(), event_msg.to_string()))
                {
                    error!("Failed to broadcast message: {}", e);
                }

                Ok("Log entry added successfully.".to_string())
            }
            Err(SqlxError::Database(e)) if e.code().unwrap() == "23503" => {
                // 23503 = ErrorKind::ForeignKeyViolation
                error!(
                    "Failed to add log entry: User with id {} does not exist. (ForeignKeyViolation)",
                    user_id
                );

                Err("User doesn't exist".to_string())
            }
            Err(e) => {
                let duration = start.elapsed();
                error!(
                    "Failed to add log entry: {} | user_id: {:?} | Time taken: {:.2?} | Error: {}",
                    msg, user_id, duration, e
                );
                Err(format!("Failed to add log entry: {}", e))
            }
        }
    }
}
