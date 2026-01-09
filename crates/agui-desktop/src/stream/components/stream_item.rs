//! Stream Item Component
//!
//! Renders a single stream item by dispatching to the appropriate
//! component based on content type.

use gpui::{div, prelude::*, px, AnyElement, Context, Window};

use super::{
    colors,
    agent_bubble::render_agent_bubble,
    approval_gate::render_approval_gate,
    plan_checklist::render_plan_checklist,
    reasoning_accordion::render_reasoning_accordion,
    status_update::render_status_update,
    tool_call_card::render_tool_call_card,
    user_bubble::render_user_bubble,
};
use crate::stream::types::{StreamContent, StreamItem};

/// Callback type for item interactions
pub type OnItemInteraction<V> = Box<dyn Fn(&mut V, &mut Window, &mut Context<V>, StreamItemInteraction) + Send + Sync>;

/// Types of item interactions
#[derive(Debug, Clone)]
pub enum StreamItemInteraction {
    /// Item was clicked/selected
    Select(String),
    /// Toggle expansion (for accordions, tool calls)
    ToggleExpand(String),
    /// Approval action taken
    ApprovalAction { item_id: String, action_id: String },
    /// Plan item clicked
    PlanItemClick { item_id: String, plan_item_id: String },
}

/// Render a stream item
pub fn render_stream_item<V: 'static>(
    item: &StreamItem,
    selected: bool,
    on_interaction: Option<OnItemInteraction<V>>,
    cx: &mut Context<V>,
) -> AnyElement {
    let item_id = item.id.clone();

    // Wrap in container with selection highlight
    div()
        .w_full()
        .px_2()
        .when(selected, |el| {
            el.bg(gpui::rgba(0x264f784d)) // 0x4d = ~30% opacity
        })
        .child(match &item.content {
            StreamContent::UserMessage(msg) => {
                render_user_bubble::<V>(msg, selected, None, cx)
            }
            StreamContent::AgentMessage(msg) => {
                render_agent_bubble::<V>(msg, selected, None, cx)
            }
            StreamContent::Reasoning(reasoning) => {
                render_reasoning_accordion::<V>(reasoning, None, cx)
            }
            StreamContent::ToolCall(tool_call) => {
                render_tool_call_card::<V>(tool_call, None, cx)
            }
            StreamContent::Plan(plan) => {
                render_plan_checklist::<V>(plan, None, cx)
            }
            StreamContent::Approval(approval) => {
                render_approval_gate::<V>(approval, None, cx)
            }
            StreamContent::StatusUpdate(status) => {
                render_status_update::<V>(status, cx)
            }
            StreamContent::Divider => {
                render_divider(cx)
            }
        })
        .into_any_element()
}

/// Render a visual divider
fn render_divider<V: 'static>(cx: &mut Context<V>) -> AnyElement {
    div()
        .w_full()
        .py_2()
        .child(
            div()
                .w_full()
                .h(px(1.0))
                .bg(colors::divider()),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::types::UserMessage;

    #[test]
    fn test_stream_item_interaction_types() {
        let select = StreamItemInteraction::Select("item-1".to_string());
        match select {
            StreamItemInteraction::Select(id) => assert_eq!(id, "item-1"),
            _ => panic!("Expected Select"),
        }

        let toggle = StreamItemInteraction::ToggleExpand("item-2".to_string());
        match toggle {
            StreamItemInteraction::ToggleExpand(id) => assert_eq!(id, "item-2"),
            _ => panic!("Expected ToggleExpand"),
        }
    }
}
