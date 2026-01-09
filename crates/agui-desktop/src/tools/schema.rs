//! Tool parameter schema definition and validation
//!
//! Provides JSON Schema support for tool parameter validation.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// JSON Schema for tool parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParameterSchema {
    /// Schema type (object, string, number, etc.)
    #[serde(rename = "type")]
    pub schema_type: SchemaType,

    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Required properties (for object type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Property definitions (for object type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, ParameterSchema>>,

    /// Array item schema
    #[serde(rename = "item", skip_serializing_if = "Option::is_none")]
    pub item: Option<Box<ParameterSchema>>,

    /// Enum values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,

    /// Minimum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    /// Maximum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    /// Minimum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

/// Schema types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
}

impl ParameterSchema {
    /// Create a new string schema
    pub fn string() -> Self {
        Self {
            schema_type: SchemaType::String,
            description: None,
            required: None,
            properties: None,
            item: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            default: None,
        }
    }

    /// Create a new number schema
    pub fn number() -> Self {
        Self {
            schema_type: SchemaType::Number,
            description: None,
            required: None,
            properties: None,
            item: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            default: None,
        }
    }

    /// Create a new integer schema
    pub fn integer() -> Self {
        Self {
            schema_type: SchemaType::Integer,
            description: None,
            required: None,
            properties: None,
            item: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            default: None,
        }
    }

    /// Create a new boolean schema
    pub fn boolean() -> Self {
        Self {
            schema_type: SchemaType::Boolean,
            description: None,
            required: None,
            properties: None,
            item: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            default: None,
        }
    }

    /// Create a new array schema
    pub fn array(item: ParameterSchema) -> Self {
        Self {
            schema_type: SchemaType::Array,
            description: None,
            required: None,
            properties: None,
            item: Some(Box::new(item)),
            enum_values: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            default: None,
        }
    }

    /// Create a new object schema
    pub fn object(properties: HashMap<String, ParameterSchema>) -> Self {
        Self {
            schema_type: SchemaType::Object,
            description: None,
            required: None,
            properties: Some(properties),
            item: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            default: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set required properties
    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }

    /// Set minimum value
    pub fn with_minimum(mut self, minimum: f64) -> Self {
        self.minimum = Some(minimum);
        self
    }

    /// Set maximum value
    pub fn with_maximum(mut self, maximum: f64) -> Self {
        self.maximum = Some(maximum);
        self
    }

    /// Set min length
    pub fn with_min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    /// Set max length
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set enum values
    pub fn with_enum(mut self, values: Vec<Value>) -> Self {
        self.enum_values = Some(values);
        self
    }

    /// Set default value
    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }

    /// Validate a value against this schema
    pub fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        match self.schema_type {
            SchemaType::String => self.validate_string(value),
            SchemaType::Number => self.validate_number(value),
            SchemaType::Integer => self.validate_integer(value),
            SchemaType::Boolean => self.validate_boolean(value),
            SchemaType::Array => self.validate_array(value),
            SchemaType::Object => self.validate_object(value),
        }
    }

    fn validate_string(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_string() {
            return Err(ValidationError::TypeMismatch {
                expected: "string".to_string(),
                found: value_type_name(value),
            });
        }

        let s = value.as_str().unwrap();

        if let Some(min) = self.min_length {
            if s.len() < min {
                return Err(ValidationError::MinLength {
                    min,
                    found: s.len(),
                });
            }
        }

        if let Some(max) = self.max_length {
            if s.len() > max {
                return Err(ValidationError::MaxLength {
                    max,
                    found: s.len(),
                });
            }
        }

        if let Some(ref enum_values) = self.enum_values {
            if !enum_values.contains(value) {
                return Err(ValidationError::InvalidEnumValue {
                    allowed: enum_values.clone(),
                    found: value.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_number(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_number() {
            return Err(ValidationError::TypeMismatch {
                expected: "number".to_string(),
                found: value_type_name(value),
            });
        }

        let n = value.as_f64().unwrap();

        if let Some(min) = self.minimum {
            if n < min {
                return Err(ValidationError::Minimum { min, found: n });
            }
        }

        if let Some(max) = self.maximum {
            if n > max {
                return Err(ValidationError::Maximum { max, found: n });
            }
        }

        Ok(())
    }

    fn validate_integer(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_i64() && !value.is_u64() {
            return Err(ValidationError::TypeMismatch {
                expected: "integer".to_string(),
                found: value_type_name(value),
            });
        }

        let n = if let Some(i) = value.as_i64() {
            i as f64
        } else if let Some(u) = value.as_u64() {
            u as f64
        } else {
            return Err(ValidationError::TypeMismatch {
                expected: "integer".to_string(),
                found: value_type_name(value),
            });
        };

        if let Some(min) = self.minimum {
            if n < min {
                return Err(ValidationError::Minimum { min, found: n });
            }
        }

        if let Some(max) = self.maximum {
            if n > max {
                return Err(ValidationError::Maximum { max, found: n });
            }
        }

        Ok(())
    }

    fn validate_boolean(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_boolean() {
            return Err(ValidationError::TypeMismatch {
                expected: "boolean".to_string(),
                found: value_type_name(value),
            });
        }
        Ok(())
    }

    fn validate_array(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_array() {
            return Err(ValidationError::TypeMismatch {
                expected: "array".to_string(),
                found: value_type_name(value),
            });
        }

        let arr = value.as_array().unwrap();

        if let Some(min) = self.min_length {
            if arr.len() < min {
                return Err(ValidationError::MinLength {
                    min,
                    found: arr.len(),
                });
            }
        }

        if let Some(max) = self.max_length {
            if arr.len() > max {
                return Err(ValidationError::MaxLength {
                    max,
                    found: arr.len(),
                });
            }
        }

        if let Some(ref item_schema) = self.item {
            for (i, item) in arr.iter().enumerate() {
                item_schema
                    .validate(item)
                    .map_err(|e| ValidationError::ItemInvalid {
                        index: i,
                        cause: Box::new(e),
                    })?;
            }
        }

        Ok(())
    }

    fn validate_object(&self, value: &Value) -> Result<(), ValidationError> {
        if !value.is_object() {
            return Err(ValidationError::TypeMismatch {
                expected: "object".to_string(),
                found: value_type_name(value),
            });
        }

        let obj = value.as_object().unwrap();

        if let Some(ref required) = self.required {
            for prop in required {
                if !obj.contains_key(prop) {
                    return Err(ValidationError::MissingProperty {
                        property: prop.clone(),
                    });
                }
            }
        }

        if let Some(ref properties) = self.properties {
            for (key, val) in obj.iter() {
                if let Some(schema) = properties.get(key) {
                    schema.validate(val).map_err(|e| ValidationError::PropertyInvalid {
                        property: key.clone(),
                        cause: Box::new(e),
                    })?;
                }
            }
        }

        Ok(())
    }
}

/// Get the type name of a JSON value
fn value_type_name(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => {
            if value.is_i64() {
                "integer".to_string()
            } else {
                "number".to_string()
            }
        }
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

/// Parameter validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Missing required property: {property}")]
    MissingProperty { property: String },

    #[error("Property '{property}' is invalid: {cause}")]
    PropertyInvalid {
        property: String,
        cause: Box<ValidationError>,
    },

    #[error("Array item at index {index} is invalid: {cause}")]
    ItemInvalid {
        index: usize,
        cause: Box<ValidationError>,
    },

    #[error("Minimum length constraint violated: min {min}, found {found}")]
    MinLength { min: usize, found: usize },

    #[error("Maximum length constraint violated: max {max}, found {found}")]
    MaxLength { max: usize, found: usize },

    #[error("Minimum value constraint violated: min {min}, found {found}")]
    Minimum { min: f64, found: f64 },

    #[error("Maximum value constraint violated: max {max}, found {found}")]
    Maximum { max: f64, found: f64 },

    #[error("Invalid enum value: must be one of {allowed:?}, found {found}")]
    InvalidEnumValue { allowed: Vec<Value>, found: Value },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_schema() {
        let schema = ParameterSchema::string()
            .with_min_length(1)
            .with_max_length(10);

        assert!(schema.validate(&serde_json::json!("hello")).is_ok());
        assert!(schema.validate(&serde_json::json!("")).is_err());
        assert!(schema.validate(&serde_json::json!("this is too long")).is_err());
        assert!(schema.validate(&serde_json::json!(123)).is_err());
    }

    #[test]
    fn test_number_schema() {
        let schema = ParameterSchema::number()
            .with_minimum(0.0)
            .with_maximum(100.0);

        assert!(schema.validate(&serde_json::json!(50.5)).is_ok());
        assert!(schema.validate(&serde_json::json!(-1.0)).is_err());
        assert!(schema.validate(&serde_json::json!(101.0)).is_err());
    }

    #[test]
    fn test_object_schema() {
        let mut props = HashMap::new();
        props.insert("name".to_string(), ParameterSchema::string());
        props.insert("age".to_string(), ParameterSchema::integer());

        let schema = ParameterSchema::object(props)
            .with_required(vec!["name".to_string()]);

        assert!(schema.validate(&serde_json::json!({"name": "Alice"})).is_ok());
        assert!(schema.validate(&serde_json::json!({})).is_err());
        assert!(schema.validate(&serde_json::json!({"name": 123})).is_err());
    }

    #[test]
    fn test_array_schema() {
        let schema = ParameterSchema::array(ParameterSchema::string())
            .with_min_length(1)
            .with_max_length(5);

        assert!(schema.validate(&serde_json::json!(["a", "b"])).is_ok());
        assert!(schema.validate(&serde_json::json!([])).is_err());
        assert!(schema.validate(&serde_json::json!(["a", "b", "c", "d", "e", "f"])).is_err());
        assert!(schema.validate(&serde_json::json!([1, 2, 3])).is_err());
    }
}
