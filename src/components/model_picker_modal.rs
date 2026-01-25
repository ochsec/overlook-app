// Model Picker Modal Component - Model selection with categories and search
use crate::state::AppState;
use crate::storage::Storage;
use crate::types::ModelDescriptor;
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::{scroll::ScrollbarAxis, StyledExt};
use std::collections::HashSet;

pub struct ModelPickerModal {
    state: Entity<AppState>,
    search_input: Entity<InputState>,
    expanded_categories: HashSet<String>,
}

impl ModelPickerModal {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("Search models..."));
        Self {
            state,
            search_input,
            expanded_categories: HashSet::new(),
        }
    }

    fn toggle_category(&mut self, category: &str) {
        if self.expanded_categories.contains(category) {
            self.expanded_categories.remove(category);
        } else {
            self.expanded_categories.insert(category.to_string());
        }
    }
}

impl Render for ModelPickerModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Read the search input value and sync it to the state
        let search_query = self.search_input.read(cx).value().to_string();

        // Update state's model_search_query to keep filtering in sync
        self.state.update(cx, |state, _| {
            state.model_search_query = search_query;
        });

        let state = self.state.read(cx);
        let state_model = self.state.clone();

        // Extract needed values
        let theme = state.theme.clone();
        let grouped_models = state.get_filtered_grouped_models();
        let total_filtered: usize = grouped_models.iter().map(|(_, models)| models.len()).sum();
        let total_models = state.available_models.len();
        let current_model_id = state.current_model.clone();
        let favorite_models = state.favorite_models.clone();

        // Build category data with expansion state
        let categories: Vec<(String, Vec<ModelDescriptor>, bool)> = grouped_models
            .into_iter()
            .map(|(category, models)| {
                let expanded = self.expanded_categories.contains(&category);
                (category, models, expanded)
            })
            .collect();

        // Modal overlay
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0., 0., 0., 0.5))
            .child(
                // Modal container
                div()
                    .flex()
                    .flex_col()
                    .w(px(600.0))
                    .max_h(px(500.0))
                    .bg(theme.surface)
                    .rounded_lg()
                    .border_1()
                    .border_color(theme.border)
                    .shadow_lg()
                    // Header
                    .child(
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
                                    .text_color(theme.text_primary)
                                    .child("Select Model"),
                            )
                            .child({
                                let state_model = state_model.clone();
                                Button::new("close_modal").label("X").compact().on_click(
                                    move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.show_model_picker_modal = false;
                                            state.model_search_query.clear();
                                        });
                                    },
                                )
                            }),
                    )
                    // Search input
                    .child(
                        div().p_3().border_b_1().border_color(theme.border).child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(Input::new(&self.search_input).cleanable(true)),
                        ),
                    )
                    // Model count
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .text_sm()
                            .text_color(theme.text_muted)
                            .child(format!(
                                "Showing {} of {} models",
                                total_filtered, total_models
                            )),
                    )
                    // Favorites section (if any)
                    .when(!favorite_models.is_empty(), |this| {
                        let favorites: Vec<ModelDescriptor> = state
                            .available_models
                            .iter()
                            .filter(|m| favorite_models.contains(&m.id))
                            .cloned()
                            .collect();

                        if favorites.is_empty() {
                            this
                        } else {
                            let state_model = state_model.clone();
                            let current_model = current_model_id.clone();
                            let theme_clone = theme.clone();
                            this.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .px_4()
                                    .py_2()
                                    .bg(theme.background)
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text_secondary)
                                            .child("Favorites"),
                                    )
                                    .children(
                                        favorites
                                            .into_iter()
                                            .map(|model| {
                                                let state_model = state_model.clone();
                                                let model_id = model.id.clone();
                                                let is_selected = current_model == model_id;
                                                render_model_item(
                                                    model,
                                                    is_selected,
                                                    true,
                                                    state_model,
                                                    theme_clone.clone(),
                                                )
                                                .into_any_element()
                                            })
                                            .collect::<Vec<_>>(),
                                    ),
                            )
                        }
                    })
                    // Categories list
                    .child(
                        div().flex().flex_col().flex_1().overflow_hidden().child(
                            div()
                                .flex()
                                .flex_col()
                                .scrollable(ScrollbarAxis::Vertical)
                                .children(
                                    categories
                                        .into_iter()
                                        .map(|(category, models, expanded)| {
                                            let state_model = state_model.clone();
                                            let current_model = current_model_id.clone();
                                            let theme_clone = theme.clone();
                                            let favorite_models_clone = favorite_models.clone();
                                            let _category_clone = category.clone();
                                            let model_count = models.len();

                                            div()
                                                .flex()
                                                .flex_col()
                                                .px_4()
                                                .py_1()
                                                // Category header (clickable to toggle)
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_2()
                                                        .py_1()
                                                        .cursor_pointer()
                                                        .text_color(theme.text_secondary)
                                                        .child(div().text_sm().child(if expanded {
                                                            "v"
                                                        } else {
                                                            ">"
                                                        }))
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_weight(FontWeight::SEMIBOLD)
                                                                .child(format!(
                                                                    "{} ({})",
                                                                    category, model_count
                                                                )),
                                                        ), // Note: Click handler for expansion would go here
                                                           // For now, all categories are expanded by default
                                                )
                                                // Models in category (always shown for now)
                                                .children(
                                                    models
                                                        .into_iter()
                                                        .map(|model| {
                                                            let state_model = state_model.clone();
                                                            let model_id = model.id.clone();
                                                            let is_selected =
                                                                current_model == model_id;
                                                            let is_favorite = favorite_models_clone
                                                                .contains(&model_id);
                                                            render_model_item(
                                                                model,
                                                                is_selected,
                                                                is_favorite,
                                                                state_model,
                                                                theme_clone.clone(),
                                                            )
                                                            .into_any_element()
                                                        })
                                                        .collect::<Vec<_>>(),
                                                )
                                                .into_any_element()
                                        })
                                        .collect::<Vec<_>>(),
                                ),
                        ),
                    )
                    // Footer with cancel button
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .p_4()
                            .border_t_1()
                            .border_color(theme.border)
                            .child({
                                let state_model = state_model.clone();
                                Button::new("cancel_modal").label("Cancel").on_click(
                                    move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.show_model_picker_modal = false;
                                            state.model_search_query.clear();
                                        });
                                    },
                                )
                            }),
                    ),
            )
    }
}

/// Helper function to render a model item
fn render_model_item(
    model: ModelDescriptor,
    is_selected: bool,
    is_favorite: bool,
    state_model: Entity<AppState>,
    theme: crate::gpui_theme::GpuiTheme,
) -> impl IntoElement {
    let model_id = model.id.clone();
    let model_name = model.name.clone();
    let context_length = model.context_length;

    div()
        .id(SharedString::from(format!("model_item_{}", model_id)))
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .when(is_selected, |this| this.bg(theme.accent))
        .hover(|style| style.bg(theme.button_bg))
        .child(
            // Favorite indicator
            div().text_sm().child(if is_favorite { "*" } else { " " }),
        )
        .child(
            // Model name
            div()
                .flex_1()
                .text_sm()
                .text_color(if is_selected {
                    theme.text_primary
                } else {
                    theme.text_secondary
                })
                .child(model_name),
        )
        .when(context_length.is_some(), |this| {
            this.child(
                // Context length badge
                div()
                    .text_xs()
                    .px_2()
                    .py_px()
                    .rounded_md()
                    .bg(theme.background)
                    .text_color(theme.text_muted)
                    .child(format!("{}k", context_length.unwrap() / 1000)),
            )
        })
        .on_click({
            let model_id = model_id.clone();
            move |_, _, app| {
                let model_id = model_id.clone();
                state_model.update(app, |state, _| {
                    state.current_model = model_id.clone();
                    state.show_model_picker_modal = false;
                    state.model_search_query.clear();
                    // Save the last used model
                    if let Err(e) = Storage::save_last_model(&model_id) {
                        eprintln!("Failed to save last model: {}", e);
                    }
                });
            }
        })
}
