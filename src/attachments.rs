use crate::types::{Attachment, AttachmentData};
use anyhow::Result;
use base64::Engine;
use std::fs;
use std::path::Path;

pub struct AttachmentProcessor;

impl AttachmentProcessor {
    pub fn process_file(file_path: &str) -> Result<Attachment> {
        let path = Path::new(file_path);
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        let data = match mime_type.as_str() {
            // Image types - encode as base64
            mime if mime.starts_with("image/") => {
                let image_data = fs::read(file_path)?;
                let base64_data = base64::prelude::BASE64_STANDARD.encode(image_data);
                AttachmentData::Image { base64: base64_data }
            }

            // PDF files - extract text
            "application/pdf" => {
                let text_content = Self::extract_pdf_text(file_path)?;
                AttachmentData::Text { content: text_content }
            }

            // Text files
            mime if mime.starts_with("text/") => {
                let text_content = fs::read_to_string(file_path)?;
                AttachmentData::Text { content: text_content }
            }

            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported file type: {}. Supported types: images (PNG, JPEG, GIF, WebP), PDFs, and text files.",
                    mime_type
                ));
            }
        };

        Ok(Attachment {
            file_name,
            file_path: file_path.to_string(),
            mime_type,
            data,
        })
    }

    fn extract_pdf_text(file_path: &str) -> Result<String> {
        let bytes = fs::read(file_path)?;
        let text = pdf_extract::extract_text_from_mem(&bytes)
            .map_err(|e| anyhow::anyhow!("Failed to extract text from PDF: {}", e))?;

        if text.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "PDF appears to be empty or contains only images/scanned content. Text extraction failed."
            ));
        }

        Ok(text)
    }

    pub fn get_supported_extensions() -> Vec<&'static str> {
        vec![
            // Images
            "png", "jpg", "jpeg", "gif", "webp", "bmp",
            // PDFs
            "pdf",
            // Text files
            "txt", "md", "json", "xml", "csv", "log"
        ]
    }

    pub fn is_supported_file(file_path: &str) -> bool {
        let path = Path::new(file_path);
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            Self::get_supported_extensions().contains(&extension.to_lowercase().as_str())
        } else {
            false
        }
    }
}