use crate::theme::ThemeType;
use crate::types::{ChatMap, FolderMap};
use anyhow::Result;

pub struct Storage;

impl Storage {
    pub fn load_api_key() -> Result<String> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            if let Ok(key) = std::fs::read_to_string(app_dir.join("api_key.txt")) {
                return Ok(key.trim().to_string());
            }
        }
        Ok(String::new())
    }

    pub fn save_api_key(api_key: &str) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            std::fs::write(app_dir.join("api_key.txt"), api_key)?;
        }
        Ok(())
    }

    pub fn load_chats() -> Result<ChatMap> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            if let Ok(chats_data) = std::fs::read_to_string(app_dir.join("chats.json")) {
                if let Ok(chats) = serde_json::from_str::<ChatMap>(&chats_data) {
                    return Ok(chats);
                }
            }
        }
        Ok(ChatMap::new())
    }

    pub fn save_chats(chats: &ChatMap) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            let chats_json = serde_json::to_string_pretty(chats)?;
            std::fs::write(app_dir.join("chats.json"), chats_json)?;
        }
        Ok(())
    }

    pub fn load_folders() -> Result<FolderMap> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            if let Ok(folders_data) = std::fs::read_to_string(app_dir.join("folders.json")) {
                if let Ok(folders) = serde_json::from_str::<FolderMap>(&folders_data) {
                    return Ok(folders);
                }
            }
        }
        Ok(FolderMap::new())
    }

    pub fn save_folders(folders: &FolderMap) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            let folders_json = serde_json::to_string_pretty(folders)?;
            std::fs::write(app_dir.join("folders.json"), folders_json)?;
        }
        Ok(())
    }

    pub fn load_theme() -> Result<ThemeType> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            if let Ok(theme_data) = std::fs::read_to_string(app_dir.join("theme.txt")) {
                match theme_data.trim() {
                    "OneLight" => return Ok(ThemeType::OneLight),
                    "OneDark" => return Ok(ThemeType::OneDark),
                    _ => {}
                }
            }
        }
        Ok(ThemeType::default())
    }

    pub fn save_theme(theme_type: &ThemeType) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            let theme_str = match theme_type {
                ThemeType::OneLight => "OneLight",
                ThemeType::OneDark => "OneDark",
            };
            std::fs::write(app_dir.join("theme.txt"), theme_str)?;
        }
        Ok(())
    }

    pub fn load_font_size() -> Result<f32> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            if let Ok(font_data) = std::fs::read_to_string(app_dir.join("font_size.txt")) {
                if let Ok(size) = font_data.trim().parse::<f32>() {
                    return Ok(size);
                }
            }
        }
        Ok(14.0) // Default font size
    }

    pub fn save_font_size(font_size: f32) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            std::fs::write(app_dir.join("font_size.txt"), font_size.to_string())?;
        }
        Ok(())
    }
}
