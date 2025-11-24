// Organize View - Project and Chat management
use gpui::*;
use gpui::prelude::*;
use gpui_component::button::Button;
use gpui_component::input::{InputState, Input};
use gpui_component::menu::DropdownMenu;
use gpui_component::{StyledExt, scroll::ScrollbarAxis};
use chrono::Utc;
use uuid::Uuid;
use crate::state::AppState;
use crate::storage::Storage;
use crate::types::Project;

pub struct OrganizeView {
    state: Entity<AppState>,
    new_project_input: Entity<InputState>,
    editing_chat_id: Option<String>,
    editing_project_id: Option<String>,
}

impl OrganizeView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let new_project_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("New project name...")
        });
        Self {
            state,
            new_project_input,
            editing_chat_id: None,
            editing_project_id: None,
        }
    }
}

impl Render for OrganizeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();
        let theme = state.theme.clone();
        let font_size = state.font_size;

        // Get all projects sorted by name
        let mut projects: Vec<_> = state.projects.values().cloned().collect();
        projects.sort_by(|a, b| a.name.cmp(&b.name));

        // Get all chats sorted by updated_at (most recent first)
        let mut chats: Vec<_> = state.chats.values().cloned().collect();
        chats.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        // Count chats per project
        let unassigned_count = chats.iter().filter(|c| c.project_id.is_none()).count();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.text_primary)
            .child(
                // Header
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .child("Organize Chats")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_muted)
                            .child(format!("{} chats, {} projects", chats.len(), projects.len()))
                    )
            )
            .child(
                // Main content - two columns
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    // Left column - Projects
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(280.0))
                            .border_r_1()
                            .border_color(theme.border)
                            .bg(theme.surface)
                            // Projects header
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .p_3()
                                    .border_b_1()
                                    .border_color(theme.border)
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text_secondary)
                                            .child("Projects")
                                    )
                            )
                            // New project input
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .p_2()
                                    .border_b_1()
                                    .border_color(theme.border)
                                    .child(
                                        div()
                                            .flex_1()
                                            .child(Input::new(&self.new_project_input))
                                    )
                                    .child({
                                        let state_model = state_model.clone();
                                        let input = self.new_project_input.clone();
                                        Button::new("add_project")
                                            .label("+")
                                            .compact()
                                            .on_click(move |_, window, app| {
                                                let name = input.read(app).value().to_string();
                                                let name = name.trim();
                                                if !name.is_empty() {
                                                    let project = Project {
                                                        id: Uuid::new_v4().to_string(),
                                                        name: name.to_string(),
                                                        created_at: Utc::now(),
                                                        updated_at: Utc::now(),
                                                        color: None,
                                                    };
                                                    state_model.update(app, |state, _| {
                                                        state.projects.insert(project.id.clone(), project);
                                                        if let Err(e) = Storage::save_projects(&state.projects) {
                                                            eprintln!("Failed to save projects: {}", e);
                                                        }
                                                    });
                                                    input.update(app, |input, cx| {
                                                        input.set_value("", window, cx);
                                                    });
                                                }
                                            })
                                    })
                            )
                            // Projects list
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .scrollable(ScrollbarAxis::Vertical)
                                    // "All Chats" item
                                    .child({
                                        let state_model = state_model.clone();
                                        let is_selected = state.current_project_id.is_none();
                                        div()
                                            .id("project_all")
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_3()
                                            .py_2()
                                            .cursor_pointer()
                                            .when(is_selected, |this| this.bg(theme.accent))
                                            .hover(|style| style.bg(theme.button_bg))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .child("All Chats")
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.text_muted)
                                                    .child(format!("{}", chats.len()))
                                            )
                                            .on_click(move |_, _, app| {
                                                state_model.update(app, |state, _| {
                                                    state.current_project_id = None;
                                                });
                                            })
                                    })
                                    // "Unassigned" item
                                    .child({
                                        let state_model = state_model.clone();
                                        div()
                                            .id("project_unassigned")
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_3()
                                            .py_2()
                                            .cursor_pointer()
                                            .hover(|style| style.bg(theme.button_bg))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .italic()
                                                    .text_color(theme.text_secondary)
                                                    .child("Unassigned")
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.text_muted)
                                                    .child(format!("{}", unassigned_count))
                                            )
                                    })
                                    // Project items
                                    .children(
                                        projects.iter().map(|project| {
                                            let project_id = project.id.clone();
                                            let project_name = project.name.clone();
                                            let state_model = state_model.clone();
                                            let state_model_delete = state_model.clone();
                                            let is_selected = state.current_project_id.as_ref() == Some(&project_id);
                                            let chat_count = chats.iter()
                                                .filter(|c| c.project_id.as_ref() == Some(&project_id))
                                                .count();
                                            let theme_clone = theme.clone();

                                            div()
                                                .id(SharedString::from(format!("project_{}", project_id)))
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .px_3()
                                                .py_2()
                                                .cursor_pointer()
                                                .when(is_selected, |this| this.bg(theme.accent))
                                                .hover(|style| style.bg(theme_clone.button_bg))
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .text_sm()
                                                        .overflow_hidden()
                                                        .child(project_name)
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(theme.text_muted)
                                                                .child(format!("{}", chat_count))
                                                        )
                                                        .child({
                                                            let project_id = project_id.clone();
                                                            Button::new(SharedString::from(format!("del_proj_{}", project_id)))
                                                                .label("x")
                                                                .compact()
                                                                .on_click(move |_, _, app| {
                                                                    let project_id = project_id.clone();
                                                                    state_model_delete.update(app, |state, _| {
                                                                        // Unassign chats from this project
                                                                        for chat in state.chats.values_mut() {
                                                                            if chat.project_id.as_ref() == Some(&project_id) {
                                                                                chat.project_id = None;
                                                                            }
                                                                        }
                                                                        state.projects.remove(&project_id);
                                                                        if state.current_project_id.as_ref() == Some(&project_id) {
                                                                            state.current_project_id = None;
                                                                        }
                                                                        Storage::save_projects(&state.projects).ok();
                                                                        Storage::save_chats(&state.chats).ok();
                                                                    });
                                                                })
                                                        })
                                                )
                                                .on_click({
                                                    let project_id = project_id.clone();
                                                    move |_, _, app| {
                                                        let project_id = project_id.clone();
                                                        state_model.update(app, |state, _| {
                                                            state.current_project_id = Some(project_id);
                                                        });
                                                    }
                                                })
                                                .into_any_element()
                                        }).collect::<Vec<_>>()
                                    )
                            )
                    )
                    // Right column - Chats
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .overflow_hidden()
                            // Chats header
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .p_3()
                                    .border_b_1()
                                    .border_color(theme.border)
                                    .bg(theme.surface)
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text_secondary)
                                            .child("Chats")
                                    )
                            )
                            // Chats list
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .scrollable(ScrollbarAxis::Vertical)
                                    .p_2()
                                    .gap_1()
                                    .children(
                                        chats.iter().map(|chat| {
                                            let chat_id = chat.id.clone();
                                            let chat_title = chat.title.clone();
                                            let chat_project_id = chat.project_id.clone();
                                            let message_count = chat.messages.len();
                                            let state_model = state_model.clone();
                                            let state_model_delete = state_model.clone();
                                            let theme_clone = theme.clone();

                                            // Get project name
                                            let project_name = chat_project_id.as_ref()
                                                .and_then(|pid| state.projects.get(pid))
                                                .map(|p| p.name.clone());

                                            // Build project options for dropdown
                                            let projects_for_dropdown: Vec<(String, String)> = projects.iter()
                                                .map(|p| (p.id.clone(), p.name.clone()))
                                                .collect();

                                            div()
                                                .id(SharedString::from(format!("chat_{}", chat_id)))
                                                .flex()
                                                .items_center()
                                                .gap_3()
                                                .px_3()
                                                .py_2()
                                                .rounded_md()
                                                .bg(theme.surface)
                                                .border_1()
                                                .border_color(theme.border)
                                                .hover(|style| style.border_color(theme_clone.accent))
                                                // Chat title
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .overflow_hidden()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(theme.text_primary)
                                                                .child(chat_title)
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(theme.text_muted)
                                                                .child(format!("{} messages", message_count))
                                                        )
                                                )
                                                // Project assignment dropdown
                                                .child({
                                                    let state_model = state_model.clone();
                                                    let chat_id = chat_id.clone();
                                                    Button::new(SharedString::from(format!("proj_btn_{}", chat_id)))
                                                        .label(SharedString::from(
                                                            project_name.clone().unwrap_or_else(|| "Unassigned".to_string())
                                                        ))
                                                        .compact()
                                                        .dropdown_menu(move |menu, _window, _cx| {
                                                            let state_model = state_model.clone();
                                                            let chat_id = chat_id.clone();

                                                            // Add "Unassigned" option
                                                            let mut menu = menu.item({
                                                                let state_model = state_model.clone();
                                                                let chat_id = chat_id.clone();
                                                                gpui_component::menu::PopupMenuItem::new("Unassigned")
                                                                    .on_click(move |_, _, app| {
                                                                        let chat_id = chat_id.clone();
                                                                        state_model.update(app, |state, _| {
                                                                            if let Some(chat) = state.chats.get_mut(&chat_id) {
                                                                                chat.project_id = None;
                                                                                chat.updated_at = Utc::now();
                                                                            }
                                                                            Storage::save_chats(&state.chats).ok();
                                                                        });
                                                                    })
                                                            });

                                                            // Add project options
                                                            for (proj_id, proj_name) in projects_for_dropdown.iter() {
                                                                let state_model = state_model.clone();
                                                                let chat_id = chat_id.clone();
                                                                let proj_id = proj_id.clone();
                                                                menu = menu.item(
                                                                    gpui_component::menu::PopupMenuItem::new(proj_name.clone())
                                                                        .on_click(move |_, _, app| {
                                                                            let chat_id = chat_id.clone();
                                                                            let proj_id = proj_id.clone();
                                                                            state_model.update(app, |state, _| {
                                                                                if let Some(chat) = state.chats.get_mut(&chat_id) {
                                                                                    chat.project_id = Some(proj_id);
                                                                                    chat.updated_at = Utc::now();
                                                                                }
                                                                                Storage::save_chats(&state.chats).ok();
                                                                            });
                                                                        })
                                                                );
                                                            }
                                                            menu
                                                        })
                                                })
                                                // Delete button
                                                .child({
                                                    let chat_id = chat_id.clone();
                                                    Button::new(SharedString::from(format!("del_chat_{}", chat_id)))
                                                        .label("Delete")
                                                        .compact()
                                                        .on_click(move |_, _, app| {
                                                            let chat_id = chat_id.clone();
                                                            state_model_delete.update(app, |state, _| {
                                                                state.chats.remove(&chat_id);
                                                                if state.current_chat_id.as_ref() == Some(&chat_id) {
                                                                    state.current_chat_id = None;
                                                                }
                                                                Storage::save_chats(&state.chats).ok();
                                                            });
                                                        })
                                                })
                                                .into_any_element()
                                        }).collect::<Vec<_>>()
                                    )
                                    .when(chats.is_empty(), |this| {
                                        this.child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .flex_1()
                                                .text_color(theme.text_muted)
                                                .child("No chats yet")
                                        )
                                    })
                            )
                    )
            )
    }
}
