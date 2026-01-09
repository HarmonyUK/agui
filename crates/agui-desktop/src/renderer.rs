//! Declarative UI Renderer
//!
//! Renders UI components from JSON schema definitions with form state management.
//! This module implements the "Generative UI" pattern where the orchestrator
//! can request dynamic UI rendering via RENDER_REQUEST events.

pub mod components;
pub mod form_state;
pub mod schema;

use gpui::{prelude::*, AnyElement, Context, Window};
use std::sync::Arc;

pub use form_state::{FormAction, FormState, FormValue};
pub use schema::{Component, *};

/// Callback type for user actions (form submissions, button clicks, etc.)
pub type ActionCallback<V> = Arc<dyn Fn(&mut V, &mut Window, &mut Context<V>, FormAction) + Send + Sync>;

/// Render context passed to component renderers
pub struct RenderContext<'a, V: 'static> {
    /// Current form state
    pub form_state: &'a FormState,
    /// Callback for user actions
    pub on_action: ActionCallback<V>,
    /// Component ID prefix for nested components
    pub id_prefix: String,
}

impl<'a, V: 'static> RenderContext<'a, V> {
    /// Create a new render context
    pub fn new(form_state: &'a FormState, on_action: ActionCallback<V>) -> Self {
        Self {
            form_state,
            on_action,
            id_prefix: String::new(),
        }
    }

    /// Create a child context with a new ID prefix
    pub fn with_prefix(&self, prefix: &str) -> RenderContext<'_, V> {
        let new_prefix = if self.id_prefix.is_empty() {
            prefix.to_string()
        } else {
            format!("{}.{}", self.id_prefix, prefix)
        };
        RenderContext {
            form_state: self.form_state,
            on_action: Arc::clone(&self.on_action),
            id_prefix: new_prefix,
        }
    }

    /// Get the full ID for a component
    pub fn full_id(&self, id: &str) -> String {
        if self.id_prefix.is_empty() {
            id.to_string()
        } else {
            format!("{}.{}", self.id_prefix, id)
        }
    }
}

/// Render a component tree from a schema definition
pub fn render_component<V: 'static + Render>(
    schema: &Component,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement {
    match schema {
        // Text components
        Component::Text(props) => components::text::render_text(props, ctx, cx),
        Component::Markdown(props) => components::text::render_markdown(props, ctx, cx),
        Component::Header(props) => components::text::render_header(props, ctx, cx),
        Component::Code(props) => components::text::render_code(props, ctx, cx),

        // Button components
        Component::Button(props) => components::button::render_button(props, ctx, cx),
        Component::IconButton(props) => components::button::render_icon_button(props, ctx, cx),

        // Input components
        Component::TextInput(props) => components::input::render_text_input(props, ctx, cx),
        Component::TextArea(props) => components::input::render_text_area(props, ctx, cx),
        Component::Select(props) => components::input::render_select(props, ctx, cx),
        Component::Toggle(props) => components::input::render_toggle(props, ctx, cx),
        Component::Slider(props) => components::input::render_slider(props, ctx, cx),
        Component::Checkbox(props) => components::input::render_checkbox(props, ctx, cx),

        // Container components
        Component::Accordion(props) => components::container::render_accordion(props, ctx, cx),
        Component::Tabs(props) => components::container::render_tabs(props, ctx, cx),
        Component::Modal(props) => components::container::render_modal(props, ctx, cx),
        Component::Drawer(props) => components::container::render_drawer(props, ctx, cx),
        Component::Card(props) => components::container::render_card(props, ctx, cx),

        // Data display components
        Component::Table(props) => components::data::render_table(props, ctx, cx),
        Component::Tree(props) => components::data::render_tree(props, ctx, cx),
        Component::Badge(props) => components::data::render_badge(props, ctx, cx),
        Component::Progress(props) => components::data::render_progress(props, ctx, cx),
        Component::Chip(props) => components::data::render_chip(props, ctx, cx),
        Component::Toast(props) => components::data::render_toast(props, ctx, cx),
        Component::Diff(props) => components::data::render_diff(props, ctx, cx),
        Component::RosterItem(props) => components::data::render_roster_item(props, ctx, cx),

        // Layout components
        Component::Row(props) => components::layout::render_row(props, ctx, cx),
        Component::Column(props) => components::layout::render_column(props, ctx, cx),
        Component::Stack(props) => components::layout::render_stack(props, ctx, cx),
        Component::Spacer(props) => components::layout::render_spacer(props, ctx, cx),
        Component::Divider(props) => components::layout::render_divider(props, ctx, cx),

        // Form component
        Component::Form(props) => components::form::render_form(props, ctx, cx),
    }
}

/// Parse a component from JSON value
pub fn parse_component(value: &serde_json::Value) -> Result<Component, serde_json::Error> {
    serde_json::from_value(value.clone())
}

/// Render a component from a JSON value
pub fn render_from_json<V: 'static + Render>(
    value: &serde_json::Value,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> Result<AnyElement, serde_json::Error> {
    let component = parse_component(value)?;
    Ok(render_component(&component, ctx, cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_component() {
        let json = serde_json::json!({
            "type": "text",
            "id": "greeting",
            "content": "Hello, World!"
        });

        let component = parse_component(&json).unwrap();
        match component {
            Component::Text(props) => {
                assert_eq!(props.id, "greeting");
                assert_eq!(props.content, "Hello, World!");
            }
            _ => panic!("Expected Text component"),
        }
    }

    #[test]
    fn test_parse_button_component() {
        let json = serde_json::json!({
            "type": "button",
            "id": "submit-btn",
            "label": "Submit",
            "action": "submit_form",
            "variant": "primary"
        });

        let component = parse_component(&json).unwrap();
        match component {
            Component::Button(props) => {
                assert_eq!(props.id, "submit-btn");
                assert_eq!(props.label, "Submit");
                assert_eq!(props.action, "submit_form");
            }
            _ => panic!("Expected Button component"),
        }
    }

    #[test]
    fn test_parse_form_component() {
        let json = serde_json::json!({
            "type": "form",
            "id": "user-form",
            "submit_action": "create_user",
            "children": [
                {
                    "type": "text_input",
                    "id": "username",
                    "label": "Username",
                    "placeholder": "Enter username"
                },
                {
                    "type": "button",
                    "id": "submit",
                    "label": "Create User",
                    "action": "submit"
                }
            ]
        });

        let component = parse_component(&json).unwrap();
        match component {
            Component::Form(props) => {
                assert_eq!(props.id, "user-form");
                assert_eq!(props.submit_action, "create_user");
                assert_eq!(props.children.len(), 2);
            }
            _ => panic!("Expected Form component"),
        }
    }
}
