//! License data storage.
//!
//! Stores license information in the app's config directory:
//! - macOS: ~/Library/Application Support/overlook/license.json
//! - Linux: ~/.config/overlook/license.json
//! - Windows: %APPDATA%\overlook\license.json

use anyhow::Result;

use super::types::LicenseData;

const LICENSE_FILE_NAME: &str = "license.json";

/// Load license data from disk
pub fn load_license_data() -> Result<LicenseData> {
    if let Some(data_dir) = dirs::config_dir() {
        let app_dir = data_dir.join("overlook");
        let license_path = app_dir.join(LICENSE_FILE_NAME);

        if let Ok(data) = std::fs::read_to_string(license_path) {
            if let Ok(license_data) = serde_json::from_str::<LicenseData>(&data) {
                return Ok(license_data);
            }
        }
    }
    Ok(LicenseData::default())
}

/// Save license data to disk
pub fn save_license_data(data: &LicenseData) -> Result<()> {
    if let Some(data_dir) = dirs::config_dir() {
        let app_dir = data_dir.join("overlook");
        std::fs::create_dir_all(&app_dir)?;
        let json = serde_json::to_string_pretty(data)?;
        std::fs::write(app_dir.join(LICENSE_FILE_NAME), json)?;
    }
    Ok(())
}
