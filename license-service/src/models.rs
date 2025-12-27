use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct License {
    pub id: i32,
    pub license_key: String,
    pub email: String,
    pub hardware_id: Option<String>,
    pub created_at: String,
    pub activated_at: Option<String>,
    pub revoked: bool,
    pub revoked_at: Option<String>,
    pub paddle_subscription_id: Option<String>,
    pub paddle_checkout_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub license_key: String,
    pub hardware_id: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_remaining: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ValidateResponse {
    pub fn success(email: String, expires_at: String, days_remaining: i64) -> Self {
        Self {
            valid: true,
            email: Some(email),
            expires_at: Some(expires_at),
            days_remaining: Some(days_remaining),
            error: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Paddle webhook event types we care about
#[derive(Debug, Deserialize)]
pub enum PaddleEventType {
    #[serde(rename = "transaction.completed")]
    TransactionCompleted,
    #[serde(rename = "adjustment.created")]
    AdjustmentCreated,
    #[serde(other)]
    Other,
}

/// Paddle Billing webhook payload structure
#[derive(Debug, Deserialize)]
pub struct PaddleWebhook {
    pub event_type: PaddleEventType,
    pub data: PaddleEventData,
}

#[derive(Debug, Deserialize)]
pub struct PaddleEventData {
    pub id: String,
    #[serde(default)]
    pub customer: Option<PaddleCustomer>,
    /// For adjustment events
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub transaction_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaddleCustomer {
    pub email: Option<String>,
}
