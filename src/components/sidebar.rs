// Sidebar Component - Projects and Chat list
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{StyledExt, scroll::ScrollbarAxis};
use crate::state::AppState;

pub struct Sidebar {
    state: Entity<AppState>,
}

impl Sidebar {
    pub fn new(state: Entity<AppState>) -> Self {
        Self { state }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();

        // Extract theme values
        let sidebar_bg = state.theme.sidebar_background;
        let border_color = state.theme.border;
        let text_primary = state.theme.text_primary;
        let text_secondary = state.theme.text_secondary;
        let separator_color = state.theme.separator;

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
                            .text_lg()
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
                                state_model.update(app, |_state, _| {
                                    println!("New Chat clicked");
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
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(text_secondary)
                            .child("Projects")
                    )
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
                            .child({
                                let state_model = state_model.clone();
                                Button::new("add_project")
                                    .label("+")
                                    .on_click(move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.show_project_modal = true;
                                        });
                                    })
                            })
                    )
                    .children(
                        projects.iter().map(|(proj_id, proj_name)| {
                            let project_id = proj_id.clone();
                            let project_name = proj_name.clone();
                            let is_selected = current_project_id.as_ref() == Some(&project_id);
                            let state_model = state_model.clone();

                            let mut btn = Button::new(SharedString::from(format!("project_{}", project_id)))
                                .label(SharedString::from(project_name))
                                .on_click(move |_, _, app| {
                                    let pid = project_id.clone();
                                    state_model.update(app, |state, _| {
                                        state.current_project_id = Some(pid);
                                    });
                                });

                            if is_selected {
                                btn = btn.primary();
                            }

                            btn.into_any_element()
                        }).collect::<Vec<_>>()
                    )
            )
            // Separator
            .child(div().h(px(1.0)).bg(separator_color))
            // Chats list
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p_2()
                            .gap_1()
                            .scrollable(ScrollbarAxis::Vertical)
                            .children(
                                chats.iter().map(|(chat_id, chat_title, _)| {
                                    let cid = chat_id.clone();
                                    let title = chat_title.clone();
                                    let is_selected = current_chat_id.as_ref() == Some(&cid);
                                    let state_model = state_model.clone();

                                    let mut btn = Button::new(SharedString::from(format!("chat_{}", cid)))
                                        .label(SharedString::from(title))
                                        .w_full()
                                        .on_click(move |_, _, app| {
                                            let chat_id = cid.clone();
                                            state_model.update(app, |state, _| {
                                                state.current_chat_id = Some(chat_id);
                                            });
                                        });

                                    if is_selected {
                                        btn = btn.primary();
                                    }

                                    btn.into_any_element()
                                }).collect::<Vec<_>>()
                            )
                    )
            )
    }
}
