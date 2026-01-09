//! Form State Management
//!
//! Tracks the state of dynamically rendered forms and handles form submissions.
//! Form state is keyed by component ID and can hold various value types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Value types that can be stored in form state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FormValue {
    /// String value (text inputs, selects, etc.)
    String(String),
    /// Boolean value (toggles, checkboxes)
    Bool(bool),
    /// Numeric value (sliders, number inputs)
    Number(f64),
    /// Array of strings (multi-select)
    StringArray(Vec<String>),
    /// Null/empty value
    Null,
}

impl Default for FormValue {
    fn default() -> Self {
        FormValue::Null
    }
}

impl FormValue {
    /// Get as string, if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            FormValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as bool, if possible
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FormValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as number, if possible
    pub fn as_number(&self) -> Option<f64> {
        match self {
            FormValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as string array, if possible
    pub fn as_string_array(&self) -> Option<&[String]> {
        match self {
            FormValue::StringArray(arr) => Some(arr),
            _ => None,
        }
    }

    /// Check if value is null/empty
    pub fn is_null(&self) -> bool {
        matches!(self, FormValue::Null)
    }

    /// Convert to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            FormValue::String(s) => serde_json::Value::String(s.clone()),
            FormValue::Bool(b) => serde_json::Value::Bool(*b),
            FormValue::Number(n) => serde_json::json!(n),
            FormValue::StringArray(arr) => serde_json::json!(arr),
            FormValue::Null => serde_json::Value::Null,
        }
    }
}

impl From<String> for FormValue {
    fn from(s: String) -> Self {
        FormValue::String(s)
    }
}

impl From<&str> for FormValue {
    fn from(s: &str) -> Self {
        FormValue::String(s.to_string())
    }
}

impl From<bool> for FormValue {
    fn from(b: bool) -> Self {
        FormValue::Bool(b)
    }
}

impl From<f64> for FormValue {
    fn from(n: f64) -> Self {
        FormValue::Number(n)
    }
}

impl From<i64> for FormValue {
    fn from(n: i64) -> Self {
        FormValue::Number(n as f64)
    }
}

impl From<Vec<String>> for FormValue {
    fn from(arr: Vec<String>) -> Self {
        FormValue::StringArray(arr)
    }
}

/// Actions that can be triggered by form components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormAction {
    /// Action type/name
    pub action_type: String,
    /// Component ID that triggered the action
    pub component_id: String,
    /// Action payload (form data for submissions, custom data for buttons)
    pub payload: serde_json::Value,
}

impl FormAction {
    /// Create a new form action
    pub fn new(action_type: impl Into<String>, component_id: impl Into<String>) -> Self {
        Self {
            action_type: action_type.into(),
            component_id: component_id.into(),
            payload: serde_json::Value::Null,
        }
    }

    /// Create with payload
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Create a button click action
    pub fn button_click(component_id: impl Into<String>, action: impl Into<String>) -> Self {
        Self::new(action, component_id)
    }

    /// Create a form submit action
    pub fn form_submit(
        form_id: impl Into<String>,
        action: impl Into<String>,
        values: HashMap<String, FormValue>,
    ) -> Self {
        let payload = values
            .into_iter()
            .map(|(k, v)| (k, v.to_json()))
            .collect::<serde_json::Map<_, _>>();

        Self::new(action, form_id).with_payload(serde_json::Value::Object(payload))
    }

    /// Create a value change action
    pub fn value_change(component_id: impl Into<String>, value: FormValue) -> Self {
        Self::new("value_change", component_id).with_payload(value.to_json())
    }
}

/// Form state container
#[derive(Debug, Clone, Default)]
pub struct FormState {
    /// Form values keyed by component ID
    values: HashMap<String, FormValue>,
    /// Validation errors keyed by component ID
    errors: HashMap<String, String>,
    /// Components that have been touched (interacted with)
    touched: HashMap<String, bool>,
    /// Component-specific metadata
    metadata: HashMap<String, serde_json::Value>,
}

impl FormState {
    /// Create a new empty form state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value for a component
    pub fn set_value(&mut self, id: impl Into<String>, value: impl Into<FormValue>) {
        self.values.insert(id.into(), value.into());
    }

    /// Get a value by component ID
    pub fn get_value(&self, id: &str) -> Option<&FormValue> {
        self.values.get(id)
    }

    /// Get a value as string
    pub fn get_string(&self, id: &str) -> Option<&str> {
        self.values.get(id).and_then(|v| v.as_string())
    }

    /// Get a value as bool
    pub fn get_bool(&self, id: &str) -> Option<bool> {
        self.values.get(id).and_then(|v| v.as_bool())
    }

    /// Get a value as number
    pub fn get_number(&self, id: &str) -> Option<f64> {
        self.values.get(id).and_then(|v| v.as_number())
    }

    /// Remove a value
    pub fn remove_value(&mut self, id: &str) -> Option<FormValue> {
        self.values.remove(id)
    }

    /// Set an error for a component
    pub fn set_error(&mut self, id: impl Into<String>, error: impl Into<String>) {
        self.errors.insert(id.into(), error.into());
    }

    /// Get error for a component
    pub fn get_error(&self, id: &str) -> Option<&str> {
        self.errors.get(id).map(|s| s.as_str())
    }

    /// Clear error for a component
    pub fn clear_error(&mut self, id: &str) {
        self.errors.remove(id);
    }

    /// Clear all errors
    pub fn clear_all_errors(&mut self) {
        self.errors.clear();
    }

    /// Check if form has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Mark a component as touched
    pub fn set_touched(&mut self, id: impl Into<String>, touched: bool) {
        self.touched.insert(id.into(), touched);
    }

    /// Check if component has been touched
    pub fn is_touched(&self, id: &str) -> bool {
        self.touched.get(id).copied().unwrap_or(false)
    }

    /// Set metadata for a component
    pub fn set_metadata(&mut self, id: impl Into<String>, metadata: serde_json::Value) {
        self.metadata.insert(id.into(), metadata);
    }

    /// Get metadata for a component
    pub fn get_metadata(&self, id: &str) -> Option<&serde_json::Value> {
        self.metadata.get(id)
    }

    /// Get all values as a HashMap
    pub fn values(&self) -> &HashMap<String, FormValue> {
        &self.values
    }

    /// Get all errors as a HashMap
    pub fn errors(&self) -> &HashMap<String, String> {
        &self.errors
    }

    /// Convert all values to JSON
    pub fn to_json(&self) -> serde_json::Value {
        let map: serde_json::Map<_, _> = self
            .values
            .iter()
            .map(|(k, v)| (k.clone(), v.to_json()))
            .collect();
        serde_json::Value::Object(map)
    }

    /// Create form state from initial values
    pub fn from_values(values: HashMap<String, FormValue>) -> Self {
        Self {
            values,
            ..Default::default()
        }
    }

    /// Reset the form state
    pub fn reset(&mut self) {
        self.values.clear();
        self.errors.clear();
        self.touched.clear();
        self.metadata.clear();
    }

    /// Get values for a specific form (by ID prefix)
    pub fn get_form_values(&self, form_id: &str) -> HashMap<String, FormValue> {
        let prefix = format!("{}.", form_id);
        self.values
            .iter()
            .filter_map(|(k, v)| {
                if k.starts_with(&prefix) {
                    Some((k.strip_prefix(&prefix).unwrap().to_string(), v.clone()))
                } else if k == form_id {
                    None // Skip the form ID itself
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Builder for form state with initial values
pub struct FormStateBuilder {
    state: FormState,
}

impl FormStateBuilder {
    pub fn new() -> Self {
        Self {
            state: FormState::new(),
        }
    }

    pub fn with_value(mut self, id: impl Into<String>, value: impl Into<FormValue>) -> Self {
        self.state.set_value(id, value);
        self
    }

    pub fn with_string(self, id: impl Into<String>, value: impl Into<String>) -> Self {
        self.with_value(id, FormValue::String(value.into()))
    }

    pub fn with_bool(self, id: impl Into<String>, value: bool) -> Self {
        self.with_value(id, FormValue::Bool(value))
    }

    pub fn with_number(self, id: impl Into<String>, value: f64) -> Self {
        self.with_value(id, FormValue::Number(value))
    }

    pub fn build(self) -> FormState {
        self.state
    }
}

impl Default for FormStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_value_types() {
        assert_eq!(FormValue::from("hello").as_string(), Some("hello"));
        assert_eq!(FormValue::from(true).as_bool(), Some(true));
        assert_eq!(FormValue::from(42.5).as_number(), Some(42.5));
        assert!(FormValue::Null.is_null());
    }

    #[test]
    fn test_form_state_basic() {
        let mut state = FormState::new();

        state.set_value("name", "John");
        state.set_value("active", true);
        state.set_value("count", 42.0);

        assert_eq!(state.get_string("name"), Some("John"));
        assert_eq!(state.get_bool("active"), Some(true));
        assert_eq!(state.get_number("count"), Some(42.0));
    }

    #[test]
    fn test_form_state_errors() {
        let mut state = FormState::new();

        state.set_error("email", "Invalid email format");
        assert_eq!(state.get_error("email"), Some("Invalid email format"));
        assert!(state.has_errors());

        state.clear_error("email");
        assert!(!state.has_errors());
    }

    #[test]
    fn test_form_state_touched() {
        let mut state = FormState::new();

        assert!(!state.is_touched("name"));
        state.set_touched("name", true);
        assert!(state.is_touched("name"));
    }

    #[test]
    fn test_form_state_to_json() {
        let mut state = FormState::new();
        state.set_value("name", "John");
        state.set_value("age", 30.0);

        let json = state.to_json();
        assert_eq!(json["name"], "John");
        assert_eq!(json["age"], 30.0);
    }

    #[test]
    fn test_form_action() {
        let action = FormAction::button_click("btn-1", "submit_form");
        assert_eq!(action.action_type, "submit_form");
        assert_eq!(action.component_id, "btn-1");
    }

    #[test]
    fn test_form_submit_action() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), FormValue::String("John".to_string()));
        values.insert("active".to_string(), FormValue::Bool(true));

        let action = FormAction::form_submit("user-form", "create_user", values);
        assert_eq!(action.action_type, "create_user");
        assert_eq!(action.payload["name"], "John");
        assert_eq!(action.payload["active"], true);
    }

    #[test]
    fn test_form_state_builder() {
        let state = FormStateBuilder::new()
            .with_string("name", "John")
            .with_bool("active", true)
            .with_number("score", 95.5)
            .build();

        assert_eq!(state.get_string("name"), Some("John"));
        assert_eq!(state.get_bool("active"), Some(true));
        assert_eq!(state.get_number("score"), Some(95.5));
    }

    #[test]
    fn test_get_form_values() {
        let mut state = FormState::new();
        state.set_value("myform.name", "John");
        state.set_value("myform.email", "john@example.com");
        state.set_value("other.field", "ignored");

        let form_values = state.get_form_values("myform");
        assert_eq!(form_values.len(), 2);
        assert_eq!(
            form_values.get("name").and_then(|v| v.as_string()),
            Some("John")
        );
        assert_eq!(
            form_values.get("email").and_then(|v| v.as_string()),
            Some("john@example.com")
        );
    }
}
