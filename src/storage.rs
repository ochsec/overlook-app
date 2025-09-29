use crate::theme::ThemeType;
use crate::types::{ChatMap, FolderMap};
use anyhow::{anyhow, Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use keyring::Entry;
use rand::rngs::OsRng;
use rand::RngCore;
use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey, NONCE_LEN};
use zeroize::{Zeroize, Zeroizing};

const SERVICE_NAME: &str = "overlook";
const MASTER_KEY_ACCOUNT: &str = "master-key";
const CHAT_FILE_NAME: &str = "chats.enc";
const VERSION_BYTE: u8 = 1;
const AAD: &[u8] = b"overlook-v1";
const MASTER_KEY_LEN: usize = 32;

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
            let chats_path = app_dir.join(CHAT_FILE_NAME);

            if let Ok(ciphertext) = std::fs::read(chats_path) {
                let mut key = ensure_master_key()?;
                let plaintext = Zeroizing::new(
                    decrypt_blob(&ciphertext, &key).context("failed to decrypt chats blob")?,
                );
                key.zeroize();

                let chats = serde_json::from_slice(&plaintext)
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
            let mut key = ensure_master_key()?;
            let chats_json = Zeroizing::new(serde_json::to_vec(chats)?);
            let ciphertext =
                encrypt_blob(&chats_json, &key).context("failed to encrypt chats JSON")?;
            key.zeroize();
            std::fs::write(app_dir.join(CHAT_FILE_NAME), ciphertext)?;
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

fn ensure_master_key() -> Result<[u8; MASTER_KEY_LEN]> {
    let entry = master_key_entry()?;
    match entry.get_password() {
        Ok(secret) => {
            let decoded = BASE64_STANDARD
                .decode(secret.trim())
                .context("keychain entry was not valid base64")?;
            let key: [u8; MASTER_KEY_LEN] = decoded
                .try_into()
                .map_err(|_| anyhow!("keychain entry wrong length"))?;
            Ok(key)
        }
        Err(keyring::Error::NoEntry) => {
            let mut key = [0u8; MASTER_KEY_LEN];
            OsRng.fill_bytes(&mut key);
            let encoded = BASE64_STANDARD.encode(key);
            entry
                .set_password(&encoded)
                .map_err(|e| anyhow!("failed to store master key in keychain: {}", e))?;
            Ok(key)
        }
        Err(e) => Err(anyhow!("failed to load master key from keychain: {}", e)),
    }
}

fn encrypt_blob(plaintext: &[u8], key: &[u8; MASTER_KEY_LEN]) -> Result<Vec<u8>> {
    let unbound = UnboundKey::new(&aead::CHACHA20_POLY1305, key)
        .map_err(|_| anyhow!("failed to create AEAD key"))?;
    let sealing_key = LessSafeKey::new(unbound);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut buffer = plaintext.to_vec();
    buffer.reserve(aead::CHACHA20_POLY1305.tag_len());

    if let Err(_) = sealing_key.seal_in_place_append_tag(nonce, Aad::from(AAD), &mut buffer) {
        buffer.zeroize();
        return Err(anyhow!("AEAD seal failed"));
    }

    let mut output = Vec::with_capacity(1 + nonce_bytes.len() + buffer.len());
    output.push(VERSION_BYTE);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&buffer);
    buffer.zeroize();
    Ok(output)
}

fn decrypt_blob(ciphertext: &[u8], key: &[u8; MASTER_KEY_LEN]) -> Result<Vec<u8>> {
    if ciphertext.len() < 1 + NONCE_LEN {
        return Err(anyhow!("ciphertext too short"));
    }

    let version = ciphertext[0];
    if version != VERSION_BYTE {
        return Err(anyhow!("unsupported ciphertext version: {}", version));
    }

    let mut nonce_bytes = [0u8; NONCE_LEN];
    nonce_bytes.copy_from_slice(&ciphertext[1..1 + NONCE_LEN]);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut buffer = ciphertext[1 + NONCE_LEN..].to_vec();
    let unbound = UnboundKey::new(&aead::CHACHA20_POLY1305, key)
        .map_err(|_| anyhow!("failed to create AEAD key"))?;
    let opening_key = LessSafeKey::new(unbound);
    let plaintext = match opening_key.open_in_place(nonce, Aad::from(AAD), &mut buffer) {
        Ok(data) => data.to_vec(),
        Err(_) => {
            buffer.zeroize();
            return Err(anyhow!("AEAD open failed"));
        }
    };
    buffer.zeroize();
    let result = plaintext;
    Ok(result)
}

fn master_key_entry() -> Result<Entry> {
    Entry::new(SERVICE_NAME, MASTER_KEY_ACCOUNT)
        .map_err(|e| anyhow!("failed to create keychain entry handle: {}", e))
}
