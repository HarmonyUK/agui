//! Tool definitions and execution framework
//!
//! Provides a trait-based system for defining and executing tools that
//! can be invoked by agents. Tools include file operations and shell commands.

pub mod schema;
pub mod file_ops;
pub mod shell;

use schema::{ParameterSchema, ValidationError};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Result type for tool execution
pub type ToolResult = Result<Value, ToolError>;

/// Tool trait that all executable tools must implement
pub trait Tool: Send + Sync {
    /// Unique name/identifier for this tool
    fn name(&self) -> &str;

    /// Human-readable description of what this tool does
    fn description(&self) -> &str;

    /// JSON Schema defining the parameters this tool accepts
    fn parameters_schema(&self) -> &ParameterSchema;

    /// Execute the tool with the given parameters
    fn execute(&self, params: &Value) -> ToolResult;
}

/// Errors that can occur during tool execution
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Parameter validation failed: {0}")]
    ValidationError(#[from] ValidationError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("Tool not found: {0}")]
    NotFound(String),
}

/// Registry for looking up tools by name
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Execute a tool by name with the given parameters
    pub fn execute(&self, name: &str, params: &Value) -> ToolResult {
        let tool = self.get(name).ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        // Validate parameters against schema
        tool.parameters_schema().validate(params)?;

        // Execute the tool
        tool.execute(params)
    }

    /// List all registered tool names
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get all tool metadata
    pub fn tool_metadata(&self) -> Vec<ToolMetadata> {
        self.tools
            .values()
            .map(|t| ToolMetadata {
                name: t.name().to_string(),
                description: t.description().to_string(),
                parameters_schema: t.parameters_schema().clone(),
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about a tool
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub parameters_schema: ParameterSchema,
}

/// Create a new registry with all standard tools registered
pub fn create_standard_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // Register file operation tools
    registry.register(Box::new(file_ops::ReadFile));
    registry.register(Box::new(file_ops::WriteFile));
    registry.register(Box::new(file_ops::ListDirectory));
    registry.register(Box::new(file_ops::FileExists));

    // Register shell tool
    registry.register(Box::new(shell::RunCommand));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_basic_operations() {
        let mut registry = ToolRegistry::new();

        // Register a simple test tool
        registry.register(Box::new(MockTestTool));

        // List tools
        let tools = registry.list_tools();
        assert_eq!(tools, vec!["mock_test"]);

        // Get tool
        let tool = registry.get("mock_test");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "mock_test");

        // Execute tool
        let result = registry.execute(
            "mock_test",
            &serde_json::json!({"value": "test"}),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!("processed: test".to_string()));
    }

    #[test]
    fn test_registry_not_found() {
        let registry = ToolRegistry::new();
        let result = registry.execute("nonexistent", &serde_json::json!({}));
        assert!(matches!(result, Err(ToolError::NotFound(_))));
    }

    #[test]
    fn test_registry_parameter_validation() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(MockTestTool));

        // Missing required parameter
        let result = registry.execute("mock_test", &serde_json::json!({}));
        assert!(matches!(result, Err(ToolError::ValidationError(_))));

        // Wrong parameter type
        let result = registry.execute(
            "mock_test",
            &serde_json::json!({"value": 123}),
        );
        assert!(matches!(result, Err(ToolError::ValidationError(_))));
    }

    #[test]
    fn test_standard_registry() {
        let registry = create_standard_registry();
        let tools = registry.list_tools();

        assert!(tools.contains(&"read_file".to_string()));
        assert!(tools.contains(&"write_file".to_string()));
        assert!(tools.contains(&"list_directory".to_string()));
        assert!(tools.contains(&"file_exists".to_string()));
        assert!(tools.contains(&"run_command".to_string()));
    }

    /// Mock tool for testing
    struct MockTestTool;

    impl Tool for MockTestTool {
        fn name(&self) -> &str {
            "mock_test"
        }

        fn description(&self) -> &str {
            "A mock test tool"
        }

        fn parameters_schema(&self) -> &ParameterSchema {
            static SCHEMA: std::sync::OnceLock<ParameterSchema> = std::sync::OnceLock::new();
            SCHEMA.get_or_init(|| {
                let mut props = std::collections::HashMap::new();
                props.insert("value".to_string(), ParameterSchema::string());
                ParameterSchema::object(props)
                    .with_required(vec!["value".to_string()])
            })
        }

        fn execute(&self, params: &Value) -> ToolResult {
            let value = params
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::ExecutionFailed("Invalid value".to_string()))?;

            Ok(serde_json::json!(format!("processed: {}", value)))
        }
    }
}
