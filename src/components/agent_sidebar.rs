// Agent Sidebar - Thread list for Hermes Agent conversations
use crate::state::AppState;
use crate::storage::Storage;
use crate::types::AgentThread;
use chrono::Utc;
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollbarAxis;
use gpui_component::StyledExt;
use uuid::Uuid;

pub struct AgentSidebar {
    state: Entity<AppState>,
    chats_scroll: ScrollHandle,
}

impl AgentSidebar {
    pub fn new(state: Entity<AppState>) -> Self {
        Self {
            state,
            chats_scroll: ScrollHandle::default(),
        }
    }
}

impl Render for AgentSidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let sidebar_bg = state.theme.sidebar_background;
        let border_color = state.theme.border;
        let text_primary = state.theme.text_primary;
        let text_secondary = state.theme.text_secondary;
        let text_muted = state.theme.text_muted;
        let separator_color = state.theme.separator;
        let accent_color = state.theme.accent;
        let hover_bg = state.theme.button_bg;
        let danger_color = state.theme.danger;
        let font_size = state.font_size;
        let state_model = self.state.clone();

        // Collect and sort threads by updated_at (newest first)
        let current_thread_id = state.current_agent_thread_id.clone();
        let mut threads: Vec<_> = state
            .agent_threads
            .values()
            .map(|t| (t.id.clone(), t.title.clone(), t.updated_at))
            .collect();
        threads.sort_by(|a, b| b.2.cmp(&a.2));

        div()
            .flex()
            .flex_col()
            .w(px(280.0))
            .h_full()
            .bg(sidebar_bg)
            .border_r_1()
            .border_color(border_color)
            // Header
            .child(
                div().p_4().child(
                    div()
                        .text_size(px(font_size + 4.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_primary)
                        .child("Agent Threads"),
                ),
            )
            // Separator
            .child(div().h(px(1.0)).bg(separator_color))
            // New Thread button
            .child(div().p_2().child({
                let state_model = state_model.clone();
                Button::new("new_agent_thread")
                    .label("+ New Thread")
                    .w_full()
                    .on_click(move |_, _, app| {
                        let thread_id = Uuid::new_v4().to_string();
                        let session_id = Uuid::new_v4().to_string();
                        let now = Utc::now();
                        let thread = AgentThread {
                            id: thread_id.clone(),
                            title: "New Thread".to_string(),
                            messages: Vec::new(),
                            session_id,
                            created_at: now,
                            updated_at: now,
                        };
                        state_model.update(app, |state, _| {
                            state.agent_threads.insert(thread_id.clone(), thread);
                            state.current_agent_thread_id = Some(thread_id);
                            if let Err(e) = Storage::save_agent_threads(&state.agent_threads) {
                                eprintln!("Failed to save agent threads: {}", e);
                            }
                        });
                    })
            }))
            // Separator
            .child(div().h(px(1.0)).bg(separator_color))
            // Thread list
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .id("agent_threads_scroll")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h_0()
                            .p_2()
                            .gap_1()
                            .track_scroll(&self.chats_scroll)
                            .scrollable(ScrollbarAxis::Vertical)
                            .children(threads.iter().map(|(thread_id, title, _)| {
                                let tid = thread_id.clone();
                                let tid_select = thread_id.clone();
                                let tid_delete = thread_id.clone();
                                let is_selected = current_thread_id.as_ref() == Some(&tid);
                                let state_model = state_model.clone();

                                div()
                                    .id(SharedString::from(format!("thread_{}", tid)))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .when(is_selected, |this| this.bg(accent_color))
                                    .hover(|style| style.bg(hover_bg))
                                    .on_click({
                                        let state_model = state_model.clone();
                                        move |_, _, app| {
                                            state_model.update(app, |state, _| {
                                                state.current_agent_thread_id = Some(tid_select.clone());
                                            });
                                        }
                                    })
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_size(px(font_size))
                                            .text_color(if is_selected { gpui::white() } else { text_primary })
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .child(title.clone()),
                                    )
                                    .child(
                                        div()
                                            .id(SharedString::from(format!("del_thread_{}", tid_delete)))
                                            .text_size(px(font_size - 2.0))
                                            .text_color(if is_selected { gpui::white() } else { text_muted })
                                            .cursor_pointer()
                                            .hover(|s| s.text_color(danger_color))
                                            .child("×")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                let tid = tid_delete.clone();
                                                this.state.update(cx, |state, _| {
                                                    state.agent_threads.remove(&tid);
                                                    if state.current_agent_thread_id.as_ref() == Some(&tid) {
                                                        state.current_agent_thread_id = None;
                                                    }
                                                    if let Err(e) = Storage::save_agent_threads(&state.agent_threads) {
                                                        eprintln!("Failed to save agent threads: {}", e);
                                                    }
                                                });
                                            })),
                                    )
                                    .into_any_element()
                            })),
                    ),
            )
    }
}