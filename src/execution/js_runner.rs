use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct JsRunner;

impl JsRunner {
    pub fn execute(file_path: &Path, input: Option<&str>) -> Result<String> {
        // Check if file exists
        if !file_path.exists() {
            return Err(anyhow::anyhow!("Exercise file not found: {}", file_path.display()));
        }
        
        let mut child = Command::new("node")
            .arg(file_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to execute node command")?;

        if let Some(input_str) = input {
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(input_str.as_bytes())
                    .context("Failed to write to stdin")?;
                // Close stdin to signal EOF
                drop(stdin);
            }
        } else {
            // Close stdin if no input
            drop(child.stdin.take());
        }

        let output = child.wait_with_output()
            .context("Failed to wait for process")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Execution error: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
}
