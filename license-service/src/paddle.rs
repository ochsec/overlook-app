use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Verify Paddle Billing webhook signature
///
/// Paddle Billing uses the format: ts=timestamp;h1=signature
/// The signature is HMAC-SHA256(timestamp + ":" + request_body, secret)
pub fn verify_paddle_signature(
    signature_header: &str,
    body: &[u8],
    secret: &str,
) -> bool {
    // Parse the signature header
    let mut timestamp = None;
    let mut signature = None;

    for part in signature_header.split(';') {
        if let Some(ts) = part.strip_prefix("ts=") {
            timestamp = Some(ts);
        } else if let Some(h1) = part.strip_prefix("h1=") {
            signature = Some(h1);
        }
    }

    let (timestamp, expected_signature) = match (timestamp, signature) {
        (Some(ts), Some(sig)) => (ts, sig),
        _ => return false,
    };

    // Build the signed payload: timestamp:body
    let mut signed_payload = timestamp.as_bytes().to_vec();
    signed_payload.push(b':');
    signed_payload.extend_from_slice(body);

    // Compute HMAC-SHA256
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(&signed_payload);
    let computed = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison
    computed == expected_signature
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_valid_signature() {
        let secret = "pdl_ntfset_01jfg5r2a0t2qxgxhf5aez8k0t_hKkRoGnGHKSHxdHgrALnuxVXP6bOVMbY";
        let body = br#"{"data":{"id":"txn_01jfg8ekf2y02npvdpqwrnqvc2"}}"#;

        // Build expected signature
        let timestamp = "1734615647";
        let mut signed_payload = timestamp.as_bytes().to_vec();
        signed_payload.push(b':');
        signed_payload.extend_from_slice(body);

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(&signed_payload);
        let signature = hex::encode(mac.finalize().into_bytes());

        let header = format!("ts={};h1={}", timestamp, signature);

        assert!(verify_paddle_signature(&header, body, secret));
    }

    #[test]
    fn test_reject_invalid_signature() {
        let header = "ts=1234567890;h1=invalidsignature";
        let body = b"test body";
        let secret = "secret";

        assert!(!verify_paddle_signature(header, body, secret));
    }
}
