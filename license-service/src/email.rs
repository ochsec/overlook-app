use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct ResendEmail {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
}

pub async fn send_license_email(
    api_key: &str,
    from: &str,
    to: &str,
    license_key: &str,
) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let email = ResendEmail {
        from: from.to_string(),
        to: vec![to.to_string()],
        subject: "Your Overlook License Key".to_string(),
        html: format!(
            r#"<div style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 40px 20px;">
    <h1 style="color: #1a1a1a; margin-bottom: 24px;">Thank you for purchasing Overlook!</h1>

    <p style="color: #4a4a4a; font-size: 16px; line-height: 1.6;">Your license key is:</p>

    <div style="background: #f5f5f5; border: 1px solid #e0e0e0; border-radius: 8px; padding: 20px; margin: 24px 0; text-align: center;">
        <code style="font-size: 20px; font-weight: bold; color: #1a1a1a; letter-spacing: 1px;">{}</code>
    </div>

    <h2 style="color: #1a1a1a; font-size: 18px; margin-top: 32px;">To activate:</h2>
    <ol style="color: #4a4a4a; font-size: 16px; line-height: 1.8;">
        <li>Open Overlook</li>
        <li>Go to Settings (or dismiss the trial expired dialog)</li>
        <li>Paste your license key in the "License Key" field</li>
        <li>Click "Activate"</li>
        <li>Restart Overlook</li>
    </ol>

    <p style="color: #666; font-size: 14px; margin-top: 32px;">Your license is valid for one year and can be used on up to 2 devices.</p>

    <p style="color: #666; font-size: 14px;">If you have any questions, reply to this email.</p>

    <p style="color: #4a4a4a; font-size: 16px; margin-top: 32px;">Best,<br>The Overlook Team</p>
</div>"#,
            license_key
        ),
    };

    client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&email)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
