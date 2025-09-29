use crate::types::Chat;

pub struct Exporter;

impl Exporter {
    pub fn generate_chat_markdown(chat: &Chat) -> String {
        let mut markdown = format!("# {}\n\n", chat.title);
        markdown.push_str(&format!("**Model:** {}\n", chat.model));
        markdown.push_str(&format!(
            "**Created:** {}\n",
            chat.created_at.format("%Y-%m-%d %H:%M:%S")
        ));
        markdown.push_str(&format!(
            "**Updated:** {}\n\n",
            chat.updated_at.format("%Y-%m-%d %H:%M:%S")
        ));

        for message in &chat.messages {
            let role = if message.role == "user" {
                "User"
            } else {
                "Assistant"
            };
            markdown.push_str(&format!("## {}\n\n{}\n\n", role, message.content));
        }

        markdown
    }

    pub fn export_to_file(content: &str, filename: &str) {
        if let Some(downloads_dir) = dirs::download_dir() {
            let file_path = downloads_dir.join(filename);
            let _ = std::fs::write(file_path, content);
        }
    }
}
