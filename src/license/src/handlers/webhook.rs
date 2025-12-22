use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::{
    db,
    models::{PaddleEventType, PaddleWebhook},
    paddle::verify_paddle_signature,
    AppState,
};

fn generate_license_key() -> String {
    let uuid = Uuid::new_v4();
    let hex = uuid.simple().to_string().to_uppercase();
    format!(
        "overlook-key-{}-{}-{}-{}",
        &hex[0..4],
        &hex[4..8],
        &hex[8..12],
        &hex[12..16]
    )
}

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
        PaddleEventType::TransactionCompleted => {
            let email = webhook
                .data
                .customer
                .as_ref()
                .and_then(|c| c.email.clone())
                .unwrap_or_else(|| "unknown@example.com".to_string());

            let license_key = generate_license_key();
            tracing::info!(license_key = %license_key, "Generated new license key");

            match db::create_license(
                pool,
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
                    if e.to_string().contains("duplicate key") {
                        tracing::info!(transaction_id = %webhook.data.id, "License already exists for transaction");
                        StatusCode::OK
                    } else {
                        tracing::error!("Failed to create license: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                }
            }
        }
        PaddleEventType::AdjustmentCreated => {
            let action = webhook.data.action.as_deref().unwrap_or("");
            let transaction_id = match &webhook.data.transaction_id {
                Some(id) => id,
                None => {
                    tracing::warn!("Adjustment missing transaction_id");
                    return StatusCode::BAD_REQUEST;
                }
            };

            // Revoke license on refund or chargeback
            if action == "refund" || action == "chargeback" {
                match db::revoke_license_by_transaction(pool, transaction_id).await {
                    Ok(count) => {
                        tracing::info!(
                            transaction_id = %transaction_id,
                            action = %action,
                            revoked_count = count,
                            "Revoked license due to {}", action
                        );
                        StatusCode::OK
                    }
                    Err(e) => {
                        tracing::error!("Failed to revoke license: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                }
            } else {
                tracing::debug!(action = %action, "Ignoring non-refund adjustment");
                StatusCode::OK
            }
        }
        PaddleEventType::Other => {
            tracing::debug!("Ignoring unhandled event type");
            StatusCode::OK
        }
    }
}
