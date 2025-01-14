pub mod database;

use sqlx::{PgPool, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tracing::instrument;

#[derive(Clone)]
pub struct ClientData {
    pub client_id: String,
    pub client_secret: String,
    pub client_name: Option<String>,
}

impl std::fmt::Display for ClientData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // mask all but the first 4 characters of client_secret
        let masked_secret = if self.client_secret.len() > 4 {
            format!(
                "{}{}",
                &self.client_secret[..4],
                "*".repeat(self.client_secret.len() - 4)
            )
        } else {
            "*".repeat(self.client_secret.len())
        };

        let masked_display = if masked_secret.len() >= 6 {
            &masked_secret[..6]
        } else {
            &masked_secret
        };

        write!(
            f,
            "ClientID: {} | ClientSecret: {} {}",
            &self.client_id,
            masked_display,
            &self
                .client_name
                .as_ref()
                .map(|n| format!("| ClientName: {}", n))
                .unwrap_or_default(),
        )
    }
}

impl std::fmt::Debug for ClientData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientData")
            .field("client_id", &self.client_id)
            .field("client_name", &self.client_name)
            .field("client_secret", &self.client_secret)
            .finish()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Client {
    pub db_pool: Arc<PgPool>,
    /// (Event, Data)
    pub sender: Sender<(String, String)>,
}

impl Client {
    #[instrument]
    pub async fn new(db_pool: Arc<Pool<Postgres>>) -> Result<Self, sqlx::Error> {
        let db_pool_clone = db_pool.clone();

        let (sender, _receiver) = broadcast::channel(10);

        database::initialize_schema(&db_pool_clone).await?;

        Ok(Self {
            db_pool: db_pool_clone,
            sender,
        })
    }

    #[instrument]
    pub async fn create_client(&self, uid: &str, name: Option<&str>) -> Result<i32, sqlx::Error> {
        let user_id = database::create_client(&self.db_pool, uid, name).await?;

        let msg = format!("uid={}, name={:?}, id={}", uid, name, user_id);
        let _ = self.sender.send(("client_created".to_string(), msg));

        Ok(user_id)
    }

    #[instrument]
    pub async fn update_client(
        &self,
        id: i32,
        new_uid: Option<&str>,
        new_name: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        database::update_client(&self.db_pool, id, new_uid, new_name).await?;

        let msg = format!("id={}, new_uid={:?}, new_name={:?}", id, new_uid, new_name);
        let _ = self.sender.send(("client_updated".to_string(), msg));

        Ok(())
    }

    #[instrument]
    pub async fn delete_client(&self, id: i32) -> Result<(), sqlx::Error> {
        database::delete_client(&self.db_pool, id).await?;

        let msg = format!("id={}", id);
        let _ = self.sender.send(("client_deleted".to_string(), msg));

        Ok(())
    }
}
