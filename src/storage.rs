//! Storage for Overlook application data.
//!
//! Chat history and projects are stored unencrypted in JSON format.
//! (Encryption feature is disabled - see commented code below)
//!
//! # File Locations
//!
//! - macOS: `~/Library/Application Support/overlook/`
//! - Linux: `~/.config/overlook/`
//! - Windows: `%APPDATA%\overlook\`

use crate::gpui_theme::ThemeType;
use crate::types::{ChatMap, FolderMap, Project, ProjectMap, ProviderKeys};
use anyhow::{Context, Result};
// use anyhow::anyhow;
// use base64::prelude::BASE64_STANDARD;
// use base64::Engine;
// use keyring::Entry;
// use rand::rngs::OsRng;
// use rand::RngCore;
// use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey, NONCE_LEN};
// use zeroize::{Zeroize, Zeroizing};

// const SERVICE_NAME: &str = "overlook";
// const MASTER_KEY_ACCOUNT: &str = "master-key";
const CHAT_FILE_NAME: &str = "chats.json";
const PROJECTS_FILE_NAME: &str = "projects.json";
const API_KEYS_FILE_NAME: &str = "api_keys.json";
// const VERSION_BYTE: u8 = 1;
// const AAD: &[u8] = b"overlook-v1";
// const MASTER_KEY_LEN: usize = 32;

pub struct Storage;

impl Storage {
    pub fn load_api_keys() -> Result<ProviderKeys> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            let json_path = app_dir.join(API_KEYS_FILE_NAME);

            if let Ok(keys_json) = std::fs::read_to_string(&json_path) {
                if let Ok(keys) = serde_json::from_str::<ProviderKeys>(&keys_json) {
                    return Ok(keys);
                }
            }

            // Fallback to legacy single key storage
            if let Ok(key) = std::fs::read_to_string(app_dir.join("api_key.txt")) {
                let mut keys = ProviderKeys::default();
                keys.openrouter = key.trim().to_string();
                return Ok(keys);
            }
        }
        Ok(ProviderKeys::default())
    }

    pub fn save_api_keys(keys: &ProviderKeys) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            let data = serde_json::to_string_pretty(keys)?;
            std::fs::write(app_dir.join(API_KEYS_FILE_NAME), data)?;
        }
        Ok(())
    }

    // Unencrypted storage (encryption disabled)
    pub fn load_chats() -> Result<ChatMap> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            let chats_path = app_dir.join(CHAT_FILE_NAME);

            if let Ok(chats_data) = std::fs::read_to_string(chats_path) {
                let chats = serde_json::from_str(&chats_data)
                    .context("failed to deserialize chats JSON")?;
                return Ok(chats);
            }
        }
        Ok(ChatMap::new())
    }

    pub fn save_chats(chats: &ChatMap) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            let chats_json = serde_json::to_string_pretty(chats)?;
            std::fs::write(app_dir.join(CHAT_FILE_NAME), chats_json)?;
        }
        Ok(())
    }

    // Encrypted versions (disabled)
    // pub fn load_chats() -> Result<ChatMap> {
    //     if let Some(data_dir) = dirs::config_dir() {
    //         let app_dir = data_dir.join("overlook");
    //         let chats_path = app_dir.join(CHAT_FILE_NAME);
    //
    //         if let Ok(ciphertext) = std::fs::read(chats_path) {
    //             let mut key = ensure_master_key()?;
    //             let plaintext = Zeroizing::new(
    //                 decrypt_blob(&ciphertext, &key).context("failed to decrypt chats blob")?,
    //             );
    //             key.zeroize();
    //
    //             let chats = serde_json::from_slice(&plaintext)
    //                 .context("failed to deserialize chats JSON")?;
    //             return Ok(chats);
    //         }
    //     }
    //     Ok(ChatMap::new())
    // }
    //
    // pub fn save_chats(chats: &ChatMap) -> Result<()> {
    //     if let Some(data_dir) = dirs::config_dir() {
    //         let app_dir = data_dir.join("overlook");
    //         std::fs::create_dir_all(&app_dir)?;
    //         let mut key = ensure_master_key()?;
    //         let chats_json = Zeroizing::new(serde_json::to_vec(chats)?);
    //         let ciphertext =
    //             encrypt_blob(&chats_json, &key).context("failed to encrypt chats JSON")?;
    //         key.zeroize();
    //         std::fs::write(app_dir.join(CHAT_FILE_NAME), ciphertext)?;
    //     }
    //     Ok(())
    // }

    // Unencrypted storage (encryption disabled)
    pub fn load_projects() -> Result<ProjectMap> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            let projects_path = app_dir.join(PROJECTS_FILE_NAME);

            // Try loading projects.json
            if let Ok(projects_data) = std::fs::read_to_string(&projects_path) {
                if let Ok(projects) = serde_json::from_str::<ProjectMap>(&projects_data) {
                    return Ok(projects);
                }
            }

            // Fall back to migrating old folders format
            if let Ok(folders_data) = std::fs::read_to_string(app_dir.join("folders.json")) {
                if let Ok(old_folders) = serde_json::from_str::<FolderMap>(&folders_data) {
                    println!("🔄 Migrating folders to projects format...");
                    let projects = Self::migrate_folders_to_projects(old_folders);

                    // Save migrated data
                    if let Err(e) = Self::save_projects(&projects) {
                        eprintln!("⚠️  Failed to save migrated projects: {}", e);
                    } else {
                        println!(
                            "✅ Successfully migrated {} folders to projects",
                            projects.len()
                        );
                        // Backup old file
                        let _ = std::fs::rename(
                            app_dir.join("folders.json"),
                            app_dir.join("folders.json.backup"),
                        );
                    }

                    return Ok(projects);
                }
            }
        }
        Ok(ProjectMap::new())
    }

    // Encrypted version (disabled)
    // pub fn load_projects() -> Result<ProjectMap> {
    //     if let Some(data_dir) = dirs::config_dir() {
    //         let app_dir = data_dir.join("overlook");
    //         let projects_path = app_dir.join(PROJECTS_FILE_NAME);
    //
    //         // Try loading encrypted format first
    //         if let Ok(ciphertext) = std::fs::read(&projects_path) {
    //             let mut key = ensure_master_key()?;
    //             let plaintext = Zeroizing::new(
    //                 decrypt_blob(&ciphertext, &key).context("failed to decrypt projects blob")?,
    //             );
    //             key.zeroize();
    //
    //             let projects = serde_json::from_slice(&plaintext)
    //                 .context("failed to deserialize projects JSON")?;
    //             return Ok(projects);
    //         }
    //
    //         // Fall back to migrating old unencrypted projects.json
    //         if let Ok(projects_data) = std::fs::read_to_string(app_dir.join("projects.json")) {
    //             if let Ok(projects) = serde_json::from_str::<ProjectMap>(&projects_data) {
    //                 println!("🔄 Migrating unencrypted projects.json to encrypted format...");
    //                 // Save in encrypted format
    //                 if let Err(e) = Self::save_projects(&projects) {
    //                     eprintln!("⚠️  Failed to save encrypted projects: {}", e);
    //                 } else {
    //                     println!("✅ Successfully migrated projects to encrypted format");
    //                     // Backup old file
    //                     let _ = std::fs::rename(
    //                         app_dir.join("projects.json"),
    //                         app_dir.join("projects.json.backup")
    //                     );
    //                 }
    //                 return Ok(projects);
    //             }
    //         }
    //
    //         // Fall back to migrating old folders format
    //         if let Ok(folders_data) = std::fs::read_to_string(app_dir.join("folders.json")) {
    //             if let Ok(old_folders) = serde_json::from_str::<FolderMap>(&folders_data) {
    //                 println!("🔄 Migrating folders to encrypted projects format...");
    //                 let projects = Self::migrate_folders_to_projects(old_folders);
    //
    //                 // Save migrated data in encrypted format
    //                 if let Err(e) = Self::save_projects(&projects) {
    //                     eprintln!("⚠️  Failed to save migrated projects: {}", e);
    //                 } else {
    //                     println!("✅ Successfully migrated {} folders to encrypted projects", projects.len());
    //                     // Backup old file
    //                     let _ = std::fs::rename(
    //                         app_dir.join("folders.json"),
    //                         app_dir.join("folders.json.backup")
    //                     );
    //                 }
    //
    //                 return Ok(projects);
    //             }
    //         }
    //     }
    //     Ok(ProjectMap::new())
    // }

    fn migrate_folders_to_projects(folders: FolderMap) -> ProjectMap {
        use chrono::Utc;
        let now = Utc::now();
        folders
            .into_iter()
            .map(|(id, name)| {
                let project = Project {
                    id: id.clone(),
                    name,
                    created_at: now,
                    updated_at: now,
                    color: None,
                };
                (id, project)
            })
            .collect()
    }

    pub fn save_projects(projects: &ProjectMap) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            let projects_json = serde_json::to_string_pretty(projects)?;
            std::fs::write(app_dir.join(PROJECTS_FILE_NAME), projects_json)?;
        }
        Ok(())
    }

    // Encrypted version (disabled)
    // pub fn save_projects(projects: &ProjectMap) -> Result<()> {
    //     if let Some(data_dir) = dirs::config_dir() {
    //         let app_dir = data_dir.join("overlook");
    //         std::fs::create_dir_all(&app_dir)?;
    //         let mut key = ensure_master_key()?;
    //         let projects_json = Zeroizing::new(serde_json::to_vec(projects)?);
    //         let ciphertext =
    //             encrypt_blob(&projects_json, &key).context("failed to encrypt projects JSON")?;
    //         key.zeroize();
    //         std::fs::write(app_dir.join(PROJECTS_FILE_NAME), ciphertext)?;
    //     }
    //     Ok(())
    // }

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

    pub fn load_confirm_delete_chat() -> Result<bool> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            if let Ok(data) = std::fs::read_to_string(app_dir.join("confirm_delete_chat.txt")) {
                match data.trim() {
                    "true" => return Ok(true),
                    "false" => return Ok(false),
                    _ => {}
                }
            }
        }
        Ok(true) // Default: confirm before deleting
    }

    pub fn save_confirm_delete_chat(confirm: bool) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            std::fs::write(app_dir.join("confirm_delete_chat.txt"), confirm.to_string())?;
        }
        Ok(())
    }

    pub fn load_favorite_models() -> Result<Vec<String>> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            if let Ok(favorites_data) =
                std::fs::read_to_string(app_dir.join("favorite_models.json"))
            {
                if let Ok(favorites) = serde_json::from_str::<Vec<String>>(&favorites_data) {
                    return Ok(favorites);
                }
            }
        }
        Ok(Vec::new())
    }

    pub fn save_favorite_models(favorites: &[String]) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            let data = serde_json::to_string_pretty(favorites)?;
            std::fs::write(app_dir.join("favorite_models.json"), data)?;
        }
        Ok(())
    }

    pub fn load_last_provider() -> Result<crate::types::ProviderKind> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            if let Ok(provider_data) = std::fs::read_to_string(app_dir.join("last_provider.txt")) {
                match provider_data.trim() {
                    "OpenRouter" => return Ok(crate::types::ProviderKind::OpenRouter),
                    "OpenAI" => return Ok(crate::types::ProviderKind::OpenAI),
                    "Anthropic" => return Ok(crate::types::ProviderKind::Anthropic),
                    "LMStudio" => return Ok(crate::types::ProviderKind::LMStudio),
                    "Ollama" => return Ok(crate::types::ProviderKind::Ollama),
                    _ => {}
                }
            }
        }
        Ok(crate::types::ProviderKind::default())
    }

    pub fn save_last_provider(provider: &crate::types::ProviderKind) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            let provider_str = match provider {
                crate::types::ProviderKind::OpenRouter => "OpenRouter",
                crate::types::ProviderKind::OpenAI => "OpenAI",
                crate::types::ProviderKind::Anthropic => "Anthropic",
                crate::types::ProviderKind::LMStudio => "LMStudio",
                crate::types::ProviderKind::Ollama => "Ollama",
            };
            std::fs::write(app_dir.join("last_provider.txt"), provider_str)?;
        }
        Ok(())
    }

    pub fn load_last_model() -> Result<String> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            if let Ok(model_data) = std::fs::read_to_string(app_dir.join("last_model.txt")) {
                let model = model_data.trim().to_string();
                if !model.is_empty() {
                    return Ok(model);
                }
            }
        }
        Ok(String::new())
    }

    pub fn save_last_model(model: &str) -> Result<()> {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            std::fs::create_dir_all(&app_dir)?;
            std::fs::write(app_dir.join("last_model.txt"), model)?;
        }
        Ok(())
    }
}

// Encryption functions (disabled)
// fn ensure_master_key() -> Result<[u8; MASTER_KEY_LEN]> {
//     // On Raspberry Pi, use file-based storage to avoid keyring blocking during GUI init
//     #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
//     {
//         return ensure_file_based_master_key();
//     }
//
//     // Try keyring first (works on macOS, Windows, and Linux with Secret Service)
//     #[cfg(not(all(target_arch = "aarch64", target_os = "linux")))]
//     match try_keyring_master_key() {
//         Ok(key) => {
//             println!("✅ Using system keyring for encryption");
//             return Ok(key);
//         }
//         Err(e) => {
//             eprintln!("⚠️  Keyring unavailable ({}), falling back to file-based key storage", e);
//             // Fall back to file-based storage on systems without keyring support
//             return ensure_file_based_master_key();
//         }
//     }
// }
//
// fn try_keyring_master_key() -> Result<[u8; MASTER_KEY_LEN]> {
//     let entry = master_key_entry()?;
//     match entry.get_password() {
//         Ok(secret) => {
//             let decoded = BASE64_STANDARD
//                 .decode(secret.trim())
//                 .context("keychain entry was not valid base64")?;
//             let key: [u8; MASTER_KEY_LEN] = decoded
//                 .try_into()
//                 .map_err(|_| anyhow!("keychain entry wrong length"))?;
//             Ok(key)
//         }
//         Err(keyring::Error::NoEntry) => {
//             let mut key = [0u8; MASTER_KEY_LEN];
//             OsRng.fill_bytes(&mut key);
//             let encoded = BASE64_STANDARD.encode(key);
//             entry
//                 .set_password(&encoded)
//                 .map_err(|e| anyhow!("failed to store master key in keychain: {}", e))?;
//             Ok(key)
//         }
//         Err(e) => Err(anyhow!("keyring error: {}", e)),
//     }
// }
//
// fn ensure_file_based_master_key() -> Result<[u8; MASTER_KEY_LEN]> {
//     let data_dir = dirs::config_dir()
//         .ok_or_else(|| anyhow!("cannot determine config directory"))?;
//     let app_dir = data_dir.join("overlook");
//     std::fs::create_dir_all(&app_dir)?;
//     let key_file = app_dir.join(".master_key");
//
//     // Try to load existing key
//     if let Ok(encoded) = std::fs::read_to_string(&key_file) {
//         let decoded = BASE64_STANDARD
//             .decode(encoded.trim())
//             .context("stored key was not valid base64")?;
//         let key: [u8; MASTER_KEY_LEN] = decoded
//             .try_into()
//             .map_err(|_| anyhow!("stored key wrong length"))?;
//         return Ok(key);
//     }
//
//     // Generate new key
//     let mut key = [0u8; MASTER_KEY_LEN];
//     OsRng.fill_bytes(&mut key);
//     let encoded = BASE64_STANDARD.encode(key);
//
//     // Save with restrictive permissions (owner read/write only)
//     std::fs::write(&key_file, encoded)?;
//
//     #[cfg(unix)]
//     {
//         use std::os::unix::fs::PermissionsExt;
//         let mut perms = std::fs::metadata(&key_file)?.permissions();
//         perms.set_mode(0o600); // Owner read/write only
//         std::fs::set_permissions(&key_file, perms)?;
//     }
//
//     println!("🔐 Created new file-based encryption key at {:?}", key_file);
//     println!("⚠️  Keep this file secure - it protects your chat data!");
//
//     Ok(key)
// }
//
// fn encrypt_blob(plaintext: &[u8], key: &[u8; MASTER_KEY_LEN]) -> Result<Vec<u8>> {
//     let unbound = UnboundKey::new(&aead::CHACHA20_POLY1305, key)
//         .map_err(|_| anyhow!("failed to create AEAD key"))?;
//     let sealing_key = LessSafeKey::new(unbound);
//
//     let mut nonce_bytes = [0u8; NONCE_LEN];
//     OsRng.fill_bytes(&mut nonce_bytes);
//     let nonce = Nonce::assume_unique_for_key(nonce_bytes);
//
//     let mut buffer = plaintext.to_vec();
//     buffer.reserve(aead::CHACHA20_POLY1305.tag_len());
//
//     if let Err(_) = sealing_key.seal_in_place_append_tag(nonce, Aad::from(AAD), &mut buffer) {
//         buffer.zeroize();
//         return Err(anyhow!("AEAD seal failed"));
//     }
//
//     let mut output = Vec::with_capacity(1 + nonce_bytes.len() + buffer.len());
//     output.push(VERSION_BYTE);
//     output.extend_from_slice(&nonce_bytes);
//     output.extend_from_slice(&buffer);
//     buffer.zeroize();
//     Ok(output)
// }
//
// fn decrypt_blob(ciphertext: &[u8], key: &[u8; MASTER_KEY_LEN]) -> Result<Vec<u8>> {
//     if ciphertext.len() < 1 + NONCE_LEN {
//         return Err(anyhow!("ciphertext too short"));
//     }
//
//     let version = ciphertext[0];
//     if version != VERSION_BYTE {
//         return Err(anyhow!("unsupported ciphertext version: {}", version));
//     }
//
//     let mut nonce_bytes = [0u8; NONCE_LEN];
//     nonce_bytes.copy_from_slice(&ciphertext[1..1 + NONCE_LEN]);
//     let nonce = Nonce::assume_unique_for_key(nonce_bytes);
//
//     let mut buffer = ciphertext[1 + NONCE_LEN..].to_vec();
//     let unbound = UnboundKey::new(&aead::CHACHA20_POLY1305, key)
//         .map_err(|_| anyhow!("failed to create AEAD key"))?;
//     let opening_key = LessSafeKey::new(unbound);
//     let plaintext = match opening_key.open_in_place(nonce, Aad::from(AAD), &mut buffer) {
//         Ok(data) => data.to_vec(),
//         Err(_) => {
//             buffer.zeroize();
//             return Err(anyhow!("AEAD open failed"));
//         }
//     };
//     buffer.zeroize();
//     let result = plaintext;
//     Ok(result)
// }
//
// fn master_key_entry() -> Result<Entry> {
//     Entry::new(SERVICE_NAME, MASTER_KEY_ACCOUNT)
//         .map_err(|e| anyhow!("failed to create keychain entry handle: {}", e))
// }
