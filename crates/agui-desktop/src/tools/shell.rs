//! Shell command execution tool
//!
//! Provides tool for running shell commands with timeout support.

use super::{Tool, ToolError, ToolResult};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Default timeout for shell commands (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Run a shell command
#[derive(Debug, Clone, Copy)]
pub struct RunCommand;

impl Tool for RunCommand {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command with optional timeout and working directory"
    }

    fn parameters_schema(&self) -> &super::schema::ParameterSchema {
        static SCHEMA: std::sync::OnceLock<super::schema::ParameterSchema> = std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let mut props = HashMap::new();
            props.insert(
                "command".to_string(),
                super::schema::ParameterSchema::string()
                    .with_description("Command to execute"),
            );
            props.insert(
                "args".to_string(),
                super::schema::ParameterSchema::array(super::schema::ParameterSchema::string())
                    .with_description("Command arguments")
                    .with_default(json!([])),
            );
            props.insert(
                "working_dir".to_string(),
                super::schema::ParameterSchema::string()
                    .with_description("Working directory for the command"),
            );
            props.insert(
                "timeout_secs".to_string(),
                super::schema::ParameterSchema::integer()
                    .with_description("Timeout in seconds (0 for no timeout)")
                    .with_minimum(0.0)
                    .with_default(json!(30)),
            );
            props.insert(
                "environment".to_string(),
                super::schema::ParameterSchema::object(HashMap::new())
                    .with_description("Environment variables for the command"),
            );
            super::schema::ParameterSchema::object(props)
                .with_required(vec!["command".to_string()])
        })
    }

    fn execute(&self, params: &Value) -> ToolResult {
        let command = params
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::ExecutionFailed("Missing 'command' parameter".to_string()))?;

        let args: Vec<String> = params
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let working_dir = params.get("working_dir").and_then(|v| v.as_str()).map(|s| s.to_string());

        let timeout_secs = params
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        let environment: HashMap<String, String> = params
            .get("environment")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        // Execute async command
        // Use block_in_place to avoid nested runtime issues
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context - spawn a blocking task
                tokio::task::block_in_place(move || {
                    handle.block_on(execute_command_async(
                        command,
                        args,
                        working_dir,
                        timeout_secs,
                        environment,
                    ))
                })
            }
            Err(_) => {
                // Not in an async context - create a new runtime
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create runtime: {}", e)))?;
                rt.block_on(execute_command_async(
                    command,
                    args,
                    working_dir,
                    timeout_secs,
                    environment,
                ))
            }
        }
    }
}

/// Execute a command with the given parameters using async I/O
///
/// Uses tokio::process for async I/O with proper timeout handling.
/// The timeout applies to the entire command execution, and the process
/// is properly killed if the timeout is exceeded.
async fn execute_command_async(
    command: &str,
    args: Vec<String>,
    working_dir: Option<String>,
    timeout_secs: u64,
    environment: HashMap<String, String>,
) -> ToolResult {
    let mut cmd = Command::new(command);

    // Add arguments
    cmd.args(&args);

    // Set working directory if provided
    if let Some(dir) = &working_dir {
        cmd.current_dir(dir);
    }

    // Set environment variables
    for (key, value) in environment {
        cmd.env(key, value);
    }

    // Capture stdout and stderr
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // Kill process on drop
    cmd.kill_on_drop(true);

    // Spawn the process
    let mut child = cmd
        .spawn()
        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to spawn command: {}", e)))?;

    // Take stdout and stderr handles
    let stdout = child.stdout.take().ok_or_else(|| {
        ToolError::ExecutionFailed("Failed to capture stdout".to_string())
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        ToolError::ExecutionFailed("Failed to capture stderr".to_string())
    })?;

    // Setup timeout if specified
    let timeout_duration = if timeout_secs > 0 {
        Some(Duration::from_secs(timeout_secs))
    } else {
        None
    };

    // Read output and wait for process completion with timeout
    let result_future = async {
        // Read stdout and stderr concurrently
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let stdout_task = tokio::spawn(async move {
            let mut lines = Vec::new();
            let mut line_stream = stdout_reader.lines();
            while let Ok(Some(line)) = line_stream.next_line().await {
                lines.push(line);
            }
            lines
        });

        let stderr_task = tokio::spawn(async move {
            let mut lines = Vec::new();
            let mut line_stream = stderr_reader.lines();
            while let Ok(Some(line)) = line_stream.next_line().await {
                lines.push(line);
            }
            lines
        });

        // Wait for process to complete
        let status = child.wait().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to wait for command: {}", e)))?;

        // Wait for output tasks to complete
        let stdout_lines = stdout_task.await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read stdout: {}", e)))?;
        let stderr_lines = stderr_task.await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read stderr: {}", e)))?;

        Ok::<_, ToolError>((status, stdout_lines, stderr_lines))
    };

    // Apply timeout if specified
    let (status, stdout_lines, stderr_lines) = if let Some(duration) = timeout_duration {
        match tokio::time::timeout(duration, result_future).await {
            Ok(result) => result?,
            Err(_) => {
                // Timeout exceeded - child process is automatically killed due to kill_on_drop
                return Err(ToolError::Timeout);
            }
        }
    } else {
        result_future.await?
    };

    let stdout_text = stdout_lines.join("\n");
    let stderr_text = stderr_lines.join("\n");

    let result = json!({
        "command": command,
        "args": args,
        "exit_code": status.code(),
        "success": status.success(),
        "stdout": stdout_text,
        "stderr": stderr_text,
        "timeout_secs": timeout_secs,
    });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_command_echo() {
        let tool = RunCommand;
        let result = tool.execute(&json!({
            "command": "echo",
            "args": ["hello", "world"],
        }));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["exit_code"], 0);
        assert_eq!(result["stdout"], "hello world");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_command_with_working_dir() {
        let tool = RunCommand;
        let result = tool.execute(&json!({
            "command": "pwd",
            "working_dir": "/tmp",
        }));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["stdout"], "/tmp");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_command_failure() {
        let tool = RunCommand;
        let result = tool.execute(&json!({
            "command": "false",
        }));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result["success"], false);
        assert!(result["exit_code"].is_number() || result["exit_code"].is_null());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_command_nonexistent() {
        let tool = RunCommand;
        let result = tool.execute(&json!({
            "command": "this-command-does-not-exist-12345",
        }));

        // The command fails to spawn, returning an error
        assert!(result.is_err());
        assert!(matches!(result, Err(ToolError::ExecutionFailed(_))));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_command_with_env() {
        let tool = RunCommand;
        let result = tool.execute(&json!({
            "command": "sh",
            "args": ["-c", "echo $TEST_VAR"],
            "environment": {
                "TEST_VAR": "test_value"
            },
        }));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["stdout"], "test_value");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_command_timeout() {
        let tool = RunCommand;
        let result = tool.execute(&json!({
            "command": "sleep",
            "args": ["10"],
            "timeout_secs": 1,
        }));

        // Should timeout
        assert!(matches!(result, Err(ToolError::Timeout)));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_command_no_timeout() {
        let tool = RunCommand;
        let result = tool.execute(&json!({
            "command": "sleep",
            "args": ["0"],
            "timeout_secs": 0,
        }));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result["success"], true);
    }
}
