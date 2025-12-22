//! Hardware fingerprint generation for device binding.
//!
//! Generates a stable machine ID for license binding:
//! - Linux: /etc/machine-id
//! - macOS: IOPlatformUUID
//! - Windows: Registry MachineGuid

use sha2::{Digest, Sha256};
use std::process::Command;

/// Generate a hardware fingerprint for this machine.
/// Returns a SHA-256 hash prefixed with "sha256:".
pub fn generate_fingerprint() -> String {
    let machine_id = get_machine_id().unwrap_or_else(|e| {
        eprintln!("Failed to get machine ID: {}, using fallback", e);
        fallback_machine_id()
    });

    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    let hash = hasher.finalize();
    format!("sha256:{}", hex::encode(hash))
}

#[cfg(target_os = "linux")]
fn get_machine_id() -> anyhow::Result<String> {
    // Primary: /etc/machine-id (systemd-generated, stable across reboots)
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        let id = id.trim().to_string();
        if !id.is_empty() {
            return Ok(id);
        }
    }

    // Fallback: /var/lib/dbus/machine-id
    if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
        let id = id.trim().to_string();
        if !id.is_empty() {
            return Ok(id);
        }
    }

    anyhow::bail!("No machine-id found")
}

#[cfg(target_os = "macos")]
fn get_machine_id() -> anyhow::Result<String> {
    // Use ioreg to get IOPlatformUUID
    let output = Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse IOPlatformUUID from output
    for line in stdout.lines() {
        if line.contains("IOPlatformUUID") {
            // Line format: "IOPlatformUUID" = "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"
            if let Some(start) = line.rfind('"') {
                let before_last = &line[..start];
                if let Some(uuid_start) = before_last.rfind('"') {
                    let uuid = &before_last[uuid_start + 1..];
                    if !uuid.is_empty() {
                        return Ok(uuid.to_string());
                    }
                }
            }
        }
    }

    anyhow::bail!("IOPlatformUUID not found")
}

#[cfg(target_os = "windows")]
fn get_machine_id() -> anyhow::Result<String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(r"SOFTWARE\Microsoft\Cryptography")?;
    let guid: String = key.get_value("MachineGuid")?;
    Ok(guid)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn get_machine_id() -> anyhow::Result<String> {
    anyhow::bail!("Unsupported platform")
}

/// Fallback: combine hostname and username
fn fallback_machine_id() -> String {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown-host".to_string());

    let username = whoami::username();

    format!("{}@{}", username, hostname)
}
