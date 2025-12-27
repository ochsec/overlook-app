use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{db, email::send_license_email, AppState};

#[derive(Deserialize)]
pub struct ResendEmailQuery {
    license_key: String,
}

#[derive(Serialize)]
pub struct ResendEmailResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
pub struct ResendFailedEmailsResponse {
    total_processed: usize,
    successful: usize,
    failed: usize,
    errors: Vec<String>,
}

/// Manually resend a license email by license key
/// GET /api/v1/resend-email?license_key=XXXX-XXXX-XXXX-XXXX
pub async fn resend_email_handler(
    State(state): State<AppState>,
    Query(params): Query<ResendEmailQuery>,
) -> (StatusCode, Json<ResendEmailResponse>) {
    let pool = &state.pool;
    let config = &state.config;

    // Check if email is configured
    if config.resend_api_key.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ResendEmailResponse {
                success: false,
                message: "Email service not configured (RESEND_API_KEY missing)".to_string(),
            }),
        );
    }

    // Find the license
    let licenses = match db::find_licenses_by_key(pool, &params.license_key).await {
        Ok(licenses) => licenses,
        Err(e) => {
            tracing::error!("Database error finding license: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResendEmailResponse {
                    success: false,
                    message: format!("Database error: {}", e),
                }),
            );
        }
    };

    if licenses.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(ResendEmailResponse {
                success: false,
                message: "License key not found".to_string(),
            }),
        );
    }

    let license = &licenses[0];

    // Send the email
    match send_license_email(
        &config.resend_api_key,
        &config.resend_from_email,
        &license.email,
        &license.license_key,
    )
    .await
    {
        Ok(_) => {
            tracing::info!(
                license_key = %license.license_key,
                email = %license.email,
                "License email resent successfully"
            );

            // Mark as sent in database
            if let Err(e) = db::mark_email_sent(pool, &license.license_key).await {
                tracing::error!("Failed to mark email as sent: {}", e);
            }

            (
                StatusCode::OK,
                Json(ResendEmailResponse {
                    success: true,
                    message: format!("Email sent successfully to {}", license.email),
                }),
            )
        }
        Err(e) => {
            tracing::error!("Failed to send email: {}", e);

            // Mark the failure
            if let Err(db_err) = db::mark_email_failed(pool, &license.license_key, &e.to_string()).await {
                tracing::error!("Failed to mark email failure: {}", db_err);
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResendEmailResponse {
                    success: false,
                    message: format!("Failed to send email: {}", e),
                }),
            )
        }
    }
}

/// Batch resend all failed emails
/// POST /api/v1/resend-failed-emails
pub async fn resend_failed_emails_handler(
    State(state): State<AppState>,
) -> (StatusCode, Json<ResendFailedEmailsResponse>) {
    let pool = &state.pool;
    let config = &state.config;

    // Check if email is configured
    if config.resend_api_key.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ResendFailedEmailsResponse {
                total_processed: 0,
                successful: 0,
                failed: 0,
                errors: vec!["Email service not configured (RESEND_API_KEY missing)".to_string()],
            }),
        );
    }

    // Find all licenses with failed emails
    let failed_licenses = match db::find_licenses_with_failed_emails(pool).await {
        Ok(licenses) => licenses,
        Err(e) => {
            tracing::error!("Database error finding failed licenses: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResendFailedEmailsResponse {
                    total_processed: 0,
                    successful: 0,
                    failed: 0,
                    errors: vec![format!("Database error: {}", e)],
                }),
            );
        }
    };

    let total = failed_licenses.len();
    let mut successful = 0;
    let mut errors = Vec::new();

    tracing::info!("Attempting to resend {} failed license emails", total);

    for license in failed_licenses {
        match send_license_email(
            &config.resend_api_key,
            &config.resend_from_email,
            &license.email,
            &license.license_key,
        )
        .await
        {
            Ok(_) => {
                tracing::info!(
                    license_key = %license.license_key,
                    email = %license.email,
                    "License email resent successfully"
                );

                if let Err(e) = db::mark_email_sent(pool, &license.license_key).await {
                    tracing::error!("Failed to mark email as sent: {}", e);
                }

                successful += 1;
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to send email for license {} ({}): {}",
                    license.license_key, license.email, e
                );
                tracing::error!("{}", error_msg);
                errors.push(error_msg);

                if let Err(db_err) = db::mark_email_failed(pool, &license.license_key, &e.to_string()).await {
                    tracing::error!("Failed to mark email failure: {}", db_err);
                }
            }
        }

        // Add a small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let failed = total - successful;

    tracing::info!(
        "Batch email resend completed: {} total, {} successful, {} failed",
        total,
        successful,
        failed
    );

    (
        StatusCode::OK,
        Json(ResendFailedEmailsResponse {
            total_processed: total,
            successful,
            failed,
            errors,
        }),
    )
}
