use axum::{extract::State, Json};

use crate::{
    db,
    error::AppError,
    models::{ValidateRequest, ValidateResponse},
    AppState,
};

pub async fn validate_handler(
    State(state): State<AppState>,
    Json(request): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, AppError> {
    let pool = &state.pool;
    // Find license by key
    let license = db::find_license_by_key(pool, &request.license_key)
        .await?
        .ok_or(AppError::LicenseNotFound)?;

    // Check if revoked
    if license.revoked {
        return Err(AppError::LicenseRevoked);
    }

    // Check hardware binding
    match &license.hardware_id {
        None => {
            // First activation - bind to this device
            db::activate_license(&pool, &request.license_key, &request.hardware_id).await?;
            tracing::info!(
                license_key = %request.license_key,
                hardware_id = %request.hardware_id,
                "License activated on new device"
            );
            Ok(Json(ValidateResponse::success(license.email)))
        }
        Some(bound_hardware_id) if bound_hardware_id == &request.hardware_id => {
            // Same device - valid
            Ok(Json(ValidateResponse::success(license.email)))
        }
        Some(_) => {
            // Different device - reject
            Err(AppError::AlreadyActivated)
        }
    }
}
