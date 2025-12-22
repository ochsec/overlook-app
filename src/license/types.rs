use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of the license check
#[derive(Debug, Clone, PartialEq)]
pub enum LicenseStatus {
    /// Still in trial period
    Trial { days_remaining: u32 },
    /// Trial has expired, no valid license
    TrialExpired,
    /// License is valid
    Valid { email: String },
    /// License validation failed
    Invalid { reason: String },
}

impl LicenseStatus {
    pub fn is_valid(&self) -> bool {
        matches!(self, LicenseStatus::Trial { .. } | LicenseStatus::Valid { .. })
    }

    pub fn is_expired(&self) -> bool {
        matches!(self, LicenseStatus::TrialExpired | LicenseStatus::Invalid { .. })
    }
}

/// Locally stored license data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    /// When the app was first launched (trial start)
    pub first_launch: Option<DateTime<Utc>>,
    /// User-entered license key (if any)
    pub license_key: Option<String>,
    /// Hardware fingerprint for this device
    pub hardware_id: Option<String>,
    /// Timestamp of last successful server validation
    pub last_validation: Option<DateTime<Utc>>,
    /// Cached validation result from server
    pub validation_result: Option<ValidationResult>,
}

impl Default for LicenseData {
    fn default() -> Self {
        Self {
            first_launch: None,
            license_key: None,
            hardware_id: None,
            last_validation: None,
            validation_result: None,
        }
    }
}

/// Response from license server validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub email: Option<String>,
    pub expires_at: Option<String>,
    pub days_remaining: Option<i64>,
    pub error: Option<String>,
}

/// Error response from license server
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Request to license server
#[derive(Debug, Serialize)]
pub struct ValidateRequest {
    pub license_key: String,
    pub hardware_id: String,
}
