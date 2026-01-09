//! AGUI Desktop - Main entry point

use agui_desktop::{
    app::AguiApp,
    config::AppConfig,
    hot_reload::HotReloadWatcher,
    layout::{LayoutState, Pane},
    logging, metrics, VERSION,
    protocol::ArtifactOpen,
    renderer::{parse_component, render_component, Component, FormAction, FormState, RenderContext},
    stage::{
        StageState, render_stage_pane, render_artifact_status_bar,
        components::tabs::TabItem,
    },
    stream::{
        StreamTimeline, StreamItem, StreamContent,
        UserMessage, AgentMessage, ReasoningBlock, ToolCallBlock, ToolCallStatus,
        PlanBlock, PlanItem, PlanItemStatus, ApprovalBlock, ApprovalAction,
        ApprovalActionVariant, StatusBlock,
        components::render_stream_timeline,
    },
};
use std::sync::Arc;
use gpui::{
    actions, div, prelude::*, px, rgb, rgba, size, App, Application, Bounds, Context, KeyBinding,
    Window, WindowBounds, WindowOptions,
};

// Define actions for keyboard shortcuts
actions!(
    agui,
    [
        Quit,
        FocusContextRail,
        FocusStream,
        FocusStage,
        FocusNextPane,
        FocusPreviousPane,
        ToggleContextRail,
        ToggleStage,
    ]
);

/// Main AGUI window view
struct AguiWindow {
    app: AguiApp,
    layout: LayoutState,
    /// Form state for dynamic UI components
    form_state: FormState,
    /// Currently rendered component schema (if any)
    rendered_component: Option<Component>,
    /// Stream timeline for conversation history
    stream_timeline: StreamTimeline,
    /// Stage state for artifact workspace
    stage_state: StageState,
}

impl AguiWindow {
    fn new() -> Self {
        let mut window = Self {
            app: AguiApp::new(),
            layout: LayoutState::new(),
            form_state: FormState::new(),
            rendered_component: None,
            stream_timeline: StreamTimeline::new(),
            stage_state: StageState::new(),
        };

        // Add demo items to showcase the stream timeline
        window.add_demo_items();
        // Add demo artifacts to showcase the stage
        window.add_demo_artifacts();
        window
    }

    /// Add demo artifacts to the stage for testing
    fn add_demo_artifacts(&mut self) {
        // Add a Rust code artifact
        self.stage_state.open_artifact(&ArtifactOpen {
            id: "artifact-1".to_string(),
            title: "auth.rs".to_string(),
            content: r#"//! Authentication module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl AuthConfig {
    /// Create a new auth configuration
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: String::new(),
            redirect_uri: String::new(),
            scopes: Vec::new(),
        }
    }

    /// Set the client secret
    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = secret.into();
        self
    }
}

/// Token storage
pub struct TokenStore {
    tokens: HashMap<String, Token>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
"#.to_string(),
            content_type: "code".to_string(),
            read_only: false,
            language: Some("rust".to_string()),
        });

        // Add a config file (read-only)
        self.stage_state.open_artifact(&ArtifactOpen {
            id: "artifact-2".to_string(),
            title: "config.toml".to_string(),
            content: r#"[package]
name = "agui-desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = "0.2"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
"#.to_string(),
            content_type: "toml".to_string(),
            read_only: true,
            language: Some("toml".to_string()),
        });
    }

    /// Add demo items to the stream timeline for testing
    fn add_demo_items(&mut self) {
        // User message
        self.stream_timeline.push(StreamItem::new(
            "msg-1",
            StreamContent::UserMessage(
                UserMessage::new("Help me implement a new feature for user authentication")
                    .with_sender("Developer"),
            ),
        ));

        // Agent response with reasoning
        self.stream_timeline.push(StreamItem::new(
            "reasoning-1",
            StreamContent::Reasoning(
                ReasoningBlock::new(
                    "I need to understand the current authentication setup first. Let me check:\n\
                     1. What auth library is being used?\n\
                     2. Are there existing user models?\n\
                     3. What's the session management approach?\n\n\
                     Based on the codebase structure, I should look at the auth/ directory."
                )
                .with_summary("Planning authentication implementation"),
            ),
        ));

        // Tool call - searching
        self.stream_timeline.push(StreamItem::new(
            "tool-1",
            StreamContent::ToolCall(ToolCallBlock {
                call_id: "tc-001".to_string(),
                tool_name: "Grep".to_string(),
                parameters: serde_json::json!({
                    "pattern": "authentication|auth|login",
                    "path": "src/"
                }),
                status: ToolCallStatus::Completed,
                result: Some(serde_json::json!({
                    "matches": 15,
                    "files": ["src/auth/mod.rs", "src/auth/session.rs", "src/routes/login.rs"]
                })),
                error: None,
                duration_ms: Some(234),
                progress: None,
                expanded: false,
            }),
        ));

        // Agent response
        self.stream_timeline.push(StreamItem::new(
            "msg-2",
            StreamContent::AgentMessage(
                AgentMessage::new(
                    "claude",
                    "I've analyzed your codebase and found the authentication system. Here's my plan:\n\n\
                     ## Implementation Plan\n\n\
                     I'll implement OAuth2 authentication with the following components:\n\n\
                     ```rust\n\
                     pub struct AuthConfig {\n\
                         pub client_id: String,\n\
                         pub client_secret: String,\n\
                         pub redirect_uri: String,\n\
                     }\n\
                     ```\n\n\
                     - Add OAuth2 provider configuration\n\
                     - Implement token refresh logic\n\
                     - Add session middleware",
                )
                .with_name("Claude"),
            ),
        ));

        // Plan card
        self.stream_timeline.push(StreamItem::new(
            "plan-1",
            StreamContent::Plan(
                PlanBlock::new("Authentication Implementation")
                    .with_items(vec![
                        PlanItem {
                            id: "p1".to_string(),
                            description: "Add OAuth2 dependencies to Cargo.toml".to_string(),
                            status: PlanItemStatus::Completed,
                            children: vec![],
                        },
                        PlanItem {
                            id: "p2".to_string(),
                            description: "Create AuthConfig struct".to_string(),
                            status: PlanItemStatus::Completed,
                            children: vec![],
                        },
                        PlanItem {
                            id: "p3".to_string(),
                            description: "Implement OAuth2 flow".to_string(),
                            status: PlanItemStatus::InProgress,
                            children: vec![
                                PlanItem::new("p3a", "Authorization redirect"),
                                PlanItem::new("p3b", "Callback handler"),
                                PlanItem::new("p3c", "Token exchange"),
                            ],
                        },
                        PlanItem::new("p4", "Add session middleware"),
                        PlanItem::new("p5", "Write integration tests"),
                    ]),
            ),
        ));

        // Approval gate
        self.stream_timeline.push(StreamItem::new(
            "approval-1",
            StreamContent::Approval(ApprovalBlock {
                title: "Confirm file modifications".to_string(),
                description: Some("I'm about to modify the following files. Please review and approve.".to_string()),
                content: Some(
                    "Modified files:\n\
                     - src/auth/oauth2.rs (new file)\n\
                     - src/auth/mod.rs (add module)\n\
                     - src/routes/mod.rs (add routes)\n\
                     - Cargo.toml (add dependencies)".to_string()
                ),
                content_type: None,
                actions: vec![
                    ApprovalAction {
                        id: "approve".to_string(),
                        label: "Approve".to_string(),
                        variant: ApprovalActionVariant::Primary,
                        payload: None,
                    },
                    ApprovalAction {
                        id: "reject".to_string(),
                        label: "Reject".to_string(),
                        variant: ApprovalActionVariant::Destructive,
                        payload: None,
                    },
                ],
                resolution: None,
                blocking: true,
            }),
        ));

        // Status update
        self.stream_timeline.push(StreamItem::new(
            "status-1",
            StreamContent::StatusUpdate(
                StatusBlock::progress("Writing OAuth2 implementation...", 45),
            ),
        ));
    }

    /// Handle a render request by parsing the component schema
    pub fn handle_render_request(&mut self, schema_json: &serde_json::Value) -> Result<(), serde_json::Error> {
        let component = parse_component(schema_json)?;
        self.rendered_component = Some(component);
        Ok(())
    }

    /// Handle a form action from rendered components
    fn handle_form_action(&mut self, action: FormAction) {
        tracing::info!("Form action: {:?}", action);
        // In a real implementation, this would send a UserAction event to the orchestrator
        // For now, just log the action
    }
}

impl Render for AguiWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Colors
        let bg_dark = rgb(0x1e1e1e);
        let bg_panel = rgb(0x252526);
        let border_color = rgb(0x3c3c3c);
        let border_focused = rgb(0x007acc); // Blue highlight for focused pane
        let text_primary = rgb(0xcccccc);
        let text_secondary = rgb(0x808080);
        let text_focused = rgb(0x3794ff); // Blue for focused pane label

        // Status Bar (top)
        let status_bar = div()
            .flex()
            .h(px(28.0))
            .w_full()
            .bg(rgb(0x007acc))
            .px_4()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        div()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(format!("AGUI Desktop v{}", VERSION)),
                    )
                    .child(
                        div()
                            .text_color(rgb(0xffffffcc))
                            .text_sm()
                            .child(format!("Focus: {}", self.layout.focused_pane.name())),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        div()
                            .text_color(rgb(0xffffffcc))
                            .text_xs()
                            .child(self.app.connection_status()),
                    )
                    .child(
                        div()
                            .text_color(rgb(0xffffffaa))
                            .text_xs()
                            .child("Ctrl+1/2/3: Switch panes | Ctrl+[/]: Toggle panels"),
                    ),
            );

        // Context Rail (Zone A) - Left panel
        let context_rail_focused = self.layout.is_focused(Pane::ContextRail);
        let context_rail = if self.layout.context_rail_collapsed {
            // Collapsed state - show a thin clickable strip
            div()
                .flex()
                .flex_col()
                .w(px(24.0))
                .h_full()
                .bg(bg_panel)
                .border_r_1()
                .border_color(border_color)
                .items_center()
                .pt_3()
                .cursor_pointer()
                .child(
                    div()
                        .text_color(text_secondary)
                        .text_xs()
                        .child("▶"),
                )
                .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, _cx| {
                    this.layout.toggle_context_rail();
                }))
        } else {
            div()
                .flex()
                .flex_col()
                .w(self.layout.context_rail_pixels())
                .h_full()
                .bg(bg_panel)
                .border_r_1()
                .border_color(if context_rail_focused {
                    border_focused
                } else {
                    border_color
                })
                .when(context_rail_focused, |el| el.border_r_2())
                .p_3()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .mb_3()
                        .child(
                            div()
                                .text_color(if context_rail_focused {
                                    text_focused
                                } else {
                                    text_secondary
                                })
                                .text_sm()
                                .font_weight(if context_rail_focused {
                                    gpui::FontWeight::BOLD
                                } else {
                                    gpui::FontWeight::NORMAL
                                })
                                .child("Context Rail"),
                        )
                        .child(
                            div()
                                .text_color(text_secondary)
                                .text_xs()
                                .child("Ctrl+1"),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .text_color(text_secondary)
                        .text_xs()
                        .child("Sessions, history, and context will appear here."),
                )
                .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, _cx| {
                    this.layout.focus_pane(Pane::ContextRail);
                }))
        };

        // Stream (Zone B) - Center panel with timeline
        let stream_focused = self.layout.is_focused(Pane::Stream);
        let stream_items = self.stream_timeline.state.items();
        let stream_empty = stream_items.is_empty();
        let selected_id = self.stream_timeline.state.selected().cloned();

        let stream = div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(bg_dark)
            .border_l_1()
            .border_r_1()
            .border_color(if stream_focused {
                border_focused
            } else {
                border_color
            })
            .when(stream_focused, |el| el.border_l_2().border_r_2())
            // Header
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border_color)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_color(if stream_focused {
                                        text_focused
                                    } else {
                                        text_secondary
                                    })
                                    .text_sm()
                                    .font_weight(if stream_focused {
                                        gpui::FontWeight::BOLD
                                    } else {
                                        gpui::FontWeight::NORMAL
                                    })
                                    .child("Stream"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_secondary)
                                    .child(format!("{} items", stream_items.len())),
                            ),
                    )
                    .child(
                        div()
                            .text_color(text_secondary)
                            .text_xs()
                            .child("Ctrl+2"),
                    ),
            )
            // Timeline content
            .child(
                div()
                    .flex_1()
                    .when(stream_empty, |el: gpui::Div| {
                        el.flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_2xl()
                                            .text_color(rgb(0x404040))
                                            .child("💬"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(text_secondary)
                                            .child("No messages yet"),
                                    ),
                            )
                    })
                    .when(!stream_empty, |el: gpui::Div| {
                        // Render stream items
                        el.flex()
                            .flex_col()
                            .py_2()
                            .children(
                                stream_items
                                    .iter()
                                    .map(|item| {
                                        let is_selected = selected_id.as_ref() == Some(&item.id);
                                        render_stream_item_inline(item, is_selected, cx)
                                    })
                                    .collect::<Vec<_>>(),
                            )
                    }),
            )
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, _cx| {
                this.layout.focus_pane(Pane::Stream);
            }));

        // Stage (Zone C) - Right panel with artifact workspace
        let stage_focused = self.layout.is_focused(Pane::Stage);
        let stage = if self.layout.stage_collapsed {
            // Collapsed state - show a thin clickable strip
            div()
                .flex()
                .flex_col()
                .w(px(24.0))
                .h_full()
                .bg(bg_panel)
                .border_l_1()
                .border_color(border_color)
                .items_center()
                .pt_3()
                .cursor_pointer()
                .child(
                    div()
                        .text_color(text_secondary)
                        .text_xs()
                        .child("◀"),
                )
                .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, _cx| {
                    this.layout.toggle_stage();
                }))
        } else {
            // Render the full stage with artifact workspace
            div()
                .flex()
                .flex_col()
                .w(self.layout.stage_pixels())
                .h_full()
                .bg(bg_panel)
                .border_l_1()
                .border_color(if stage_focused {
                    border_focused
                } else {
                    border_color
                })
                .when(stage_focused, |el| el.border_l_2())
                // Stage header
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .px_3()
                        .py_2()
                        .border_b_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_color(if stage_focused {
                                            text_focused
                                        } else {
                                            text_secondary
                                        })
                                        .text_sm()
                                        .font_weight(if stage_focused {
                                            gpui::FontWeight::BOLD
                                        } else {
                                            gpui::FontWeight::NORMAL
                                        })
                                        .child("Stage"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(text_secondary)
                                        .child(format!("{} artifacts", self.stage_state.artifact_count())),
                                ),
                        )
                        .child(
                            div()
                                .text_color(text_secondary)
                                .text_xs()
                                .child("Ctrl+3"),
                        ),
                )
                // Stage content - artifact workspace
                .child(render_stage_pane(&mut self.stage_state))
                .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, _cx| {
                    this.layout.focus_pane(Pane::Stage);
                }))
        };

        // Main content area (3-column layout)
        let main_content = div()
            .flex()
            .flex_row()
            .flex_1()
            .w_full()
            .child(context_rail)
            .child(stream)
            .child(stage);

        // Footer / Status bar
        let footer = div()
            .flex()
            .h(px(24.0))
            .w_full()
            .bg(bg_panel)
            .border_t_1()
            .border_color(border_color)
            .px_4()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_color(text_secondary)
                    .text_xs()
                    .child(self.layout.status_string()),
            )
            .child(
                div()
                    .text_color(text_secondary)
                    .text_xs()
                    .child(format!("Frames: {}", self.app.frame_count)),
            );

        // Root layout with keyboard action handlers
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg_dark)
            .text_color(text_primary)
            .track_focus(&cx.focus_handle())
            .on_action(cx.listener(|this, _: &FocusContextRail, _window, _cx| {
                this.layout.focus_pane(Pane::ContextRail);
            }))
            .on_action(cx.listener(|this, _: &FocusStream, _window, _cx| {
                this.layout.focus_pane(Pane::Stream);
            }))
            .on_action(cx.listener(|this, _: &FocusStage, _window, _cx| {
                this.layout.focus_pane(Pane::Stage);
            }))
            .on_action(cx.listener(|this, _: &FocusNextPane, _window, _cx| {
                this.layout.focus_next();
            }))
            .on_action(cx.listener(|this, _: &FocusPreviousPane, _window, _cx| {
                this.layout.focus_previous();
            }))
            .on_action(cx.listener(|this, _: &ToggleContextRail, _window, _cx| {
                this.layout.toggle_context_rail();
            }))
            .on_action(cx.listener(|this, _: &ToggleStage, _window, _cx| {
                this.layout.toggle_stage();
            }))
            .child(status_bar)
            .child(main_content)
            .child(footer)
    }
}

/// Render a stream item inline (simplified version for main.rs)
fn render_stream_item_inline<V: 'static + Render>(
    item: &StreamItem,
    selected: bool,
    cx: &mut Context<V>,
) -> gpui::Div {
    use agui_desktop::stream::components::colors;

    let item_id = item.id.clone();

    div()
        .w_full()
        .px_2()
        .when(selected, |el| el.bg(rgba(0x264f784d))) // 0x4d = ~30% opacity
        .child(match &item.content {
            StreamContent::UserMessage(msg) => {
                // User bubble (right-aligned)
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .py_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .max_w(px(500.0))
                            .gap_1()
                            .when_some(msg.sender_name.clone(), |el, name| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x808080))
                                        .text_right()
                                        .child(name),
                                )
                            })
                            .child(
                                div()
                                    .bg(rgb(0x0e639c))
                                    .rounded_lg()
                                    .rounded_br_none()
                                    .px_4()
                                    .py_2()
                                    .text_color(rgb(0xffffff))
                                    .text_sm()
                                    .child(msg.content.clone()),
                            ),
                    )
            }
            StreamContent::AgentMessage(msg) => {
                // Agent bubble (left-aligned)
                let agent_name = msg.agent_name.clone().unwrap_or_else(|| msg.agent_id.clone());
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .justify_start()
                    .py_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .max_w(px(600.0))
                            .gap_1()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x808080))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child(agent_name),
                                    )
                                    .when(msg.streaming, |el| {
                                        el.child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x007acc))
                                                .child("typing..."),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .bg(rgb(0x2d2d30))
                                    .rounded_lg()
                                    .rounded_bl_none()
                                    .px_4()
                                    .py_3()
                                    .text_color(rgb(0xcccccc))
                                    .text_sm()
                                    .child(msg.content.clone()),
                            ),
                    )
            }
            StreamContent::Reasoning(reasoning) => {
                // Reasoning accordion
                let summary = reasoning
                    .summary
                    .clone()
                    .unwrap_or_else(|| {
                        let first_line = reasoning.content.lines().next().unwrap_or(&reasoning.content);
                        if first_line.len() > 50 {
                            format!("{}...", &first_line[..50])
                        } else {
                            first_line.to_string()
                        }
                    });
                div()
                    .w_full()
                    .py_1()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .bg(rgb(0x1e1e1e))
                            .border_1()
                            .border_color(rgb(0x404040))
                            .rounded_md()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .px_3()
                                    .py_2()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x606060))
                                                    .child(if reasoning.expanded { "▼" } else { "▶" }),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(rgb(0x606060))
                                                    .child("Thinking"),
                                            )
                                            .when(!reasoning.expanded, |el| {
                                                el.child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x808080))
                                                        .child(format!("— {}", summary)),
                                                )
                                            }),
                                    ),
                            ),
                    )
            }
            StreamContent::ToolCall(tc) => {
                // Tool call card
                let status_color = match tc.status {
                    ToolCallStatus::Pending => rgb(0x808080),
                    ToolCallStatus::Running => rgb(0x007acc),
                    ToolCallStatus::Completed => rgb(0x4ec9b0),
                    ToolCallStatus::Failed => rgb(0xf14c4c),
                    ToolCallStatus::Cancelled => rgb(0x808080),
                };
                div()
                    .w_full()
                    .py_1()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .bg(rgb(0x252526))
                            .rounded_md()
                            .border_1()
                            .border_color(status_color)
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .px_3()
                                    .py_2()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().child("🔧"))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(rgb(0xdcdcaa))
                                                    .child(tc.tool_name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .px_2()
                                                    .py_px()
                                                    .rounded_sm()
                                                    .bg(rgb(0x404040)) // Solid dark background
                                                    .text_xs()
                                                    .text_color(status_color)
                                                    .child(tc.status.label()),
                                            ),
                                    )
                                    .when_some(tc.duration_ms, |el, ms| {
                                        el.child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x606060))
                                                .child(format!("{}ms", ms)),
                                        )
                                    }),
                            ),
                    )
            }
            StreamContent::Plan(plan) => {
                // Plan checklist
                let completion = plan.completion_percentage();
                div()
                    .w_full()
                    .py_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .bg(rgb(0x252526))
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(0x3c3c3c))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .px_3()
                                    .py_2()
                                    .border_b_1()
                                    .border_color(rgb(0x3c3c3c))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(div().text_sm().child("📋"))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(rgb(0xcccccc))
                                                    .child(plan.title.clone()),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x808080))
                                            .child(format!("{}%", completion)),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .py_1()
                                    .children(
                                        plan.items.iter().map(|item| {
                                            let (icon, color) = match item.status {
                                                PlanItemStatus::Pending => ("○", rgb(0x606060)),
                                                PlanItemStatus::InProgress => ("◐", rgb(0x007acc)),
                                                PlanItemStatus::Completed => ("✓", rgb(0x4ec9b0)),
                                                PlanItemStatus::Skipped => ("⊘", rgb(0x606060)),
                                                PlanItemStatus::Failed => ("✗", rgb(0xf14c4c)),
                                            };
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .px_3()
                                                .py_1()
                                                .child(
                                                    div()
                                                        .w(px(16.0))
                                                        .text_xs()
                                                        .text_color(color)
                                                        .child(icon),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(if item.status == PlanItemStatus::Completed {
                                                            rgb(0x808080)
                                                        } else {
                                                            rgb(0xcccccc)
                                                        })
                                                        .when(item.status == PlanItemStatus::Completed, |el| {
                                                            el.line_through()
                                                        })
                                                        .child(item.description.clone()),
                                                )
                                        }).collect::<Vec<_>>(),
                                    ),
                            ),
                    )
            }
            StreamContent::Approval(approval) => {
                // Approval gate
                let is_resolved = approval.resolution.is_some();
                div()
                    .w_full()
                    .py_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .bg(rgb(0x2d2d30))
                            .rounded_md()
                            .border_2()
                            .border_color(if is_resolved { rgb(0x3c3c3c) } else { rgb(0x007acc) })
                            .when(!is_resolved && approval.blocking, |el| el.shadow_lg())
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .px_4()
                                    .py_3()
                                    .border_b_1()
                                    .border_color(rgb(0x3c3c3c))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .child(if is_resolved { "✓" } else { "⚠" }),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(rgb(0xcccccc))
                                                    .child(approval.title.clone()),
                                            ),
                                    )
                                    .when(approval.blocking && !is_resolved, |el| {
                                        el.child(
                                            div()
                                                .px_2()
                                                .py_px()
                                                .bg(rgb(0x5a2727))
                                                .rounded_sm()
                                                .text_xs()
                                                .text_color(rgb(0xf14c4c))
                                                .child("Blocking"),
                                        )
                                    }),
                            )
                            .when_some(approval.description.clone(), |el, desc| {
                                el.child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .text_sm()
                                        .text_color(rgb(0x808080))
                                        .child(desc),
                                )
                            })
                            .when(!is_resolved, |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_end()
                                        .gap_2()
                                        .px_4()
                                        .py_3()
                                        .border_t_1()
                                        .border_color(rgb(0x3c3c3c))
                                        .children(
                                            approval.actions.iter().map(|action| {
                                                let (bg, text) = match action.variant {
                                                    ApprovalActionVariant::Primary => (rgb(0x4ec9b0), rgb(0x1e1e1e)),
                                                    ApprovalActionVariant::Secondary => (rgb(0x3c3c3c), rgb(0xcccccc)),
                                                    ApprovalActionVariant::Destructive => (rgb(0xf14c4c), rgb(0xffffff)),
                                                };
                                                div()
                                                    .px_4()
                                                    .py_2()
                                                    .rounded_md()
                                                    .bg(bg)
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(text)
                                                    .cursor_pointer()
                                                    .child(action.label.clone())
                                            }).collect::<Vec<_>>(),
                                        ),
                                )
                            }),
                    )
            }
            StreamContent::StatusUpdate(status) => {
                // Status update
                let (icon, color) = match status.status_type {
                    agui_desktop::stream::types::StatusType::Info => ("ℹ", rgb(0x808080)),
                    agui_desktop::stream::types::StatusType::Success => ("✓", rgb(0x4ec9b0)),
                    agui_desktop::stream::types::StatusType::Warning => ("⚠", rgb(0xdcdcaa)),
                    agui_desktop::stream::types::StatusType::Error => ("✗", rgb(0xf14c4c)),
                    agui_desktop::stream::types::StatusType::Progress => ("◌", rgb(0x007acc)),
                };
                div()
                    .w_full()
                    .py_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .px_4()
                            .py_1()
                            .when(status.ephemeral, |el| el.opacity(0.8))
                            .child(div().text_xs().text_color(color).child(icon))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(color)
                                    .child(status.message.clone()),
                            )
                            .when_some(status.progress, |el, p| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            div()
                                                .w(px(40.0))
                                                .h(px(3.0))
                                                .bg(rgb(0x3c3c3c))
                                                .rounded_sm()
                                                .child(
                                                    div()
                                                        .h_full()
                                                        .w(px((p as f32 / 100.0) * 40.0))
                                                        .bg(rgb(0x0e639c))
                                                        .rounded_sm(),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x606060))
                                                .child(format!("{}%", p)),
                                        ),
                                )
                            }),
                    )
            }
            StreamContent::Divider => {
                div()
                    .w_full()
                    .py_2()
                    .child(div().w_full().h(px(1.0)).bg(rgb(0x3c3c3c)))
            }
        })
}

fn main() {
    // Load configuration from environment
    let config = AppConfig::from_env();

    // Initialize logging
    if let Err(e) = logging::init(&config.log_level) {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }
    logging::log_startup(&VERSION.to_string(), &format!("{:#?}", config));

    // Initialize metrics if enabled (need async runtime for this)
    if config.enable_metrics {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = metrics::init(config.metrics_port).await {
                tracing::error!("Failed to initialize metrics: {}", e);
            }
        });
    }

    // Start hot reload watcher if enabled
    let _hot_reload = if config.enable_hot_reload {
        match HotReloadWatcher::new(config.project_root.clone()) {
            Ok(watcher) => {
                tracing::info!("Hot reload enabled");
                Some(watcher)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize hot reload: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Run the gpui application
    Application::new().run(|cx: &mut App| {
        // Set up window bounds (centered, reasonable default size)
        let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);

        // Open the main window
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|_cx| AguiWindow::new()),
        )
        .expect("Failed to open window");

        // Activate the application
        cx.activate(true);

        // Set up global actions and key bindings
        cx.on_action(|_: &Quit, cx| cx.quit());

        // Key bindings
        cx.bind_keys([
            // Quit
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("ctrl-q", Quit, None),
            // Focus panes (Ctrl+1/2/3)
            KeyBinding::new("ctrl-1", FocusContextRail, None),
            KeyBinding::new("ctrl-2", FocusStream, None),
            KeyBinding::new("ctrl-3", FocusStage, None),
            // Navigate panes (Tab / Shift+Tab)
            KeyBinding::new("ctrl-tab", FocusNextPane, None),
            KeyBinding::new("ctrl-shift-tab", FocusPreviousPane, None),
            // Toggle panels (Ctrl+[ and Ctrl+])
            KeyBinding::new("ctrl-[", ToggleContextRail, None),
            KeyBinding::new("ctrl-]", ToggleStage, None),
        ]);
    });
}
