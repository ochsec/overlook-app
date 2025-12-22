use axum::{extract::State, Json};
use chrono::{DateTime, Months, Utc};

use crate::{
    db,
    error::AppError,
    models::{ValidateRequest, ValidateResponse},
    AppState,
};

const MAX_DEVICES: usize = 2;
const LICENSE_VALIDITY_MONTHS: u32 = 12;

pub async fn validate_handler(
    State(state): State<AppState>,
    Json(request): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, AppError> {
    let pool = &state.pool;

    // Find all license rows for this key
    let licenses = db::find_licenses_by_key(pool, &request.license_key).await?;

    if licenses.is_empty() {
        return Err(AppError::LicenseNotFound);
    }

    // Check if any row is revoked (if one is revoked, the license is revoked)
    if licenses.iter().any(|l| l.revoked) {
        return Err(AppError::LicenseRevoked);
    }

    // Calculate expiration from created_at
    let license = &licenses[0];
    let created_at = DateTime::parse_from_rfc3339(&license.created_at)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid created_at: {}", e)))?
        .with_timezone(&Utc);
    let expires_at = created_at
        .checked_add_months(Months::new(LICENSE_VALIDITY_MONTHS))
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Date overflow")))?;
    let now = Utc::now();

    if now > expires_at {
        return Err(AppError::LicenseExpired);
    }

    let days_remaining = (expires_at - now).num_days();
    let expires_at_str = expires_at.to_rfc3339();

    // Check if this hardware_id is already activated
    if licenses.iter().any(|l| l.hardware_id.as_deref() == Some(&request.hardware_id)) {
        return Ok(Json(ValidateResponse::success(
            license.email.clone(),
            expires_at_str,
            days_remaining,
        )));
    }

    // Count activated devices (rows with non-null hardware_id)
    let activated_count = licenses.iter().filter(|l| l.hardware_id.is_some()).count();

    if activated_count >= MAX_DEVICES {
        return Err(AppError::MaxDevicesReached);
    }

    // Activate on this device
    db::activate_license(pool, license, &request.hardware_id).await?;

    tracing::info!(
        license_key = %request.license_key,
        hardware_id = %request.hardware_id,
        device_number = activated_count + 1,
        "License activated on new device"
    );

    Ok(Json(ValidateResponse::success(
        license.email.clone(),
        expires_at_str,
        days_remaining,
    )))
}
