use crate::core::client::Client;
use crate::core::log::Logging;
use crate::core::tasks::TaskManager;
use crate::core::token::masked_token::MaskedToken;
use crate::core::token::Token;
use serde::Deserialize;
use sqlx::PgPool;
use std::fmt;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) jwt: MaskedToken,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UserConfig {
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) server: ServerConfig,
    pub(crate) user: Vec<UserConfig>,
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) log: Arc<Logging>,
    pub(crate) user: Arc<Client>,
    pub(crate) token: Arc<Token>,
    pub(crate) tasks: Arc<TaskManager>,
    pub(crate) config: Arc<Config>,
    pub(crate) refresh_token_path: Arc<Mutex<String>>,
}

impl AppState {
    pub(crate) async fn default() -> Self {
        // todo! better method for setting (environment or config file)
        let database_url = "postgres://devuser:devpass@localhost/devdb";
        let pool = PgPool::connect(database_url)
            .await
            .expect("Can't connect to database");

        let db_pool = Arc::new(pool);

        let toml_data = fs::read_to_string("./config/default.toml")
            .await
            .expect("Failed to read config");

        let config: Config = toml::from_str(&toml_data).expect("Failed to parse config");

        let user = Client::new(db_pool.clone())
            .await
            .expect("Failed to initialize Logging");

        let token = Token::new(db_pool.clone(), config.server.jwt.to_string())
            .await
            .expect("Failed to create Token");

        let log = Logging::new(db_pool.clone())
            .await
            .expect("Failed to initialize Logging");

        let tasks = TaskManager::new().await;

        AppState {
            log: Arc::new(log),
            user: Arc::new(user),
            token: Arc::new(token),
            tasks: Arc::new(tasks),
            config: Arc::new(config),
            refresh_token_path: Arc::new(Default::default()),
        }
    }

    pub(crate) async fn jwt_secret_bytes(&self) -> Vec<u8> {
        let jwt_secret = self.config.server.jwt.as_ref();
        println!("jwt_secret: {}", jwt_secret);
        jwt_secret.as_bytes().to_vec()
    }

    /// (user_receiver, token_receiver, sender_receiver)
    pub(crate) async fn get_receivers(
        &self,
    ) -> (
        Receiver<(String, String)>,
        Receiver<(String, String)>,
        Receiver<(String, String)>,
    ) {
        let user_receiver = self.user.sender.subscribe();
        let token_receiver = self.token.sender.subscribe();
        let sender_receiver = self.log.sender.subscribe();

        (user_receiver, token_receiver, sender_receiver)
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Configuration:")?;
        writeln!(f, "{}", self.server)?;
        writeln!(f, "Users:")?;
        for (i, user) in self.user.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, user)?;
        }
        Ok(())
    }
}

impl fmt::Display for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Server:\n  Host: {}\n  Port: {}\n  JWT: {}",
            self.host, self.port, self.jwt
        )
    }
}

impl fmt::Display for UserConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {}, Password: [hidden]", self.name)
    }
}
