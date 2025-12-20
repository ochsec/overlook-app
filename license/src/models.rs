use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct License {
    pub id: i64,
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
    pub error: Option<String>,
}

impl ValidateResponse {
    pub fn success(email: String) -> Self {
        Self {
            valid: true,
            email: Some(email),
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
#[serde(rename_all = "snake_case")]
pub enum PaddleEventType {
    TransactionCompleted,
    SubscriptionCreated,
    SubscriptionCanceled,
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
    pub custom_data: Option<PaddleCustomData>,
    #[serde(default)]
    pub customer: Option<PaddleCustomer>,
}

#[derive(Debug, Deserialize)]
pub struct PaddleCustomData {
    pub license_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaddleCustomer {
    pub email: Option<String>,
}
