---
id: doc-kj8
title: AGUI Declarative UI Renderer Implementation
type: reference
scope: internal
created_at: '2026-01-09T03:16:44.651767Z'
updated_at: '2026-01-09T03:16:44.651767Z'
---

# AGUI Declarative UI Renderer Implementation

## Overview

The AGUI declarative UI renderer enables dynamic UI generation from JSON schemas. This implements the "Generative UI" pattern where the orchestrator can request UI rendering via `RENDER_REQUEST` protocol events.

**Issue:** agui-2gh - AGUI: Declarative UI renderer & forms

**Location:** `crates/agui-desktop/src/renderer/`

## Architecture

```
renderer/
├── mod.rs              # Main module, render_component dispatcher
├── schema.rs           # Component type definitions (25+ types)
├── form_state.rs       # Form state management
└── components/
    ├── mod.rs          # Color palette
    ├── text.rs         # Text, Markdown, Header, Code
    ├── button.rs       # Button, IconButton
    ├── input.rs        # TextInput, TextArea, Select, Toggle, Slider, Checkbox
    ├── container.rs    # Accordion, Tabs, Modal, Drawer, Card
    ├── data.rs         # Table, Tree, Badge, Progress, Chip, Toast, Diff, RosterItem
    ├── layout.rs       # Row, Column, Stack, Spacer, Divider
    └── form.rs         # Form container
```

## API Reference

### Core Functions

```rust
/// Parse a component from JSON value
pub fn parse_component(value: &serde_json::Value) -> Result<Component, serde_json::Error>

/// Render a component tree from a schema definition
pub fn render_component<V: 'static + Render>(
    schema: &Component,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> AnyElement

/// Render directly from JSON
pub fn render_from_json<V: 'static + Render>(
    value: &serde_json::Value,
    ctx: &RenderContext<'_, V>,
    cx: &mut Context<V>,
) -> Result<AnyElement, serde_json::Error>
```

### RenderContext

```rust
pub struct RenderContext<'a, V: 'static> {
    pub form_state: &'a FormState,
    pub on_action: ActionCallback<V>,
    pub id_prefix: String,
}

pub type ActionCallback<V> = Arc<dyn Fn(&mut V, &mut Window, &mut Context<V>, FormAction) + Send + Sync>;
```

### FormState

```rust
pub struct FormState {
    values: HashMap<String, FormValue>,
    errors: HashMap<String, String>,
    touched: HashSet<String>,
}

pub enum FormValue {
    String(String),
    Bool(bool),
    Number(f64),
    StringArray(Vec<String>),
    Null,
}
```

### FormAction

```rust
pub struct FormAction {
    pub action_type: ActionType,
    pub component_id: String,
    pub action_name: String,
    pub form_data: Option<HashMap<String, FormValue>>,
    pub payload: serde_json::Value,
}

pub enum ActionType {
    ButtonClick,
    FormSubmit,
    InputChange,
    SelectionChange,
}
```

## Component Types

### Text Components
- **Text**: Basic text display with optional color/size
- **Markdown**: Markdown content (parsed to styled elements)
- **Header**: H1-H6 headings
- **Code**: Code blocks with optional line numbers and language

### Button Components
- **Button**: Standard button with label, icon, variants (Primary, Secondary, Outline, Ghost, Destructive)
- **IconButton**: Icon-only button with tooltip support

### Input Components
- **TextInput**: Single-line input with label, placeholder, validation
- **TextArea**: Multi-line text input
- **Select**: Dropdown selection with options
- **Toggle**: Boolean toggle switch
- **Slider**: Numeric slider with min/max/step
- **Checkbox**: Checkbox with label

### Container Components
- **Accordion**: Collapsible sections
- **Tabs**: Tabbed content panels
- **Modal**: Modal dialog overlay
- **Drawer**: Slide-in panel (left, right, top, bottom)
- **Card**: Card container with header/footer

### Data Display Components
- **Table**: Data table with sortable columns, row selection
- **Tree**: Hierarchical tree view with expand/collapse
- **Badge**: Status badges with variants
- **Progress**: Progress bars (determinate/indeterminate)
- **Chip**: Tag/chip with optional remove action
- **Toast**: Notification toasts with variants
- **Diff**: Side-by-side diff viewer
- **RosterItem**: Agent roster display item

### Layout Components
- **Row**: Horizontal flex container
- **Column**: Vertical flex container
- **Stack**: Overlapping layers
- **Spacer**: Fixed or flexible spacing
- **Divider**: Horizontal/vertical divider line

### Form Component
- **Form**: Form container with submit/cancel handling

## Configuration

### JSON Schema Format

Components use a discriminated union with `type` field:

```json
{
  "type": "button",
  "id": "submit-btn",
  "label": "Submit",
  "action": "submit_form",
  "variant": "primary"
}
```

### Nested Components

```json
{
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
}
```

## Examples

### Basic Usage

```rust
use agui_desktop::renderer::{
    parse_component, render_component, FormState, RenderContext
};
use std::sync::Arc;

// Parse component from JSON
let json = serde_json::json!({
    "type": "text",
    "id": "greeting",
    "content": "Hello, World!"
});
let component = parse_component(&json).unwrap();

// Create render context
let form_state = FormState::new();
let on_action = Arc::new(|_view, _window, _cx, action| {
    println!("Action: {:?}", action);
});
let ctx = RenderContext::new(&form_state, on_action);

// Render (inside gpui context)
let element = render_component(&component, &ctx, cx);
```

### Protocol Integration

```rust
impl AguiWindow {
    pub fn handle_render_request(
        &mut self,
        schema_json: &serde_json::Value
    ) -> Result<(), serde_json::Error> {
        let component = parse_component(schema_json)?;
        self.rendered_component = Some(component);
        Ok(())
    }

    fn handle_form_action(&mut self, action: FormAction) {
        // Convert to UserAction protocol event
        let user_action = UserAction {
            action_type: action.action_type.to_string(),
            component_id: action.component_id,
            payload: serde_json::to_value(&action.form_data).unwrap(),
        };
        // Send to orchestrator...
    }
}
```

## Implementation Notes

### GPUI Compatibility

- Element IDs require `SharedString` for 'static lifetime: `.id(gpui::SharedString::from(props.id.clone()))`
- `gpui::rgba()` takes single u32 hex, not 4 floats
- Some CSS methods unavailable (e.g., `overflow_y_scroll`), use alternatives like `overflow_hidden`
- `justify_evenly` not available, mapped to `justify_around`

### Color Palette

Defined in `components/mod.rs`:
- Background: `bg_dark`, `bg_panel`, `bg_elevated`, `bg_hover`, `bg_input`
- Text: `text_primary`, `text_secondary`, `text_muted`
- Borders: `border_default`, `border_focused`, `border_error`
- Semantic: `primary`, `success`, `warning`, `error`, `info`

## Testing

34 unit tests covering:
- Schema parsing for all component types
- Form state operations (values, errors, touched)
- Form action creation
- Nested component parsing

Run tests: `cargo test -p agui-desktop`

## See Also

- Protocol specification: `crates/agui-desktop/src/protocol.rs`
- Implementation plan: `docs/architecture/implementation_plan.md`
- GPUI documentation: https://gpui.rs/
