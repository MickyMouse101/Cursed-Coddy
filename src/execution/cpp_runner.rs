use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct CppRunner;

impl CppRunner {
    pub fn execute(file_path: &Path, input: Option<&str>) -> Result<String> {
        // Check if file exists
        if !file_path.exists() {
            return Err(anyhow::anyhow!("Exercise file not found: {}", file_path.display()));
        }
        
        let exe_path = file_path.with_extension("");

        // Compile
        let compile_output = Command::new("g++")
            .arg("-o")
            .arg(&exe_path)
            .arg(file_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute g++ command")?;

        if !compile_output.status.success() {
            let stderr = String::from_utf8_lossy(&compile_output.stderr);
            return Err(anyhow::anyhow!("Compilation error: {}", stderr));
        }

        // Run
        let mut child = Command::new(&exe_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to execute compiled program")?;

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

        let run_output = child.wait_with_output()
            .context("Failed to wait for process")?;

        // Cleanup
        let _ = std::fs::remove_file(&exe_path);

        if !run_output.status.success() {
            let stderr = String::from_utf8_lossy(&run_output.stderr);
            return Err(anyhow::anyhow!("Runtime error: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&run_output.stdout);
        Ok(stdout.to_string())
    }
}
