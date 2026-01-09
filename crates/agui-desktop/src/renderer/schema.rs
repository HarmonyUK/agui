//! UI Component Schema Definitions
//!
//! Defines the JSON schema types for all declarative UI components.
//! These types are deserialized from RENDER_REQUEST events.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root component enum that can represent any UI component
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Component {
    // Text components
    Text(TextProps),
    Markdown(MarkdownProps),
    Header(HeaderProps),
    Code(CodeProps),

    // Button components
    Button(ButtonProps),
    IconButton(IconButtonProps),

    // Input components
    TextInput(TextInputProps),
    TextArea(TextAreaProps),
    Select(SelectProps),
    Toggle(ToggleProps),
    Slider(SliderProps),
    Checkbox(CheckboxProps),

    // Container components
    Accordion(AccordionProps),
    Tabs(TabsProps),
    Modal(ModalProps),
    Drawer(DrawerProps),
    Card(CardProps),

    // Data display components
    Table(TableProps),
    Tree(TreeProps),
    Badge(BadgeProps),
    Progress(ProgressProps),
    Chip(ChipProps),
    Toast(ToastProps),
    Diff(DiffProps),
    RosterItem(RosterItemProps),

    // Layout components
    Row(RowProps),
    Column(ColumnProps),
    Stack(StackProps),
    Spacer(SpacerProps),
    Divider(DividerProps),

    // Form component
    Form(FormProps),
}

// ============================================================================
// Text Components
// ============================================================================

/// Plain text display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextProps {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub size: TextSize,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub weight: FontWeight,
    #[serde(default)]
    pub align: TextAlign,
}

/// Markdown-rendered text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarkdownProps {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub max_height: Option<f32>,
}

/// Header/title text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeaderProps {
    pub id: String,
    pub content: String,
    #[serde(default = "default_header_level")]
    pub level: u8,
}

fn default_header_level() -> u8 {
    1
}

/// Code block with syntax highlighting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeProps {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub line_numbers: bool,
    #[serde(default)]
    pub editable: bool,
}

// ============================================================================
// Button Components
// ============================================================================

/// Standard button
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ButtonProps {
    pub id: String,
    pub label: String,
    pub action: String,
    #[serde(default)]
    pub variant: ButtonVariant,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub loading: bool,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Icon-only button
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IconButtonProps {
    pub id: String,
    pub icon: String,
    pub action: String,
    #[serde(default)]
    pub tooltip: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub variant: ButtonVariant,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

// ============================================================================
// Input Components
// ============================================================================

/// Single-line text input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextInputProps {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub input_type: InputType,
    #[serde(default)]
    pub error: Option<String>,
}

/// Multi-line text area
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextAreaProps {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default = "default_rows")]
    pub rows: u32,
    #[serde(default)]
    pub error: Option<String>,
}

fn default_rows() -> u32 {
    4
}

/// Dropdown select
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectProps {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub multi: bool,
    #[serde(default)]
    pub error: Option<String>,
}

/// Select option
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub disabled: bool,
}

/// Toggle/switch input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToggleProps {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub checked: bool,
    #[serde(default)]
    pub disabled: bool,
}

/// Slider/range input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SliderProps {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub min: f64,
    #[serde(default = "default_max")]
    pub max: f64,
    #[serde(default)]
    pub value: f64,
    #[serde(default = "default_step")]
    pub step: f64,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub show_value: bool,
}

fn default_max() -> f64 {
    100.0
}

fn default_step() -> f64 {
    1.0
}

/// Checkbox input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckboxProps {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub checked: bool,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub indeterminate: bool,
}

// ============================================================================
// Container Components
// ============================================================================

/// Collapsible accordion sections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccordionProps {
    pub id: String,
    pub sections: Vec<AccordionSection>,
    #[serde(default)]
    pub allow_multiple: bool,
    #[serde(default)]
    pub expanded: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccordionSection {
    pub id: String,
    pub title: String,
    pub content: Box<Component>,
}

/// Tabbed interface
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TabsProps {
    pub id: String,
    pub tabs: Vec<Tab>,
    #[serde(default)]
    pub active_tab: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tab {
    pub id: String,
    pub label: String,
    pub content: Box<Component>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

/// Modal dialog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModalProps {
    pub id: String,
    pub title: String,
    pub content: Box<Component>,
    #[serde(default)]
    pub open: bool,
    #[serde(default)]
    pub close_action: Option<String>,
    #[serde(default)]
    pub footer: Option<Box<Component>>,
    #[serde(default)]
    pub width: Option<f32>,
}

/// Side drawer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrawerProps {
    pub id: String,
    pub title: Option<String>,
    pub content: Box<Component>,
    #[serde(default)]
    pub open: bool,
    #[serde(default)]
    pub position: DrawerPosition,
    #[serde(default)]
    pub close_action: Option<String>,
    #[serde(default)]
    pub width: Option<f32>,
}

/// Card container
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardProps {
    pub id: String,
    pub content: Box<Component>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub header: Option<Box<Component>>,
    #[serde(default)]
    pub footer: Option<Box<Component>>,
    #[serde(default)]
    pub padding: Option<f32>,
}

// ============================================================================
// Data Display Components
// ============================================================================

/// Data table/grid
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableProps {
    pub id: String,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    #[serde(default)]
    pub selectable: bool,
    #[serde(default)]
    pub selected: Vec<String>,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub sort_column: Option<String>,
    #[serde(default)]
    pub sort_direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableColumn {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub align: TextAlign,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableRow {
    pub id: String,
    pub cells: HashMap<String, serde_json::Value>,
}

/// Tree view for files/resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreeProps {
    pub id: String,
    pub nodes: Vec<TreeNode>,
    #[serde(default)]
    pub expanded: Vec<String>,
    #[serde(default)]
    pub selected: Option<String>,
    #[serde(default)]
    pub on_select: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub children: Vec<TreeNode>,
    #[serde(default)]
    pub node_type: Option<String>,
}

/// Status badge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BadgeProps {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub variant: BadgeVariant,
    #[serde(default)]
    pub dot: bool,
}

/// Progress indicator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressProps {
    pub id: String,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub max: f64,
    #[serde(default)]
    pub variant: ProgressVariant,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub show_percentage: bool,
}

/// Chip/tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChipProps {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub variant: ChipVariant,
    #[serde(default)]
    pub dismissible: bool,
    #[serde(default)]
    pub dismiss_action: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

/// Toast notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToastProps {
    pub id: String,
    pub message: String,
    #[serde(default)]
    pub variant: ToastVariant,
    #[serde(default)]
    pub duration: Option<u32>,
    #[serde(default)]
    pub action: Option<ToastAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToastAction {
    pub label: String,
    pub action: String,
}

/// Diff view
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffProps {
    pub id: String,
    pub old_content: String,
    pub new_content: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub unified: bool,
    #[serde(default)]
    pub context_lines: u32,
}

/// Agent roster item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RosterItemProps {
    pub id: String,
    pub name: String,
    pub status: AgentStatusType,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub on_click: Option<String>,
}

// ============================================================================
// Layout Components
// ============================================================================

/// Horizontal row layout
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RowProps {
    pub id: String,
    pub children: Vec<Component>,
    #[serde(default)]
    pub gap: Option<f32>,
    #[serde(default)]
    pub align: AlignItems,
    #[serde(default)]
    pub justify: JustifyContent,
    #[serde(default)]
    pub wrap: bool,
}

/// Vertical column layout
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnProps {
    pub id: String,
    pub children: Vec<Component>,
    #[serde(default)]
    pub gap: Option<f32>,
    #[serde(default)]
    pub align: AlignItems,
    #[serde(default)]
    pub justify: JustifyContent,
}

/// Stacked/layered layout
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StackProps {
    pub id: String,
    pub children: Vec<Component>,
    #[serde(default)]
    pub align: AlignItems,
}

/// Spacer element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpacerProps {
    pub id: String,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub flex: bool,
}

/// Divider line
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DividerProps {
    pub id: String,
    #[serde(default)]
    pub vertical: bool,
    #[serde(default)]
    pub margin: Option<f32>,
}

// ============================================================================
// Form Component
// ============================================================================

/// Form container with submit handling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormProps {
    pub id: String,
    pub children: Vec<Component>,
    pub submit_action: String,
    #[serde(default)]
    pub cancel_action: Option<String>,
    #[serde(default)]
    pub submit_label: Option<String>,
    #[serde(default)]
    pub cancel_label: Option<String>,
}

// ============================================================================
// Enums for variants and options
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
    Xxl,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FontWeight {
    Light,
    #[default]
    Normal,
    Medium,
    Semibold,
    Bold,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    #[default]
    Text,
    Password,
    Email,
    Number,
    Url,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DrawerPosition {
    #[default]
    Right,
    Left,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum BadgeVariant {
    #[default]
    Default,
    Primary,
    Secondary,
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProgressVariant {
    #[default]
    Linear,
    Circular,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChipVariant {
    #[default]
    Default,
    Primary,
    Secondary,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToastVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatusType {
    Online,
    Busy,
    #[default]
    Idle,
    Offline,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AlignItems {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum JustifyContent {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_props_defaults() {
        let json = serde_json::json!({
            "type": "text",
            "id": "test",
            "content": "Hello"
        });

        let component: Component = serde_json::from_value(json).unwrap();
        if let Component::Text(props) = component {
            assert_eq!(props.size, TextSize::Md);
            assert_eq!(props.weight, FontWeight::Normal);
            assert_eq!(props.align, TextAlign::Left);
        } else {
            panic!("Expected Text component");
        }
    }

    #[test]
    fn test_button_variants() {
        let variants = ["primary", "secondary", "outline", "ghost", "destructive"];
        for variant in variants {
            let json = serde_json::json!({
                "type": "button",
                "id": "btn",
                "label": "Test",
                "action": "test",
                "variant": variant
            });
            let _: Component = serde_json::from_value(json).unwrap();
        }
    }

    #[test]
    fn test_nested_components() {
        let json = serde_json::json!({
            "type": "card",
            "id": "card-1",
            "content": {
                "type": "column",
                "id": "col-1",
                "children": [
                    {"type": "text", "id": "t1", "content": "Title"},
                    {"type": "text", "id": "t2", "content": "Subtitle"}
                ]
            }
        });

        let component: Component = serde_json::from_value(json).unwrap();
        if let Component::Card(props) = component {
            if let Component::Column(col) = *props.content {
                assert_eq!(col.children.len(), 2);
            } else {
                panic!("Expected Column");
            }
        } else {
            panic!("Expected Card");
        }
    }
}
