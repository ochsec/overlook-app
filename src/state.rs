// Global Application State Management for GPUI
// Replaces the monolithic OverlookApp struct with structured state

use crate::gpui_theme::{GpuiTheme, ThemeType};
use crate::storage::Storage;
use crate::types::{Chat, ChatMap, ModelDescriptor, Project, ProjectMap, ProviderKeys, ProviderKind};
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
    pub last_export_result: Option<Result<String, String>>,
}

impl AppState {
    pub fn new() -> Self {
        let theme_type = Storage::load_theme().unwrap_or_default();
        let theme = GpuiTheme::from_theme_type(&theme_type);
        let font_size = Storage::load_font_size().unwrap_or(14.0);

        Self {
            current_view: ViewMode::Chat,
            theme_type,
            theme,
            font_size,

            api_keys: ProviderKeys::default(),
            current_provider: ProviderKind::default(),
            current_model: String::default(),
            available_models: Vec::new(),
            loading_models: false,

            chats: ChatMap::default(),
            projects: ProjectMap::default(),
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

            message_input: String::new(),
            selected_attachments: Vec::new(),
            sending_message: false,

            streaming_enabled: true,
            streaming_active: false,
            streaming_message: String::new(),
            stream_completed: false,
            streaming_message_persisted: false,

            last_export_result: None,
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
        self.current_chat_id.as_ref()
            .and_then(|id| self.chats.get(id))
    }

    pub fn get_current_project(&self) -> Option<&Project> {
        self.current_project_id.as_ref()
            .and_then(|id| self.projects.get(id))
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
