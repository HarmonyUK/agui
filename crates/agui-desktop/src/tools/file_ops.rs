//! File operation tools
//!
//! Provides tools for reading, writing, listing, and checking files.

use super::{Tool, ToolError, ToolResult};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Read file contents
#[derive(Debug, Clone, Copy)]
pub struct ReadFile;

impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file"
    }

    fn parameters_schema(&self) -> &super::schema::ParameterSchema {
        static SCHEMA: std::sync::OnceLock<super::schema::ParameterSchema> = std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let mut props = HashMap::new();
            props.insert(
                "path".to_string(),
                super::schema::ParameterSchema::string()
                    .with_description("Path to the file to read"),
            );
            super::schema::ParameterSchema::object(props)
                .with_required(vec!["path".to_string()])
        })
    }

    fn execute(&self, params: &Value) -> ToolResult {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::ExecutionFailed("Missing 'path' parameter".to_string()))?;

        let content = fs::read_to_string(path)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read file: {}", e)))?;

        Ok(json!({
            "path": path,
            "content": content,
        }))
    }
}

/// Write content to a file
#[derive(Debug, Clone, Copy)]
pub struct WriteFile;

impl Tool for WriteFile {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file"
    }

    fn parameters_schema(&self) -> &super::schema::ParameterSchema {
        static SCHEMA: std::sync::OnceLock<super::schema::ParameterSchema> = std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let mut props = HashMap::new();
            props.insert(
                "path".to_string(),
                super::schema::ParameterSchema::string()
                    .with_description("Path to the file to write"),
            );
            props.insert(
                "content".to_string(),
                super::schema::ParameterSchema::string()
                    .with_description("Content to write to the file"),
            );
            super::schema::ParameterSchema::object(props)
                .with_required(vec!["path".to_string(), "content".to_string()])
        })
    }

    fn execute(&self, params: &Value) -> ToolResult {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::ExecutionFailed("Missing 'path' parameter".to_string()))?;

        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::ExecutionFailed("Missing 'content' parameter".to_string()))?;

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create directories: {}", e)))?;
        }

        fs::write(path, content)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write file: {}", e)))?;

        Ok(json!({
            "path": path,
            "bytes_written": content.len(),
            "success": true,
        }))
    }
}

/// List directory contents
#[derive(Debug, Clone, Copy)]
pub struct ListDirectory;

impl Tool for ListDirectory {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List the contents of a directory"
    }

    fn parameters_schema(&self) -> &super::schema::ParameterSchema {
        static SCHEMA: std::sync::OnceLock<super::schema::ParameterSchema> = std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let mut props = HashMap::new();
            props.insert(
                "path".to_string(),
                super::schema::ParameterSchema::string()
                    .with_description("Path to the directory to list"),
            );
            props.insert(
                "recursive".to_string(),
                super::schema::ParameterSchema::boolean()
                    .with_description("Whether to list recursively")
                    .with_default(json!(false)),
            );
            super::schema::ParameterSchema::object(props)
                .with_required(vec!["path".to_string()])
        })
    }

    fn execute(&self, params: &Value) -> ToolResult {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::ExecutionFailed("Missing 'path' parameter".to_string()))?;

        let recursive = params
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let entries = if recursive {
            list_recursive(path)?
        } else {
            list_flat(path)?
        };

        Ok(json!({
            "path": path,
            "entries": entries,
        }))
    }
}

/// Check if a file or directory exists
#[derive(Debug, Clone, Copy)]
pub struct FileExists;

impl Tool for FileExists {
    fn name(&self) -> &str {
        "file_exists"
    }

    fn description(&self) -> &str {
        "Check if a file or directory exists"
    }

    fn parameters_schema(&self) -> &super::schema::ParameterSchema {
        static SCHEMA: std::sync::OnceLock<super::schema::ParameterSchema> = std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let mut props = HashMap::new();
            props.insert(
                "path".to_string(),
                super::schema::ParameterSchema::string()
                    .with_description("Path to check"),
            );
            super::schema::ParameterSchema::object(props)
                .with_required(vec!["path".to_string()])
        })
    }

    fn execute(&self, params: &Value) -> ToolResult {
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::ExecutionFailed("Missing 'path' parameter".to_string()))?;

        let path_obj = Path::new(path);
        let exists = path_obj.exists();

        let mut result = json!({
            "path": path,
            "exists": exists,
        });

        if exists {
            let metadata = fs::metadata(path)
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get metadata: {}", e)))?;

            result["is_file"] = json!(metadata.is_file());
            result["is_directory"] = json!(metadata.is_dir());
            let size = metadata.len();
            result["size"] = json!(size);
        }

        Ok(result)
    }
}

/// List directory entries (non-recursive)
fn list_flat(path: &str) -> Result<Vec<Value>, ToolError> {
    let entries = fs::read_dir(path)
        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read directory: {}", e)))?
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let path = entry.path();
            let metadata = entry.metadata().ok();

            let mut obj = json!({
                "name": entry.file_name().to_string_lossy(),
                "path": path.to_string_lossy(),
            });

            if let Some(meta) = metadata {
                obj["is_file"] = json!(meta.is_file());
                obj["is_directory"] = json!(meta.is_dir());
                let size = meta.len();
                obj["size"] = json!(size);
            }

            obj
        })
        .collect();

    Ok(entries)
}

/// List directory entries recursively
fn list_recursive(path: &str) -> Result<Vec<Value>, ToolError> {
    let mut entries = Vec::new();

    fn visit(path: &Path, entries: &mut Vec<Value>) -> std::io::Result<()> {
        let entry = path.metadata()?;
        let mut obj = json!({
            "name": path.file_name().unwrap_or_default().to_string_lossy(),
            "path": path.to_string_lossy(),
            "is_file": entry.is_file(),
            "is_directory": entry.is_dir(),
        });

        let size = entry.len();
        obj["size"] = json!(size);

        entries.push(obj);

        if path.is_dir() {
            for sub_entry in fs::read_dir(path)? {
                let sub_entry = sub_entry?;
                visit(&sub_entry.path(), entries)?;
            }
        }

        Ok(())
    }

    visit(Path::new(path), &mut entries)
        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to traverse directory: {}", e)))?;

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, World!").unwrap();

        let tool = ReadFile;
        let result = tool.execute(&json!({
            "path": file_path.to_string_lossy(),
        }));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result["content"], "Hello, World!\n");
    }

    #[test]
    fn test_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let tool = WriteFile;
        let result = tool.execute(&json!({
            "path": file_path.to_string_lossy(),
            "content": "Test content",
        }));

        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[test]
    fn test_write_file_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nested/dir/test.txt");

        let tool = WriteFile;
        let result = tool.execute(&json!({
            "path": file_path.to_string_lossy(),
            "content": "Test",
        }));

        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn test_list_directory() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("file1.txt")).unwrap();
        File::create(temp_dir.path().join("file2.txt")).unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let tool = ListDirectory;
        let result = tool.execute(&json!({
            "path": temp_dir.path().to_string_lossy(),
        }));

        assert!(result.is_ok());
        let result = result.unwrap();
        let entries = result["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 3); // file1.txt, file2.txt, subdir
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let tool = FileExists;

        // Existing file
        let result = tool.execute(&json!({
            "path": file_path.to_string_lossy(),
        }));
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["exists"], true);

        // Non-existing file
        let result = tool.execute(&json!({
            "path": temp_dir.path().join("nonexistent.txt").to_string_lossy(),
        }));
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["exists"], false);
    }
}
