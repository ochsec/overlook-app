// Global Application State Management for GPUI
// Replaces the monolithic OverlookApp struct with structured state

use crate::gpui_theme::{GpuiTheme, ThemeType};
use crate::license::{self, LicenseData, LicenseStatus};
use crate::storage::Storage;
use crate::types::{
    Chat, ChatMap, ModelDescriptor, Project, ProjectMap, ProviderKeys, ProviderKind,
};
use gpui::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ViewMode {
    Chat,
    Settings,
    Export,
    Organize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProjectModalMode {
    Create,
    Rename,
}

/// Global application state
pub struct AppState {
    // Core application state
    pub current_view: ViewMode,
    pub theme_type: ThemeType,
    pub theme: GpuiTheme,
    pub font_size: f32,

    // Provider and model state
    pub api_keys: ProviderKeys,
    pub current_provider: ProviderKind,
    pub current_model: String,
    pub available_models: Vec<ModelDescriptor>,
    pub loading_models: bool,

    // Chat and project state
    pub chats: ChatMap,
    pub projects: ProjectMap,
    pub current_chat_id: Option<String>,
    pub current_project_id: Option<String>,

    // UI state
    pub show_project_modal: bool,
    pub show_model_picker_modal: bool,
    pub project_modal_mode: ProjectModalMode,
    pub project_modal_target_id: Option<String>,
    pub new_project_name: String,
    pub model_search_query: String,
    pub favorite_models: Vec<String>,
    pub api_key_masked: bool,
    pub confirm_delete_chat: bool,
    pub scroll_to_bottom: bool,
    /// Height of the projects section in pixels (for slideable divider)
    pub sidebar_projects_height: f32,

    // Message input state
    pub message_input: String,
    pub selected_attachments: Vec<String>,
    pub sending_message: bool,

    // Streaming state
    pub streaming_enabled: bool,
    pub streaming_active: bool,
    pub streaming_message: String,
    pub stream_completed: bool,
    pub streaming_message_persisted: bool,

    // Export state
    pub export_selected_chat_id: Option<String>,
    pub last_export_result: Option<Result<String, String>>,

    // License state
    pub license_data: LicenseData,
    pub license_status: LicenseStatus,
    pub show_license_modal: bool,
    pub license_key_input: String,
}

impl AppState {
    pub fn new() -> Self {
        let theme_type = Storage::load_theme().unwrap_or_default();
        let theme = GpuiTheme::from_theme_type(&theme_type);
        let font_size = Storage::load_font_size().unwrap_or(14.0);

        // Load and check license status
        let mut license_data = license::load_license_data().unwrap_or_default();
        let license_status = license::check_license_status(&mut license_data);
        let show_license_modal = license_status.is_expired();

        Self {
            current_view: ViewMode::Chat,
            theme_type,
            theme,
            font_size,

            api_keys: Storage::load_api_keys().unwrap_or_default(),
            current_provider: Storage::load_last_provider().unwrap_or_default(),
            current_model: Storage::load_last_model().unwrap_or_default(),
            available_models: Vec::new(),
            loading_models: false,

            chats: Storage::load_chats().unwrap_or_default(),
            projects: Storage::load_projects().unwrap_or_default(),
            current_chat_id: None,
            current_project_id: None,

            show_project_modal: false,
            show_model_picker_modal: false,
            project_modal_mode: ProjectModalMode::Create,
            project_modal_target_id: None,
            new_project_name: String::new(),
            model_search_query: String::new(),
            favorite_models: Vec::new(),
            api_key_masked: true,
            confirm_delete_chat: Storage::load_confirm_delete_chat().unwrap_or(true),
            scroll_to_bottom: false,
            sidebar_projects_height: 200.0, // Default height for projects section

            message_input: String::new(),
            selected_attachments: Vec::new(),
            sending_message: false,

            streaming_enabled: true,
            streaming_active: false,
            streaming_message: String::new(),
            stream_completed: false,
            streaming_message_persisted: false,

            export_selected_chat_id: None,
            last_export_result: None,

            license_data,
            license_status,
            show_license_modal,
            license_key_input: String::new(),
        }
    }

    pub fn switch_view(&mut self, view: ViewMode) {
        self.current_view = view;
    }

    pub fn switch_theme(&mut self, theme_type: ThemeType) {
        self.theme_type = theme_type;
        self.theme = GpuiTheme::from_theme_type(&theme_type);
        Storage::save_theme(&theme_type).ok();
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
        Storage::save_font_size(size).ok();
    }

    pub fn get_current_chat(&self) -> Option<&Chat> {
        self.current_chat_id
            .as_ref()
            .and_then(|id| self.chats.get(id))
    }

    pub fn get_current_project(&self) -> Option<&Project> {
        self.current_project_id
            .as_ref()
            .and_then(|id| self.projects.get(id))
    }

    /// Get models filtered by search query and grouped by category
    pub fn get_filtered_grouped_models(&self) -> Vec<(String, Vec<ModelDescriptor>)> {
        use std::collections::HashMap;

        let query = self.model_search_query.to_lowercase();

        // Filter models by search query
        let filtered: Vec<ModelDescriptor> = self
            .available_models
            .iter()
            .filter(|model| {
                if query.is_empty() {
                    true
                } else {
                    model.name.to_lowercase().contains(&query)
                        || model.id.to_lowercase().contains(&query)
                }
            })
            .cloned()
            .collect();

        // Group by category
        let mut groups: HashMap<String, Vec<ModelDescriptor>> = HashMap::new();
        for model in filtered {
            let category = Self::get_model_category(&model);
            groups.entry(category).or_default().push(model);
        }

        // Sort groups and models within groups
        let mut result: Vec<(String, Vec<ModelDescriptor>)> = groups.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));

        for (_, models) in &mut result {
            models.sort_by(|a, b| a.name.cmp(&b.name));
        }

        result
    }

    /// Get category name for a model (for grouping)
    fn get_model_category(model: &ModelDescriptor) -> String {
        // For OpenRouter, extract vendor from model ID (format: vendor/model)
        if model.provider == ProviderKind::OpenRouter {
            if let Some(slash_pos) = model.id.find('/') {
                let vendor = &model.id[..slash_pos];
                // Capitalize and clean up vendor name
                return vendor
                    .split('-')
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
            }
        }

        // For other providers, use simple categorization by model family
        let lower = model.name.to_lowercase();

        if lower.contains("gpt-4") || lower.contains("gpt4") {
            "GPT-4".to_string()
        } else if lower.contains("gpt-3.5") || lower.contains("gpt3") {
            "GPT-3.5".to_string()
        } else if lower.contains("gpt") {
            "GPT".to_string()
        } else if lower.contains("claude-3.5") || lower.contains("claude-3-5") {
            "Claude 3.5".to_string()
        } else if lower.contains("claude-3") || lower.contains("claude3") {
            "Claude 3".to_string()
        } else if lower.contains("claude") {
            "Claude".to_string()
        } else if lower.contains("llama") {
            "Llama".to_string()
        } else if lower.contains("gemini") {
            "Gemini".to_string()
        } else if lower.contains("mistral") || lower.contains("mixtral") {
            "Mistral".to_string()
        } else {
            "Other".to_string()
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
