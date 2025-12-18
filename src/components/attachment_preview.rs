// Attachment Preview Component - Renders file attachments
use gpui::*;
use gpui::prelude::*;
use gpui_component::button::Button;
use crate::gpui_theme::GpuiTheme;
use crate::types::{Attachment, AttachmentData};

/// Render a single attachment preview
pub fn render_attachment(
    attachment: &Attachment,
    theme: &GpuiTheme,
    font_size: f32,
    show_remove: bool,
    on_remove: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
) -> AnyElement {
    match &attachment.data {
        AttachmentData::Image { base64 } => {
            render_image_attachment(attachment, base64, theme, font_size, show_remove, on_remove)
        }
        AttachmentData::Text { content } => {
            render_text_attachment(attachment, content, theme, font_size, show_remove, on_remove)
        }
    }
}

/// Render a compact attachment chip (for chat input pending attachments)
pub fn render_attachment_chip(
    file_name: &str,
    theme: &GpuiTheme,
    font_size: f32,
    index: usize,
    on_remove: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let file_name_owned = file_name.to_string();
    let display_name = truncate_filename(&file_name_owned, 20);
    let icon = get_file_icon(&file_name_owned);

    div()
        .flex()
        .items_center()
        .gap_1()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(theme.surface)
        .border_1()
        .border_color(theme.border)
        .child(
            div()
                .text_size(px(font_size - 2.0))
                .child(icon)
        )
        .child(
            div()
                .text_size(px(font_size - 2.0))
                .text_color(theme.text_secondary)
                .child(display_name)
        )
        .child(
            Button::new(SharedString::from(format!("remove_attachment_{}", index)))
                .label("x")
                .compact()
                .on_click(move |_, window, app| {
                    on_remove(index, window, app);
                })
        )
        .into_any_element()
}

fn render_image_attachment(
    attachment: &Attachment,
    base64_data: &str,
    theme: &GpuiTheme,
    font_size: f32,
    show_remove: bool,
    on_remove: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
) -> AnyElement {
    // Decode base64 to check if it's valid
    let is_valid = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        base64_data,
    ).is_ok();

    let file_name = attachment.file_name.clone();

    div()
        .flex()
        .flex_col()
        .gap_1()
        .p_2()
        .rounded_md()
        .bg(theme.surface)
        .border_1()
        .border_color(theme.border)
        .max_w(px(400.0))
        .child(
            if is_valid {
                // For now, show a placeholder since GPUI image rendering from bytes
                // requires specific handling. Show image icon with filename.
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .h(px(100.0))
                    .bg(theme.background)
                    .rounded_sm()
                    .child(
                        div()
                            .text_size(px(32.0))
                            .text_color(theme.text_muted)
                            .child("🖼️")
                    )
                    .child(
                        div()
                            .text_size(px(font_size - 2.0))
                            .text_color(theme.text_muted)
                            .child("Image preview")
                    )
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(60.0))
                    .bg(theme.background)
                    .rounded_sm()
                    .child(
                        div()
                            .text_color(theme.text_muted)
                            .child("Invalid image data")
                    )
                    .into_any_element()
            }
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(font_size - 2.0))
                                .child("🖼️")
                        )
                        .child(
                            div()
                                .text_size(px(font_size - 2.0))
                                .text_color(theme.text_secondary)
                                .child(truncate_filename(&file_name, 30))
                        )
                )
                .when(show_remove, |this| {
                    if let Some(callback) = on_remove {
                        this.child(
                            Button::new("remove_img")
                                .label("x")
                                .compact()
                                .on_click(move |_, window, app| {
                                    callback(window, app);
                                })
                        )
                    } else {
                        this
                    }
                })
        )
        .into_any_element()
}

fn render_text_attachment(
    attachment: &Attachment,
    content: &str,
    theme: &GpuiTheme,
    font_size: f32,
    show_remove: bool,
    on_remove: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
) -> AnyElement {
    let file_name = attachment.file_name.clone();
    let icon = get_file_icon(&file_name);

    // Get a preview snippet (first 150 chars)
    let preview = if content.len() > 150 {
        format!("{}...", &content[..150])
    } else {
        content.to_string()
    };

    div()
        .flex()
        .flex_col()
        .gap_1()
        .p_2()
        .rounded_md()
        .bg(theme.surface)
        .border_1()
        .border_color(theme.border)
        .max_w(px(400.0))
        // Header with icon and filename
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(font_size))
                                .child(icon)
                        )
                        .child(
                            div()
                                .text_size(px(font_size - 1.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text_primary)
                                .child(truncate_filename(&file_name, 30))
                        )
                )
                .when(show_remove, |this| {
                    if let Some(callback) = on_remove {
                        this.child(
                            Button::new("remove_text")
                                .label("x")
                                .compact()
                                .on_click(move |_, window, app| {
                                    callback(window, app);
                                })
                        )
                    } else {
                        this
                    }
                })
        )
        // Content preview
        .child(
            div()
                .p_2()
                .rounded_sm()
                .bg(theme.background)
                .child(
                    div()
                        .text_size(px(font_size - 2.0))
                        .text_color(theme.text_muted)
                        .font_family("monospace")
                        .child(preview)
                )
        )
        .into_any_element()
}

/// Get an appropriate icon for a file type
fn get_file_icon(file_name: &str) -> &'static str {
    let lower = file_name.to_lowercase();

    if lower.ends_with(".pdf") {
        "📄"
    } else if lower.ends_with(".txt") || lower.ends_with(".md") {
        "📝"
    } else if lower.ends_with(".rs") || lower.ends_with(".py") || lower.ends_with(".js") ||
              lower.ends_with(".ts") || lower.ends_with(".go") || lower.ends_with(".java") ||
              lower.ends_with(".c") || lower.ends_with(".cpp") || lower.ends_with(".h") {
        "💻"
    } else if lower.ends_with(".json") || lower.ends_with(".yaml") || lower.ends_with(".yml") ||
              lower.ends_with(".toml") || lower.ends_with(".xml") {
        "⚙️"
    } else if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") ||
              lower.ends_with(".gif") || lower.ends_with(".webp") || lower.ends_with(".svg") {
        "🖼️"
    } else {
        "📎"
    }
}

/// Truncate a filename to a maximum length, preserving the extension
fn truncate_filename(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        return name.to_string();
    }

    // Try to preserve the extension
    if let Some(dot_pos) = name.rfind('.') {
        let extension = &name[dot_pos..];
        let base_max = max_len.saturating_sub(extension.len() + 3); // -3 for "..."

        if base_max > 0 {
            let base = &name[..base_max.min(dot_pos)];
            return format!("{}...{}", base, extension);
        }
    }

    // Fallback: just truncate
    format!("{}...", &name[..max_len.saturating_sub(3)])
}

/// Render attachments for a message (shown after message content)
pub fn render_message_attachments(
    attachments: &[Attachment],
    theme: &GpuiTheme,
    font_size: f32,
) -> AnyElement {
    if attachments.is_empty() {
        return div().into_any_element();
    }

    div()
        .flex()
        .flex_wrap()
        .gap_2()
        .pt_2()
        .children(
            attachments.iter().map(|attachment| {
                render_attachment(attachment, theme, font_size, false, None)
            })
        )
        .into_any_element()
}
