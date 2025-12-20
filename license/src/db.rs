use chrono::Utc;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::models::License;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::query(include_str!("../migrations/001_create_licenses.sql"))
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn find_license_by_key(pool: &SqlitePool, license_key: &str) -> Result<Option<License>, sqlx::Error> {
    sqlx::query_as::<_, License>(
        "SELECT id, license_key, email, hardware_id, created_at, activated_at, revoked, revoked_at, paddle_subscription_id, paddle_checkout_id FROM licenses WHERE license_key = ?"
    )
    .bind(license_key)
    .fetch_optional(pool)
    .await
}

pub async fn activate_license(pool: &SqlitePool, license_key: &str, hardware_id: &str) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE licenses SET hardware_id = ?, activated_at = ? WHERE license_key = ?")
        .bind(hardware_id)
        .bind(&now)
        .bind(license_key)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_license(
    pool: &SqlitePool,
    license_key: &str,
    email: &str,
    paddle_subscription_id: Option<&str>,
    paddle_checkout_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO licenses (license_key, email, created_at, paddle_subscription_id, paddle_checkout_id) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(license_key)
    .bind(email)
    .bind(&now)
    .bind(paddle_subscription_id)
    .bind(paddle_checkout_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn revoke_license(pool: &SqlitePool, paddle_subscription_id: &str) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE licenses SET revoked = 1, revoked_at = ? WHERE paddle_subscription_id = ?")
        .bind(&now)
        .bind(paddle_subscription_id)
        .execute(pool)
        .await?;
    Ok(())
}
