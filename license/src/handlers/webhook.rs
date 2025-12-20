use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::{
    db,
    models::{PaddleEventType, PaddleWebhook},
    paddle::verify_paddle_signature,
    AppState,
};

pub async fn paddle_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let pool = &state.pool;
    let config = &state.config;
    // Get signature header
    let signature = match headers.get("paddle-signature") {
        Some(sig) => match sig.to_str() {
            Ok(s) => s,
            Err(_) => {
                tracing::warn!("Invalid paddle-signature header encoding");
                return StatusCode::BAD_REQUEST;
            }
        },
        None => {
            tracing::warn!("Missing paddle-signature header");
            return StatusCode::BAD_REQUEST;
        }
    };

    // Verify signature
    if !config.paddle_webhook_secret.is_empty()
        && !verify_paddle_signature(signature, &body, &config.paddle_webhook_secret)
    {
        tracing::warn!("Invalid Paddle webhook signature");
        return StatusCode::UNAUTHORIZED;
    }

    // Parse webhook payload
    let webhook: PaddleWebhook = match serde_json::from_slice(&body) {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("Failed to parse webhook payload: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    tracing::info!(event_type = ?webhook.event_type, "Received Paddle webhook");

    match webhook.event_type {
        PaddleEventType::TransactionCompleted | PaddleEventType::SubscriptionCreated => {
            // Extract license key from custom_data
            let license_key = webhook
                .data
                .custom_data
                .as_ref()
                .and_then(|c| c.license_key.clone());

            let email = webhook
                .data
                .customer
                .as_ref()
                .and_then(|c| c.email.clone())
                .unwrap_or_else(|| "unknown@example.com".to_string());

            let license_key = match license_key {
                Some(key) => key,
                None => {
                    tracing::warn!("No license_key in custom_data");
                    return StatusCode::BAD_REQUEST;
                }
            };

            // Create license
            match db::create_license(
                &pool,
                &license_key,
                &email,
                Some(&webhook.data.id),
                None,
            )
            .await
            {
                Ok(_) => {
                    tracing::info!(
                        license_key = %license_key,
                        email = %email,
                        "Created new license"
                    );
                    StatusCode::OK
                }
                Err(e) => {
                    // Could be duplicate key - that's ok for idempotency
                    if e.to_string().contains("UNIQUE constraint failed") {
                        tracing::info!(license_key = %license_key, "License already exists");
                        StatusCode::OK
                    } else {
                        tracing::error!("Failed to create license: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                }
            }
        }
        PaddleEventType::SubscriptionCanceled => {
            // Revoke the license
            match db::revoke_license(&pool, &webhook.data.id).await {
                Ok(_) => {
                    tracing::info!(
                        subscription_id = %webhook.data.id,
                        "Revoked license"
                    );
                    StatusCode::OK
                }
                Err(e) => {
                    tracing::error!("Failed to revoke license: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
        PaddleEventType::Other => {
            // Ignore other events
            tracing::debug!("Ignoring unhandled event type");
            StatusCode::OK
        }
    }
}
