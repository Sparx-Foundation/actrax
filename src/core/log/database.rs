use sqlx::{Error, PgPool};
use std::time::Duration;
use tokio::time::sleep;

async fn execute_embedded_schema(pool: &PgPool, schema_file: &str) -> Result<(), sqlx::Error> {
    let schema_sql = match schema_file {
        "init" => include_str!("../../../migrations/log/0001_log_table.sql"),
        _ => {
            return Err(sqlx::Error::Configuration(
                format!("Unknown schema file '{}'", schema_file).into(),
            ))
        }
    };

    sqlx::query(schema_sql).execute(pool).await.map_err(|e| {
        sqlx::Error::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to execute schema '{}': {}", schema_file, e),
        ))
    })?;

    Ok(())
}

pub async fn initialize_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    let schema_files = ["init"];

    for file in schema_files {
        execute_embedded_schema(pool, file).await?;
    }

    Ok(())
}

pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub user_id: i32,
}

pub async fn add_log_entry(pool: &PgPool, entry: LogEntry) -> Result<(), sqlx::Error> {
    let mut retries = 5;
    let mut wait_time = Duration::from_millis(200);

    while retries > 0 {
        let query = sqlx::query(
            r#"
            INSERT INTO actrax_logs (level, message, client_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(&entry.level)
        .bind(&entry.message)
        .bind(&entry.user_id);

        match query.execute(pool).await {
            Ok(_) => return Ok(()),
            Err(Error::PoolTimedOut) => {
                tracing::error!("Connection Pool timed out. Retrying...");
                sleep(wait_time).await;
                retries -= 1;
                wait_time *= 2;
            }
            Err(e) => {
                return Err(sqlx::Error::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to insert log entry: {}", e),
                )));
            }
        }
    }

    Err(sqlx::Error::PoolTimedOut)
}
