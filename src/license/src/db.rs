use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::models::License;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations (each statement separately for PostgreSQL)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS licenses (
            id SERIAL PRIMARY KEY,
            license_key TEXT NOT NULL,
            email TEXT NOT NULL,
            hardware_id TEXT,
            created_at TEXT NOT NULL,
            activated_at TEXT,
            revoked BOOLEAN DEFAULT FALSE,
            revoked_at TEXT,
            paddle_subscription_id TEXT,
            paddle_checkout_id TEXT
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_license_key ON licenses(license_key)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_hardware_id ON licenses(hardware_id)")
        .execute(&pool)
        .await?;

    // Drop old unique constraint on license_key if it exists (migration to multi-device)
    sqlx::query("ALTER TABLE licenses DROP CONSTRAINT IF EXISTS licenses_license_key_key")
        .execute(&pool)
        .await?;

    // Unique constraint on license_key + hardware_id (allows multiple devices per license)
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_license_hardware ON licenses(license_key, hardware_id) WHERE hardware_id IS NOT NULL")
        .execute(&pool)
        .await?;

    Ok(pool)
}

/// Find all license rows for a given license key
pub async fn find_licenses_by_key(pool: &PgPool, license_key: &str) -> Result<Vec<License>, sqlx::Error> {
    sqlx::query_as::<_, License>(
        "SELECT id, license_key, email, hardware_id, created_at, activated_at, revoked, revoked_at, paddle_subscription_id, paddle_checkout_id FROM licenses WHERE license_key = $1"
    )
    .bind(license_key)
    .fetch_all(pool)
    .await
}

/// Activate a license on a device. Updates existing unactivated row or inserts new row for second device.
pub async fn activate_license(pool: &PgPool, license: &License, hardware_id: &str) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    // Try to update an existing row that has no hardware_id
    let result = sqlx::query(
        "UPDATE licenses SET hardware_id = $1, activated_at = $2 WHERE license_key = $3 AND hardware_id IS NULL"
    )
    .bind(hardware_id)
    .bind(&now)
    .bind(&license.license_key)
    .execute(pool)
    .await?;

    // If no row was updated, insert a new activation row
    if result.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO licenses (license_key, email, hardware_id, created_at, activated_at, paddle_subscription_id, paddle_checkout_id) VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(&license.license_key)
        .bind(&license.email)
        .bind(hardware_id)
        .bind(&license.created_at)
        .bind(&now)
        .bind(&license.paddle_subscription_id)
        .bind(&license.paddle_checkout_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn create_license(
    pool: &PgPool,
    license_key: &str,
    email: &str,
    paddle_subscription_id: Option<&str>,
    paddle_checkout_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO licenses (license_key, email, created_at, paddle_subscription_id, paddle_checkout_id) VALUES ($1, $2, $3, $4, $5)"
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

pub async fn revoke_license_by_transaction(pool: &PgPool, transaction_id: &str) -> Result<u64, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE licenses SET revoked = TRUE, revoked_at = $1 WHERE paddle_subscription_id = $2 AND revoked = FALSE"
    )
    .bind(&now)
    .bind(transaction_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
