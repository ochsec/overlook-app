//! License management module for Overlook.
//!
//! Handles:
//! - Trial period tracking (30 days from first launch)
//! - License key validation via license server
//! - Hardware fingerprinting for device binding
//! - Offline validation caching

mod client;
mod fingerprint;
mod storage;
mod types;

pub use client::LicenseClient;
pub use fingerprint::generate_fingerprint;
pub use storage::{load_license_data, save_license_data};
pub use types::{LicenseData, LicenseStatus, ValidationResult};

use chrono::Utc;

/// Number of days for the trial period
const TRIAL_DAYS: i64 = 30;

/// Check the current license status.
///
/// This function:
/// 1. Records first launch time if not set
/// 2. Checks if trial period has expired
/// 3. Validates license key if present (online or cached)
/// 4. Returns the appropriate status
pub fn check_license_status(data: &mut LicenseData) -> LicenseStatus {
    let now = Utc::now();

    // Record first launch if not set
    if data.first_launch.is_none() {
        data.first_launch = Some(now);
        data.hardware_id = Some(generate_fingerprint());
        if let Err(e) = save_license_data(data) {
            eprintln!("Failed to save license data: {}", e);
        }
    }

    // Calculate trial status
    let first_launch = data.first_launch.unwrap();
    let days_since_launch = (now - first_launch).num_days();
    let trial_expired = days_since_launch >= TRIAL_DAYS;

    // Check for valid license key
    if let Some(ref license_key) = data.license_key {
        // Use cached validation if available
        if let Some(ref cached) = data.validation_result {
            if cached.valid {
                return LicenseStatus::Valid {
                    email: cached.email.clone().unwrap_or_default(),
                };
            } else {
                return LicenseStatus::Invalid {
                    reason: cached.error.clone().unwrap_or_else(|| "Invalid license".to_string()),
                };
            }
        }
    }

    // No valid license - check trial status
    if trial_expired {
        LicenseStatus::TrialExpired
    } else {
        let days_remaining = (TRIAL_DAYS - days_since_launch) as u32;
        LicenseStatus::Trial { days_remaining }
    }
}

/// Validate license key with the server (async).
///
/// Updates the license data with the result and saves to disk.
pub async fn validate_license_online(data: &mut LicenseData) -> LicenseStatus {
    let license_key = match &data.license_key {
        Some(key) if !key.is_empty() => key.clone(),
        _ => return check_license_status(data),
    };

    let hardware_id = data.hardware_id.clone().unwrap_or_else(|| {
        let id = generate_fingerprint();
        data.hardware_id = Some(id.clone());
        id
    });

    let client = LicenseClient::new();
    match client.validate(&license_key, &hardware_id).await {
        Ok(result) => {
            // Update cached result
            data.last_validation = Some(Utc::now());
            data.validation_result = Some(result.clone());

            if let Err(e) = save_license_data(data) {
                eprintln!("Failed to save license data: {}", e);
            }

            if result.valid {
                LicenseStatus::Valid {
                    email: result.email.unwrap_or_default(),
                }
            } else {
                LicenseStatus::Invalid {
                    reason: result.error.unwrap_or_else(|| "Invalid license".to_string()),
                }
            }
        }
        Err(e) => {
            eprintln!("License validation failed: {}", e);
            // Fall back to cached result or trial status
            check_license_status(data)
        }
    }
}

/// Set the license key and trigger validation.
///
/// Returns the new status after attempting online validation.
pub async fn set_license_key(data: &mut LicenseData, key: String) -> LicenseStatus {
    data.license_key = Some(key);
    data.validation_result = None; // Clear cached result
    validate_license_online(data).await
}

/// Clear the license key.
pub fn clear_license_key(data: &mut LicenseData) -> LicenseStatus {
    data.license_key = None;
    data.validation_result = None;
    data.last_validation = None;

    if let Err(e) = save_license_data(data) {
        eprintln!("Failed to save license data: {}", e);
    }

    check_license_status(data)
}
