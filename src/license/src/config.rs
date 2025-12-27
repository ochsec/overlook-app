use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub listen_addr: String,
    pub paddle_webhook_secret: String,
    pub resend_api_key: String,
    pub resend_from_email: String,
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
            resend_api_key: env::var("RESEND_API_KEY")
                .unwrap_or_else(|_| String::new()),
            resend_from_email: env::var("RESEND_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@overlook.app".to_string()),
        }
    }
}
