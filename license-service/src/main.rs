mod config;
mod db;
mod error;
mod handlers;
mod models;
mod paddle;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "overlook_license_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();
    tracing::info!("Starting license server on {}", config.listen_addr);

    // Create database pool
    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Connected to database: {}", config.database_url);

    let listen_addr = config.listen_addr.clone();
    let state = AppState { pool, config };

    // Build router
    let app = Router::new()
        .route("/api/v1/health", get(handlers::health_handler))
        .route("/api/v1/validate", post(handlers::validate_handler))
        .route(
            "/api/v1/webhook/paddle",
            post(handlers::paddle_webhook_handler),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("License server ready");
    axum::serve(listener, app).await?;

    Ok(())
}
