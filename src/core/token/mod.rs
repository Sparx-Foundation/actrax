use crate::core::client::database;
use crate::Claims;
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tracing::instrument;
use uuid::Uuid;

pub(crate) mod masked_token;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Token {
    pub db_pool: Arc<PgPool>,
    /// (Event, Data)
    pub sender: Sender<(String, String)>,
    pub secret: String,
}

impl Token {
    /// Creates a new `Token` instance with a given database pool and secret key.
    pub async fn new(db_pool: Arc<PgPool>, secret: String) -> Result<Self, sqlx::Error> {
        let (sender, _receiver) = broadcast::channel(10);

        initialize_schema(&db_pool).await?;

        database::initialize_schema(&db_pool).await?;

        Ok(Self {
            db_pool,
            sender,
            secret,
        })
    }

    /// Generates a session token with a short expiration time.
    pub fn generate_session_token(&self, sub: &i32) -> String {
        let expiration = SystemTime::now()
            .checked_add(Duration::from_secs(40 * 60))
            .expect("valid timestamp")
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("valid duration")
            .as_secs() as usize;

        let claims = Claims {
            exp: expiration,
            sub: *sub,
        };

       encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .unwrap()
        
    }

    /// Generates a refresh token for a user and stores it in the database.
    #[instrument(skip(self))]
    pub async fn generate_refresh_token(&self, client_id: &i32) -> Result<String, sqlx::Error> {
        let refresh_token = Uuid::new_v4().to_string();

        match sqlx::query("DELETE FROM refresh_tokens WHERE client_id = $1")
            .bind(client_id)
            .execute(&*self.db_pool)
            .await
        {
            Ok(_) => {
                tracing::debug!(
                    "Successfully deleted existing refresh tokens for client_id: {}",
                    client_id
                );
            }
            Err(err) => {
                tracing::error!(
                    "Failed to delete existing refresh tokens for client_id: {}. Error: {}",
                    client_id,
                    err
                );
            }
        };

        match sqlx::query("INSERT INTO refresh_tokens (client_id, token) VALUES ($1, $2)")
            .bind(client_id)
            .bind(&refresh_token)
            .execute(&*self.db_pool)
            .await
        {
            Ok(_) => {
                tracing::info!(
                    "Successfully inserted new refresh token for client_id: {}",
                    client_id
                );
            }
            Err(err) => {
                tracing::error!(
                    "Failed to insert new refresh token for client_id: {}. Error: {}",
                    client_id,
                    err
                );
                return Err(err);
            }
        };

        Ok(refresh_token)
    }

    /// Validates a given refresh token against the stored token in the database.
    #[instrument(skip(self))]
    pub async fn validate_refresh_token(&self, client_id: &i32, token: &str) -> bool {
        tracing::debug!(
            "Starting validation for client_id: {} with token: {}",
            client_id,
            token
        );

        let row = match sqlx::query("SELECT token FROM refresh_tokens WHERE client_id = $1")
            .bind(client_id)
            .fetch_optional(&*self.db_pool)
            .await
        {
            Ok(result) => {
                tracing::debug!("Query executed successfully for client_id: {}", client_id);
                result
            }
            Err(err) => {
                tracing::error!(
                    "Failed to fetch refresh token for client_id: {}. Error: {}",
                    client_id,
                    err
                );
                return false;
            }
        };

        if let Some(row) = row {
            let stored_token: String = match row.try_get("token") {
                Ok(token) => token,
                Err(err) => {
                    tracing::error!(
                        "Failed to extract token from query result for client_id: {}. Error: {}",
                        client_id,
                        err
                    );
                    return false;
                }
            };

            tracing::debug!("Retrieved token from database: {:?}", stored_token);

            if stored_token == token {
                tracing::info!("Token validated successfully for client_id: {}", client_id);
                true
            } else {
                tracing::warn!(
                    "Token mismatch for client_id: {}. Expected: {}, Provided: {}",
                    client_id,
                    stored_token,
                    token
                );
                false
            }
        } else {
            tracing::warn!("No token found for client_id: {}", client_id);
            false
        }
    }
}

pub async fn initialize_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    // TODO: fix this shit

    sqlx::query(include_str!(
        "../../../migrations/token/0001_token_table.sql"
    ))
    .execute(pool)
    .await
    .map_err(|e| {
        sqlx::Error::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to execute schema: {}", e),
        ))
    })?;

    Ok(())
}
