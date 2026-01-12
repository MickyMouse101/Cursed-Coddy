use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct RustRunner;

impl RustRunner {
    pub fn execute(file_path: &Path, input: Option<&str>) -> Result<String> {
        if !file_path.exists() {
            return Err(anyhow::anyhow!("Exercise file not found: {}", file_path.display()));
        }
        
        // Read the code to detect dependencies
        let code = std::fs::read_to_string(file_path)
            .context("Failed to read exercise file")?;
        
        let dependencies = Self::detect_dependencies(&code);
        
        // Create a temporary Cargo project
        let cargo_project_dir = file_path.parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(format!("cargo_exercise_{}", 
                file_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("exercise")));
        
        // Clean up any existing project directory
        if cargo_project_dir.exists() {
            let _ = std::fs::remove_dir_all(&cargo_project_dir);
        }
        
        std::fs::create_dir_all(&cargo_project_dir)
            .context("Failed to create Cargo project directory")?;
        
        // Create Cargo.toml
        let cargo_toml = Self::generate_cargo_toml(&dependencies);
        std::fs::write(cargo_project_dir.join("Cargo.toml"), cargo_toml)
            .context("Failed to write Cargo.toml")?;
        
        // Create src directory
        let src_dir = cargo_project_dir.join("src");
        std::fs::create_dir_all(&src_dir)
            .context("Failed to create src directory")?;
        
        // Copy user code to src/main.rs
        std::fs::write(src_dir.join("main.rs"), &code)
            .context("Failed to write main.rs")?;
        
        // Run with cargo (show compiler output)
        let mut child = Command::new("cargo")
            .arg("run")
            .current_dir(&cargo_project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to execute cargo run")?;

        if let Some(input_str) = input {
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(input_str.as_bytes())
                    .context("Failed to write to stdin")?;
                drop(stdin);
            }
        } else {
            drop(child.stdin.take());
        }

        let run_output = child.wait_with_output()
            .context("Failed to wait for cargo process")?;

        // Always show compiler output (stderr contains compilation messages)
        let stderr = String::from_utf8_lossy(&run_output.stderr);
        if !stderr.trim().is_empty() {
            eprintln!("{}", stderr);
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&cargo_project_dir);

        if !run_output.status.success() {
            return Err(anyhow::anyhow!("Compilation or runtime error: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&run_output.stdout);
        Ok(stdout.to_string())
    }
    
    fn detect_dependencies(code: &str) -> Vec<String> {
        let mut deps = Vec::new();
        
        // Common crate patterns to detect
        let crate_patterns = vec![
            ("rand", "rand = \"0.8\""),
            ("serde", "serde = { version = \"1.0\", features = [\"derive\"] }"),
            ("serde_json", "serde_json = \"1.0\""),
            ("tokio", "tokio = { version = \"1\", features = [\"full\"] }"),
            ("reqwest", "reqwest = { version = \"0.12\", features = [\"json\", \"blocking\"] }"),
            ("clap", "clap = { version = \"4.5\", features = [\"derive\"] }"),
        ];
        
        for (crate_name, dep_line) in crate_patterns {
            if code.contains(&format!("use {}::", crate_name)) || 
               code.contains(&format!("use {};", crate_name)) ||
               code.contains(&format!("extern crate {}", crate_name)) {
                deps.push(dep_line.to_string());
            }
        }
        
        deps
    }
    
    fn generate_cargo_toml(dependencies: &[String]) -> String {
        let deps_section = if dependencies.is_empty() {
            String::new()
        } else {
            format!("\n[dependencies]\n{}\n", dependencies.join("\n"))
        };
        
        format!(
            r#"[package]
name = "exercise"
version = "0.1.0"
edition = "2021"
{}"#,
            deps_section
        )
    }
}
