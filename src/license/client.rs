//! HTTP client for license server communication.

use std::time::Duration;

use anyhow::Result;

use super::types::{ErrorResponse, ValidateRequest, ValidationResult};

/// Default license server URL (can be overridden via environment)
const DEFAULT_LICENSE_SERVER: &str = "https://overlook.chat";

/// HTTP client for license validation
pub struct LicenseClient {
    base_url: String,
    client: reqwest::Client,
}

impl LicenseClient {
    pub fn new() -> Self {
        let base_url = std::env::var("LICENSE_SERVER_URL")
            .unwrap_or_else(|_| DEFAULT_LICENSE_SERVER.to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { base_url, client }
    }

    /// Validate a license key with the server
    pub async fn validate(&self, license_key: &str, hardware_id: &str) -> Result<ValidationResult> {
        let url = format!("{}/api/v1/validate", self.base_url);

        let request = ValidateRequest {
            license_key: license_key.to_string(),
            hardware_id: hardware_id.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let result: ValidationResult = response.json().await?;
            Ok(result)
        } else {
            // Server returns error responses with {"error": "..."} body
            let error_response: ErrorResponse = response.json().await?;
            Ok(ValidationResult {
                valid: false,
                email: None,
                expires_at: None,
                days_remaining: None,
                error: Some(error_response.error),
            })
        }
    }

    /// Check if the license server is reachable
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/v1/health", self.base_url);
        self.client.get(&url).send().await.is_ok()
    }
}

impl Default for LicenseClient {
    fn default() -> Self {
        Self::new()
    }
}
