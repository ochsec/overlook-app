use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub listen_addr: String,
    pub paddle_webhook_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./licenses.db".to_string()),
            listen_addr: env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            paddle_webhook_secret: env::var("PADDLE_WEBHOOK_SECRET")
                .unwrap_or_else(|_| String::new()),
        }
    }
}
