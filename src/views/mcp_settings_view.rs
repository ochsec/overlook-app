// MCP Settings View - Manage MCP servers and tools
use crate::gpui_theme::GpuiTheme;
use crate::mcp::{save_mcp_config, McpServerConfig};
use crate::state::AppState;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};
use gpui_component::switch::Switch;
use gpui_component::{scroll::ScrollbarAxis, StyledExt};
// Arc is not used in this file

pub struct McpSettingsView {
    state_model: Entity<AppState>,
    // Input states for adding a new server
    new_name_input: Entity<InputState>,
    new_command_input: Entity<InputState>,
    new_args_input: Entity<InputState>,
}

impl McpSettingsView {
    pub fn new(state_model: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let new_name_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Server Name (e.g. brave-search)"));
        let new_command_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Command (e.g. npx, uvx)"));
        let new_args_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Arguments (space separated)"));

        Self {
            state_model,
            new_name_input,
            new_command_input,
            new_args_input,
        }
    }

    fn render_header(&self, theme: &GpuiTheme, font_size: f32) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .p_4()
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_size(px(font_size + 6.0))
                    .font_weight(FontWeight::BOLD)
                    .child("MCP Servers"),
            )
            .child(
                Button::new("connect_all_mcp")
                    .label("Connect All")
                    .on_click({
                        let state_model = self.state_model.clone();
                        move |_, _, cx| {
                            state_model.read(cx).connect_mcp_servers(&mut cx.to_async());
                        }
                    }),
            )
    }

    fn render_add_server_section(&self, theme: &GpuiTheme, font_size: f32) -> impl IntoElement {
        let state_model = self.state_model.clone();
        let new_name_input = self.new_name_input.clone();
        let new_command_input = self.new_command_input.clone();
        let new_args_input = self.new_args_input.clone();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_size(px(font_size + 2.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Add New Server"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Input::new(&new_name_input))
                    .child(Input::new(&new_command_input))
                    .child(Input::new(&new_args_input)),
            )
            .child(
                div().flex().justify_end().child(
                    Button::new("add_server_btn")
                        .label("Add Server")
                        .primary()
                        .on_click(move |_, window, cx| {
                            let name = new_name_input.read(cx).value().to_string();
                            let command = new_command_input.read(cx).value().to_string();
                            let args_str = new_args_input.read(cx).value().to_string();

                            if name.is_empty() || command.is_empty() {
                                return;
                            }

                            let args = args_str.split_whitespace().map(|s| s.to_string()).collect();
                            let new_server = McpServerConfig::new(name, command, args);

                            state_model.update(cx, |state, _| {
                                let mut config = state.mcp_manager.config().clone();
                                config.servers.push(new_server);
                                state.mcp_manager.set_config(config.clone());
                                let _ = save_mcp_config(&config);
                            });

                            // Clear inputs
                            new_name_input.update(cx, |state, cx| state.set_value("", window, cx));
                            new_command_input
                                .update(cx, |state, cx| state.set_value("", window, cx));
                            new_args_input.update(cx, |state, cx| state.set_value("", window, cx));
                        }),
                ),
            )
    }

    fn render_server_list(
        &self,
        cx: &mut Context<Self>,
        theme: &GpuiTheme,
        font_size: f32,
    ) -> impl IntoElement {
        let state = self.state_model.read(cx);
        let mcp_config = state.mcp_manager.config();
        let state_model = self.state_model.clone();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_size(px(font_size + 2.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Configured Servers"),
            )
            .children(mcp_config.servers.iter().enumerate().map(|(_idx, server)| {
                let server_name = server.name.clone();
                let state_model = state_model.clone();
                let enabled = server.enabled;

                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .rounded_md()
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .child(server_name.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(font_size - 2.0))
                                    .text_color(theme.text_secondary)
                                    .child(format!("{} {}", server.command, server.args.join(" "))),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                Switch::new(SharedString::from(format!("enable_{}", server_name)))
                                    .checked(enabled)
                                    .on_click({
                                        let state_model = state_model.clone();
                                        let server_name = server_name.clone();
                                        move |checked, _, cx| {
                                            state_model.update(cx, |state, _| {
                                                let mut config = state.mcp_manager.config().clone();
                                                if let Some(s) = config
                                                    .servers
                                                    .iter_mut()
                                                    .find(|s| s.name == server_name)
                                                {
                                                    s.enabled = *checked;
                                                    state.mcp_manager.set_config(config.clone());
                                                    let _ = save_mcp_config(&config);
                                                }
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new(SharedString::from(format!("remove_{}", server_name)))
                                    .label("Remove")
                                    .on_click(move |_, _, cx| {
                                        state_model.update(cx, |state, _| {
                                            let mut config = state.mcp_manager.config().clone();
                                            config.servers.retain(|s| s.name != server_name);
                                            state.mcp_manager.set_config(config.clone());
                                            let _ = save_mcp_config(&config);
                                        });
                                    }),
                            ),
                    )
            }))
    }
}

impl Render for McpSettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state_model.read(cx);
        let theme = state.theme.clone();
        let font_size = state.font_size;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.text_primary)
            .child(self.render_header(&theme, font_size))
            .child(
                div().flex_1().overflow_hidden().child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_6()
                        .p_4()
                        .scrollable(ScrollbarAxis::Vertical)
                        .child(self.render_add_server_section(&theme, font_size))
                        .child(self.render_server_list(cx, &theme, font_size)),
                ),
            )
    }
}
