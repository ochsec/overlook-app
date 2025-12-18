// Sidebar Component - Projects and Chat list
use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{InputState, Input};
use gpui_component::{StyledExt, scroll::ScrollbarAxis};
use chrono::Utc;
use uuid::Uuid;
use crate::state::AppState;
use crate::storage::Storage;
use crate::types::Project;

pub struct Sidebar {
    state: Entity<AppState>,
    new_project_input: Option<Entity<InputState>>,
    pending_delete_chat_id: Option<String>,
}

impl Sidebar {
    pub fn new(state: Entity<AppState>) -> Self {
        Self {
            state,
            new_project_input: None,
            pending_delete_chat_id: None,
        }
    }

    fn ensure_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.new_project_input.is_none() {
            self.new_project_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Project name...")
            }));
        }
    }

    fn request_delete(&mut self, chat_id: String, cx: &mut Context<Self>) {
        let confirm_required = self.state.read(cx).confirm_delete_chat;
        if confirm_required {
            self.pending_delete_chat_id = Some(chat_id);
            cx.notify();
        } else {
            self.delete_chat(chat_id, cx);
        }
    }

    fn cancel_delete(&mut self, cx: &mut Context<Self>) {
        self.pending_delete_chat_id = None;
        cx.notify();
    }

    fn delete_chat(&mut self, chat_id: String, cx: &mut Context<Self>) {
        self.state.update(cx, |state, _| {
            // Remove the chat
            state.chats.remove(&chat_id);

            // Clear current_chat_id if we deleted the active chat
            if state.current_chat_id.as_ref() == Some(&chat_id) {
                state.current_chat_id = None;
            }

            // Save to storage
            if let Err(e) = Storage::save_chats(&state.chats) {
                eprintln!("Failed to save chats after deletion: {}", e);
            }
        });

        self.pending_delete_chat_id = None;
        cx.notify();
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure input is initialized
        self.ensure_input(window, cx);

        let state = self.state.read(cx);
        let state_model = self.state.clone();

        // Extract theme values
        let sidebar_bg = state.theme.sidebar_background;
        let border_color = state.theme.border;
        let text_primary = state.theme.text_primary;
        let text_secondary = state.theme.text_secondary;
        let separator_color = state.theme.separator;
        let font_size = state.font_size;

        // Extract project modal state
        let show_project_input = state.show_project_modal;

        // Extract project data
        let is_all_selected = state.current_project_id.is_none();
        let current_project_id = state.current_project_id.clone();
        let projects: Vec<_> = state.projects.values()
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect();

        // Extract chat data
        let current_chat_id = state.current_chat_id.clone();
        let mut chats: Vec<_> = state.chats.values()
            .filter(|chat| {
                if let Some(current_proj_id) = &current_project_id {
                    chat.project_id.as_ref() == Some(current_proj_id)
                } else {
                    true
                }
            })
            .map(|c| (c.id.clone(), c.title.clone(), c.updated_at))
            .collect();
        chats.sort_by(|a, b| b.2.cmp(&a.2));

        div()
            .flex()
            .flex_col()
            .w(px(300.0))
            .h_full()
            .bg(sidebar_bg)
            .border_r_1()
            .border_color(border_color)
            // Header
            .child(
                div()
                    .p_4()
                    .child(
                        div()
                            .text_size(px(font_size + 4.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(text_primary)
                            .child("Overlook Chat")
                    )
            )
            // Separator
            .child(div().h(px(1.0)).bg(separator_color))
            // New Chat button
            .child(
                div()
                    .p_2()
                    .child({
                        let state_model = state_model.clone();
                        Button::new("new_chat")
                            .label("+ New Chat")
                            .w_full()
                            .on_click(move |_, _, app| {
                                state_model.update(app, |state, _| {
                                    // Clear current chat to start fresh
                                    state.current_chat_id = None;
                                });
                            })
                    })
            )
            // Separator
            .child(div().h(px(1.0)).bg(separator_color))
            // Projects section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p_2()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(font_size - 2.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(text_secondary)
                                    .child("Projects")
                            )
                            .child({
                                let state_model = state_model.clone();
                                Button::new("add_project")
                                    .label("+")
                                    .compact()
                                    .on_click(move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.show_project_modal = !state.show_project_modal;
                                        });
                                    })
                            })
                    )
                    // New project input (shown when + is clicked)
                    .when(show_project_input, |this| {
                        if let Some(input) = &self.new_project_input {
                            let input_entity = input.clone();
                            let state_model = state_model.clone();
                            this.child(
                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .flex_1()
                                            .child(Input::new(&input_entity))
                                    )
                                    .child({
                                        let input_entity = input_entity.clone();
                                        let state_model = state_model.clone();
                                        Button::new("create_project")
                                            .label("Create")
                                            .compact()
                                            .on_click(move |_, window, app| {
                                                let name = input_entity.read(app).value().to_string();
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
                                                        state.show_project_modal = false;
                                                        if let Err(e) = Storage::save_projects(&state.projects) {
                                                            eprintln!("Failed to save projects: {}", e);
                                                        }
                                                    });
                                                    input_entity.update(app, |input, cx| {
                                                        input.set_value("", window, cx);
                                                    });
                                                }
                                            })
                                    })
                            )
                        } else {
                            this
                        }
                    })
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child({
                                let state_model = state_model.clone();
                                let mut btn = Button::new("all_chats")
                                    .label("All Chats")
                                    .on_click(move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.current_project_id = None;
                                        });
                                    });
                                if is_all_selected {
                                    btn = btn.primary();
                                }
                                btn
                            })
                    )
                    .children(
                        projects.iter().map(|(proj_id, proj_name)| {
                            let project_id = proj_id.clone();
                            let project_name = proj_name.clone();
                            let is_selected = current_project_id.as_ref() == Some(&project_id);
                            let state_model = state_model.clone();
                            let accent_color = state.theme.accent;
                            let hover_bg = state.theme.button_bg;

                            div()
                                .id(SharedString::from(format!("project_{}", project_id)))
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .cursor_pointer()
                                .when(is_selected, |this| this.bg(accent_color))
                                .hover(|style| style.bg(hover_bg))
                                // Folder icon
                                .child(
                                    div()
                                        .text_color(text_secondary)
                                        .child("[P]")
                                )
                                // Project name
                                .child(
                                    div()
                                        .flex_1()
                                        .text_color(text_primary)
                                        .font_weight(FontWeight::MEDIUM)
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .child(project_name)
                                )
                                .on_click(move |_, _, app| {
                                    let pid = project_id.clone();
                                    state_model.update(app, |state, _| {
                                        state.current_project_id = Some(pid);
                                    });
                                })
                                .into_any_element()
                        }).collect::<Vec<_>>()
                    )
            )
            // Separator
            .child(div().h(px(1.0)).bg(separator_color))
            // Chats list
            .child({
                let pending_delete = self.pending_delete_chat_id.clone();
                let accent_color = state.theme.accent;
                let hover_bg = state.theme.button_bg;
                let danger_color = state.theme.danger;

                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .scrollable(ScrollbarAxis::Vertical)
                            .children(
                                chats.iter().map(|(chat_id, chat_title, _)| {
                                    let cid = chat_id.clone();
                                    let title = chat_title.clone();
                                    let is_selected = current_chat_id.as_ref() == Some(&cid);
                                    let is_pending_delete = pending_delete.as_ref() == Some(&cid);
                                    let state_model = state_model.clone();

                                    if is_pending_delete {
                                        // Show delete confirmation (matching chat row style)
                                        let cid_confirm = cid.clone();
                                        div()
                                            .id(SharedString::from(format!("chat_{}", cid)))
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_3()
                                            .py_2()
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .text_sm()
                                                    .text_color(text_primary)
                                                    .child("Delete?")
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .id(SharedString::from(format!("confirm_{}", cid)))
                                                            .text_sm()
                                                            .text_color(text_secondary)
                                                            .cursor_pointer()
                                                            .hover(|s| s.text_color(danger_color))
                                                            .child("[y]")
                                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                                this.delete_chat(cid_confirm.clone(), cx);
                                                            }))
                                                    )
                                                    .child(
                                                        div()
                                                            .id(SharedString::from(format!("cancel_{}", cid)))
                                                            .text_sm()
                                                            .text_color(text_secondary)
                                                            .cursor_pointer()
                                                            .hover(|s| s.text_color(accent_color))
                                                            .child("[n]")
                                                            .on_click(cx.listener(|this, _, _, cx| {
                                                                this.cancel_delete(cx);
                                                            }))
                                                    )
                                            )
                                            .into_any_element()
                                    } else {
                                        // Normal chat row with delete button
                                        let cid_select = cid.clone();
                                        let cid_delete = cid.clone();
                                        div()
                                            .id(SharedString::from(format!("chat_{}", cid)))
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_3()
                                            .py_2()
                                            .cursor_pointer()
                                            .when(is_selected, |this| this.bg(accent_color))
                                            .hover(|style| style.bg(hover_bg))
                                            .on_click({
                                                let state_model = state_model.clone();
                                                move |_, _, app| {
                                                    let chat_id = cid_select.clone();
                                                    state_model.update(app, |state, _| {
                                                        state.current_chat_id = Some(chat_id);
                                                    });
                                                }
                                            })
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .text_sm()
                                                    .text_color(text_primary)
                                                    .overflow_hidden()
                                                    .text_ellipsis()
                                                    .child(title)
                                            )
                                            .child(
                                                div()
                                                    .id(SharedString::from(format!("delete_{}", cid)))
                                                    .text_sm()
                                                    .text_color(text_secondary)
                                                    .cursor_pointer()
                                                    .hover(|s| s.text_color(danger_color))
                                                    .child("[x]")
                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                        this.request_delete(cid_delete.clone(), cx);
                                                    }))
                                            )
                                            .into_any_element()
                                    }
                                }).collect::<Vec<_>>()
                            )
                    )
            })
    }
}
