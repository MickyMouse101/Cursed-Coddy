use crate::cli::colors::Colors;
use crate::config::{Difficulty, Language, LessonType};
use crate::ollama::{formatter::GeneratedContent, ruleset::Ruleset};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<serde_json::Value>,
}

pub struct Generator {
    base_url: String,
    model: String,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            base_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string()),
            model: std::env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "qwen2.5-coder:7b".to_string()),
        }
    }

    fn check_system_gpu(&self) -> bool {
        // Check for NVIDIA GPU
        if std::process::Command::new("nvidia-smi")
            .output()
            .is_ok()
        {
            return true;
        }
        
        // Check for AMD GPU (ROCm)
        if std::process::Command::new("rocm-smi")
            .output()
            .is_ok()
        {
            return true;
        }
        
        // Check for Intel GPU
        if std::process::Command::new("intel_gpu_top")
            .output()
            .is_ok()
        {
            return true;
        }
        
        // Check for GPU in /dev (common GPU device files)
        let gpu_devices = [
            "/dev/dri/renderD128",
            "/dev/dri/card0",
            "/dev/kfd",
        ];
        
        for device in &gpu_devices {
            if std::path::Path::new(device).exists() {
                return true;
            }
        }
        
        false
    }

    fn check_device_info(&self) -> String {
        // Method 0: Check system for GPU availability first (most reliable)
        if self.check_system_gpu() {
            // If GPU is available, assume Ollama is using it (Ollama auto-detects GPU)
            // We'll verify from API responses if possible, but system check is primary
        }
        
        let client = match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
        {
            Ok(c) => c,
            Err(_) => {
                // If we can't connect but system has GPU, assume GPU
                if self.check_system_gpu() {
                    return "GPU".to_string();
                }
                return "CPU".to_string();
            },
        };
        
        let base_url = self.base_url.trim_end_matches('/');
        
        // Method 1: Check model show endpoint for detailed info
        if let Ok(response) = client
            .post(&format!("{}/api/show", base_url))
            .json(&serde_json::json!({
                "name": self.model
            }))
            .timeout(std::time::Duration::from_secs(3))
            .send()
        {
            if let Ok(json) = response.json::<serde_json::Value>() {
                // Check various fields that might indicate GPU usage
                let json_str = json.to_string().to_lowercase();
                if json_str.contains("gpu") || json_str.contains("cuda") || json_str.contains("metal") || 
                   json_str.contains("vulkan") || json_str.contains("rocm") || json_str.contains("rocm0") {
                    return "GPU".to_string();
                }
                
                // Check for specific GPU-related fields
                if let Some(details) = json.get("details") {
                    let details_str = details.to_string().to_lowercase();
                    if details_str.contains("gpu") || details_str.contains("cuda") || details_str.contains("metal") ||
                       details_str.contains("rocm") {
                        return "GPU".to_string();
                    }
                }
                
                // Check modelfile for GPU settings
                if let Some(modelfile) = json.get("modelfile") {
                    if let Some(modelfile_str) = modelfile.as_str() {
                        let modelfile_lower = modelfile_str.to_lowercase();
                        if modelfile_lower.contains("gpu") || modelfile_lower.contains("cuda") ||
                           modelfile_lower.contains("rocm") {
                            return "GPU".to_string();
                        }
                    }
                }
            }
        }
        
        // Method 2: Check ps endpoint for running models
        if let Ok(response) = client
            .get(&format!("{}/api/ps", base_url))
            .timeout(std::time::Duration::from_secs(3))
            .send()
        {
            if let Ok(json) = response.json::<serde_json::Value>() {
                // Check if any model is using GPU
                if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                    for model in models {
                        // Check the entire model object for GPU indicators
                        let model_str = model.to_string().to_lowercase();
                        if model_str.contains("gpu") || model_str.contains("cuda") || model_str.contains("metal") ||
                           model_str.contains("rocm") || model_str.contains("rocm0") {
                            return "GPU".to_string();
                        }
                        
                        // Check details field if present
                        if let Some(details) = model.get("details") {
                            let details_str = details.to_string().to_lowercase();
                            if details_str.contains("gpu") || details_str.contains("cuda") || details_str.contains("metal") ||
                               details_str.contains("rocm") {
                                return "GPU".to_string();
                            }
                        }
                    }
                }
            }
        }
        
        // Method 3: Check environment variable (Ollama sets OLLAMA_NUM_GPU)
        if std::env::var("OLLAMA_NUM_GPU").is_ok() {
            if let Ok(num_gpu) = std::env::var("OLLAMA_NUM_GPU") {
                if num_gpu.parse::<i32>().unwrap_or(0) > 0 {
                    return "GPU".to_string();
                }
            }
        }
        
        // Method 4: If system has GPU available, assume Ollama is using it
        // (Ollama automatically uses GPU if available)
        if self.check_system_gpu() {
            return "GPU".to_string();
        }
        
        // Fallback: assume CPU
        "CPU".to_string()
    }
    
    fn detect_device_from_response(&self, response_json: &serde_json::Value) -> Option<String> {
        // Check the response for device information
        let response_str = response_json.to_string().to_lowercase();
        
        // Check for GPU indicators in the response (including ROCm)
        if response_str.contains("gpu") || response_str.contains("cuda") || response_str.contains("metal") ||
           response_str.contains("vulkan") || response_str.contains("rocm") || response_str.contains("rocm0") {
            return Some("GPU".to_string());
        }
        
        // Check specific fields that might contain device info
        if let Some(context) = response_json.get("context") {
            let context_str = context.to_string().to_lowercase();
            if context_str.contains("gpu") || context_str.contains("cuda") || context_str.contains("rocm") {
                return Some("GPU".to_string());
            }
        }
        
        // Check for performance indicators (GPU is typically faster)
        // If system has GPU, assume it's being used even if not explicitly mentioned
        if self.check_system_gpu() {
            return Some("GPU".to_string());
        }
        
        None
    }

    pub fn generate(
        &self,
        language: Language,
        difficulty: Difficulty,
        lesson_type: LessonType,
        topic: &str,
    ) -> Result<GeneratedContent> {
        // Check device info before generation (initial guess)
        let device = self.check_device_info();
        let device_label = if device.contains("GPU") { 
            Colors::label_gpu("GPU")
        } else { 
            Colors::label_cpu("CPU")
        };
        
        println!("{}", Colors::info("Generating lesson content (this may take 30-60 seconds)..."));
        println!("{} {}", device_label, Colors::muted(&device));
        
        let prompt = Ruleset::generate_prompt(language, difficulty, lesson_type, topic);

        // Limit response length to prevent timeouts, but ensure enough tokens for complete JSON
        let options = serde_json::json!({
            "num_predict": 7000, // Increased to ensure complete JSON responses
            "temperature": 0.5,   // Lower temperature for more consistent JSON output
        });

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            options: Some(options),
        };

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;
        
        // Validate URL format
        let url = format!("{}/api/generate", self.base_url.trim_end_matches('/'));
        
        // Check if Ollama is reachable
        let test_url = format!("{}/api/tags", self.base_url.trim_end_matches('/'));
        let _ = client.get(&test_url).timeout(std::time::Duration::from_secs(5)).send()
            .context("Cannot connect to Ollama. Make sure Ollama is running on the specified URL.")?;
        
        // Create animated progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(format!("Generating with {}...", device));
        pb.enable_steady_tick(Duration::from_millis(100));
        
        // Start request in a thread to allow progress bar to animate
        let request_clone = request.clone();
        let url_clone = url.clone();
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        
        let handle = thread::spawn(move || {
            // Create a new client in the thread since reqwest::blocking::Client is not Clone
            let thread_client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build();
            
            let result = match thread_client {
                Ok(c) => c
                    .post(&url_clone)
                    .json(&request_clone)
                    .timeout(std::time::Duration::from_secs(120))
                    .send(),
                Err(e) => Err(e),
            };
            done_clone.store(true, Ordering::Relaxed);
            result
        });
        
        // Animate progress bar while waiting
        while !done.load(Ordering::Relaxed) {
            pb.tick();
            thread::sleep(Duration::from_millis(50));
        }
        
        let response = match handle.join() {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                pb.finish_and_clear();
                if e.is_timeout() {
                    return Err(anyhow::anyhow!(
                        "Request timed out after 120 seconds. The model may be too slow. Try using a faster model or reducing the prompt complexity."
                    ));
                }
                if e.is_connect() {
                    return Err(anyhow::anyhow!(
                        "Cannot connect to Ollama at {}. Make sure Ollama is running: 'ollama serve'",
                        self.base_url
                    ));
                }
                return Err(anyhow::anyhow!("Failed to connect to Ollama: {}", e));
            }
            Err(_) => {
                pb.finish_and_clear();
                return Err(anyhow::anyhow!("Thread error while generating"));
            }
        };
        

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Ollama API error: {}",
                response.status()
            ));
        }

        // Get the raw response text first
        let response_text = response
            .text()
            .context("Failed to read Ollama response")?;

        // Try to parse as JSON response structure
        // Ollama returns a JSON object with a "response" field containing the text
        let ollama_response: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse Ollama response as JSON")?;

        // Update device detection based on actual response (more accurate)
        let final_device = if let Some(detected_device) = self.detect_device_from_response(&ollama_response) {
            detected_device
        } else {
            device
        };
        
               pb.finish_with_message(format!("[OK] Generated with {}", final_device));

        // Extract the response text - handle different possible formats
        let response_content = if let Some(response_val) = ollama_response.get("response") {
            match response_val {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                    // If response is an object/array, serialize it back to JSON string
                    serde_json::to_string(response_val)
                        .unwrap_or_else(|_| ollama_response.to_string())
                }
                _ => response_val.to_string(),
            }
        } else {
            // If no "response" field, check if the whole response is the content
            // or if there's another field that might contain it
            if ollama_response.is_string() {
                ollama_response.as_str().unwrap_or("").to_string()
            } else {
                // Try to find common alternative fields
                ollama_response
                    .get("text")
                    .or_else(|| ollama_response.get("content"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| ollama_response.to_string())
            }
        };

        // Debug: log the extracted content (first 200 chars) if extraction fails later
        if response_content.is_empty() {
            return Err(anyhow::anyhow!(
                "Empty response from Ollama. Full response: {}",
                serde_json::to_string_pretty(&ollama_response).unwrap_or_default()
            ));
        }

        // Try to extract JSON from the response content
        let json_str = match Self::extract_json(&response_content) {
            Ok(json) => json,
            Err(_e) => {
                // If extraction fails, try one more time with the raw response
                eprintln!("{}", Colors::label_warn("WARN"));
                eprintln!("{}", Colors::warning("JSON extraction failed, trying alternative methods..."));
                // Try extracting from the full ollama response as fallback
                if let Some(response_val) = ollama_response.get("response") {
                    if let Some(s) = response_val.as_str() {
                        if let Ok(json) = Self::extract_json(s) {
                            json
                        } else {
                            // If all JSON extraction fails, create a fallback lesson instead of erroring
                            eprintln!("{}", Colors::label_warn("WARN"));
                            eprintln!("{}", Colors::warning("Could not extract JSON. Creating fallback lesson from response content..."));
                            // Use empty string as json_str - create_fallback_from_response will handle it
                            String::new()
                        }
                    } else {
                        // Use empty string as json_str - create_fallback_from_response will handle it
                        eprintln!("{}", Colors::label_warn("WARN"));
                        eprintln!("{}", Colors::warning("Could not extract JSON. Creating fallback lesson from response content..."));
                        String::new()
                    }
                } else {
                    // Use empty string as json_str - create_fallback_from_response will handle it
                    eprintln!("{}", Colors::label_warn("WARN"));
                    eprintln!("{}", Colors::warning("Could not extract JSON. Creating fallback lesson from response content..."));
                    String::new()
                }
            }
        };
        
        // If json_str is empty, it means JSON extraction completely failed - create fallback immediately
        let mut content = if json_str.is_empty() {
            eprintln!("{}", Colors::label_warn("WARN"));
            eprintln!("{}", Colors::warning("Could not extract JSON from response. Creating fallback lesson..."));
            Self::create_fallback_from_response(
                language,
                &topic,
                &response_content,
                "",
            )?
        } else {
            let content_result = GeneratedContent::from_json(&json_str);
            match content_result {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", Colors::label_warn("WARN"));
                    eprintln!("{}", Colors::warning("Failed to parse generated content. Creating fallback lesson..."));
                    
                    // Show helpful diagnostic info
                    let error_msg = e.to_string();
                    if error_msg.contains("missing field") {
                        eprintln!("{}", Colors::muted("Reason: Missing required field in JSON"));
                    } else if error_msg.contains("expected") && error_msg.contains("found") {
                        eprintln!("{}", Colors::muted(&format!("Reason: Type mismatch - {}", error_msg)));
                    } else if error_msg.contains("EOF") || error_msg.contains("unexpected end") {
                        eprintln!("{}", Colors::muted("Reason: JSON was truncated (incomplete response)"));
                    } else if error_msg.contains("trailing") {
                        eprintln!("{}", Colors::muted("Reason: Invalid JSON syntax"));
                    } else {
                        eprintln!("{}", Colors::muted(&format!("Reason: {}", error_msg)));
                    }
                    
                    // Try to extract any useful information from the response before creating fallback
                    Self::create_fallback_from_response(
                        language,
                        &topic,
                        &response_content,
                        &json_str,
                    )?
                }
            }
        };

        // Ensure we have at least 2 code examples
        if content.code_examples.len() < 2 {
            eprintln!("{}", Colors::label_warn("WARN"));
            eprintln!("{}", Colors::warning(&format!("Only {} code example(s) found. Ensuring at least 2 examples.", content.code_examples.len())));
            
            // Get topic-specific examples if we don't have enough
            let (_, topic_examples) = Self::generate_topic_specific_content(language, topic);
            if !topic_examples.is_empty() && content.code_examples.is_empty() {
                content.code_examples = topic_examples;
            }
            
            // If still less than 2, add fallback examples
            while content.code_examples.len() < 2 {
                let example_num = content.code_examples.len() + 1;
                content.code_examples.push(crate::ollama::formatter::CodeExample {
                    code: format!("// Example {} for {} in {}\n// Add your code here", example_num, topic, language.display_name()),
                    explanation: format!("Example {} demonstrating {} in {}.", example_num, topic, language.display_name()),
                });
            }
        }
        
        // Ensure exercises exist and have test cases
        if content.exercises.is_empty() {
            eprintln!("{}", Colors::label_warn("WARN"));
            eprintln!("{}", Colors::warning("No exercises generated. Adding fallback exercise."));
            let fallback_exercise = Self::create_fallback_exercise_with_tests(language, topic);
            content.exercises.push(fallback_exercise);
        } else {
            // Ensure all exercises have test cases
            for exercise in &mut content.exercises {
                if exercise.test_cases.is_empty() {
                    eprintln!("{}", Colors::label_warn("WARN"));
                    eprintln!("{}", Colors::warning(&format!("Exercise '{}' has no test cases. Adding test cases.", exercise.title)));
                    let example_output = exercise.example_output.as_ref().unwrap_or(&"".to_string()).clone();
                    exercise.test_cases = Self::generate_test_cases_for_exercise(language, &exercise.description, &example_output);
                }
            }
        }

        Ok(content)
    }

    fn extract_json(text: &str) -> Result<String> {
        // Try to find JSON block in markdown code fences (```json ... ```)
        if let Some(start) = text.find("```json") {
            let json_start = text[start + 7..].find('\n').unwrap_or(0) + start + 7;
            // Look for closing ```
            if let Some(end_marker) = text[json_start..].find("```") {
                let json = text[json_start..json_start + end_marker].trim();
                if let Ok(_) = serde_json::from_str::<serde_json::Value>(json) {
                    return Ok(json.to_string());
                }
                if let Some(json_obj) = Self::try_extract_incomplete_json(json) {
                    return Ok(json_obj);
                }
            } else {
                let json = text[json_start..].trim();
                if let Some(json_obj) = Self::try_extract_incomplete_json(json) {
                    return Ok(json_obj);
                }
            }
        }

        // Try to find JSON block without language specifier (``` ... ```)
        if let Some(start) = text.find("```") {
            let json_start = text[start + 3..].find('\n').unwrap_or(0) + start + 3;
            if let Some(end_marker) = text[json_start..].find("```") {
                let json = text[json_start..json_start + end_marker].trim();
                if let Ok(_) = serde_json::from_str::<serde_json::Value>(json) {
                    return Ok(json.to_string());
                }
                if let Some(json_obj) = Self::try_extract_incomplete_json(json) {
                    return Ok(json_obj);
                }
            } else {
                // No closing ```, try to extract anyway
                let json = text[json_start..].trim();
                if let Some(json_obj) = Self::try_extract_incomplete_json(json) {
                    return Ok(json_obj);
                }
            }
        }

        // Try to find JSON object directly (even if incomplete)
        if let Some(start) = text.find('{') {
            let mut brace_count = 0;
            let mut in_string = false;
            let mut escape_next = false;
            let mut end_pos = None;

            for (i, ch) in text[start..].char_indices() {
                if escape_next {
                    escape_next = false;
                    continue;
                }

                match ch {
                    '\\' => escape_next = true,
                    '"' => in_string = !in_string,
                    '{' if !in_string => brace_count += 1,
                    '}' if !in_string => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            end_pos = Some(start + i + 1);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let json = if let Some(end) = end_pos {
                text[start..end].trim()
            } else {
                // Incomplete JSON - try to extract what we can
                text[start..].trim()
            };

            // Try to parse
            if serde_json::from_str::<serde_json::Value>(json).is_ok() {
                return Ok(json.to_string());
            }
            
            // Try to fix incomplete JSON
            if let Some(json_obj) = Self::try_extract_incomplete_json(json) {
                return Ok(json_obj);
            }
        }

        // If we can't extract JSON, return a helpful error with the text
        Err(anyhow::anyhow!(
            "Could not extract valid JSON from response. Response text (first 1000 chars):\n{}",
            text.chars().take(1000).collect::<String>()
        ))
    }

    fn try_extract_incomplete_json(text: &str) -> Option<String> {
        // Try to find the JSON object and close it if needed
        if let Some(start) = text.find('{') {
            let mut brace_count = 0;
            let mut bracket_count = 0;
            let mut in_string = false;
            let mut escape_next = false;
            let mut last_valid_pos = start;
            let mut last_char_was_colon = false;
            let mut needs_string_close = false;

            for (i, ch) in text[start..].char_indices() {
                if escape_next {
                    escape_next = false;
                    continue;
                }

                match ch {
                    '\\' => escape_next = true,
                    '"' => {
                        in_string = !in_string;
                        if !in_string {
                            needs_string_close = false;
                        }
                        last_valid_pos = start + i + 1;
                    }
                    '{' if !in_string => {
                        brace_count += 1;
                        last_valid_pos = start + i;
                    }
                    '}' if !in_string => {
                        brace_count -= 1;
                        last_valid_pos = start + i + 1;
                        if brace_count == 0 {
                            // Found complete JSON
                            let json = text[start..start + i + 1].trim();
                            if serde_json::from_str::<serde_json::Value>(json).is_ok() {
                                return Some(json.to_string());
                            }
                        }
                    }
                    '[' if !in_string => {
                        bracket_count += 1;
                        last_valid_pos = start + i;
                    }
                    ']' if !in_string => {
                        bracket_count -= 1;
                        last_valid_pos = start + i + 1;
                    }
                    ':' if !in_string => {
                        last_char_was_colon = true;
                    }
                    _ if !in_string && (ch == ' ' || ch == '\n' || ch == '\t') => {
                        if last_char_was_colon {
                            // We're after a colon, might need to add a value
                            last_char_was_colon = false;
                        }
                    }
                    _ => {
                        if in_string {
                            last_valid_pos = start + i + 1;
                        }
                        last_char_was_colon = false;
                    }
                }
            }

            // If we ended in a string, mark it as needing closure
            if in_string {
                needs_string_close = true;
            }

            // If we have an incomplete JSON, try to close it
            if brace_count > 0 || bracket_count > 0 || needs_string_close {
                let mut json = text[start..=last_valid_pos.min(text.len().saturating_sub(1))].to_string();
                
                // Close incomplete string if needed
                if needs_string_close {
                    // Find the last unclosed quote and close the string properly
                    // Remove any incomplete word/characters after the last quote
                    let mut chars: Vec<char> = json.chars().collect();
                    let mut last_quote_idx = None;
                    for (i, &ch) in chars.iter().enumerate().rev() {
                        if ch == '"' && (i == 0 || chars[i-1] != '\\') {
                            last_quote_idx = Some(i);
                            break;
                        }
                    }
                    
                    if let Some(quote_idx) = last_quote_idx {
                        // Truncate to the quote position and ensure it's closed
                        chars.truncate(quote_idx + 1);
                        json = chars.into_iter().collect();
                    } else {
                        // No quote found, this is malformed - try to salvage
                        // Find the last complete field and close there
                        if let Some(last_comma) = json.rfind(',') {
                            json.truncate(last_comma);
                            json.push('}');
                        }
                    }
                }
                
                // Close incomplete arrays
                for _ in 0..bracket_count {
                    json.push(']');
                }
                
                // Close incomplete objects and add missing required fields
                if brace_count > 0 {
                    // Before closing, check if we need to add missing fields
                    let needs_exercises = !json.contains("\"exercises\"");
                    let needs_common_patterns = !json.contains("\"common_patterns\"");
                    let needs_syntax_guide = !json.contains("\"syntax_guide\"");
                    
                    // Remove trailing comma if present
                    json = json.trim_end_matches(',').to_string();
                    
                    // Add missing fields before closing
                    if needs_exercises || needs_common_patterns || needs_syntax_guide {
                        if !json.ends_with('{') && !json.ends_with('[') {
                            json.push(',');
                        }
                        if needs_syntax_guide && !json.contains("\"syntax_guide\"") {
                            json.push_str(r#" "syntax_guide": ""#);
                        }
                        if needs_common_patterns && !json.contains("\"common_patterns\"") {
                            json.push_str(r#", "common_patterns": []"#);
                        }
                        if needs_exercises && !json.contains("\"exercises\"") {
                            json.push_str(r#", "exercises": []"#);
                        }
                    }
                    
                    // Close all open braces
                    for _ in 0..brace_count {
                        json.push('}');
                    }
                }
                
                // Try to parse the fixed JSON
                if serde_json::from_str::<serde_json::Value>(&json).is_ok() {
                    return Some(json);
                }
                
                // If still invalid, try a more aggressive fix - extract just the valid parts
                if let Some(concept_start) = json.find(r#""concept": "#) {
                    // Try to build a minimal valid JSON with what we have
                    let mut minimal_json = String::from("{");
                    
                    // Extract concept if present
                    if let Some(concept_end) = json[concept_start..].find('"').map(|i| concept_start + i + 1) {
                        if let Some(concept_value_end) = json[concept_end..].find('"').map(|i| concept_end + i + 1) {
                            minimal_json.push_str(&json[concept_start..concept_value_end]);
                            minimal_json.push_str(r#", "step_by_step": [], "code_examples": [], "syntax_guide": "", "common_patterns": [], "exercises": []}"#);
                            if serde_json::from_str::<serde_json::Value>(&minimal_json).is_ok() {
                                return Some(minimal_json);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn create_fallback_from_response(
        language: Language,
        topic: &str,
        response_content: &str,
        _json_str: &str,
    ) -> Result<GeneratedContent> {
        // Try to extract any useful information from the response
        let mut concept = format!("An introduction to {} in {}.", topic, language.display_name());
        let mut code_examples = Vec::new();
        let mut step_by_step = Vec::new();
        
        // Try to find concept in the response
        if let Some(concept_start) = response_content.to_lowercase().find("concept") {
            let end_pos = (concept_start + 300).min(response_content.len());
            let concept_section = &response_content[concept_start..end_pos];
            if let Some(colon) = concept_section.find(':') {
                let extracted = concept_section[colon + 1..].trim();
                // Extract up to first newline or reasonable length
                let concept_text = extracted.lines().next().unwrap_or(extracted);
                if concept_text.len() > 20 && concept_text.len() < 500 {
                    concept = concept_text.to_string();
                }
            }
        }
        
        // Try to extract step-by-step instructions
        if let Some(step_start) = response_content.to_lowercase().find("step") {
            let end_pos = (step_start + 500).min(response_content.len());
            let step_section = &response_content[step_start..end_pos];
            // Look for numbered steps
            for i in 1..=6 {
                let step_pattern = format!("step {}:", i);
                if let Some(pos) = step_section.to_lowercase().find(&step_pattern) {
                    let step_text = &step_section[pos + step_pattern.len()..];
                    if let Some(line_end) = step_text.find('\n') {
                        let step = step_text[..line_end].trim();
                        if !step.is_empty() && step.len() < 200 {
                            step_by_step.push(format!("Step {}: {}", i, step));
                        }
                    }
                }
            }
        }
        
        // Try to extract code examples (skip JSON blocks)
        let mut code_start = 0;
        while let Some(code_pos) = response_content[code_start..].find("```") {
            let actual_pos = code_start + code_pos;
            let end_pos = (actual_pos + 1000).min(response_content.len());
            let code_block = &response_content[actual_pos..end_pos];
            if let Some(code_end) = code_block[3..].find("```") {
                let code = code_block[3..code_end + 3].trim();
                // Skip if it's a JSON block (we already tried that)
                if !code.starts_with('{') && !code.is_empty() && code.len() < 500 {
                    code_examples.push(crate::ollama::formatter::CodeExample {
                        code: code.to_string(),
                        explanation: format!("Example code demonstrating {} in {}.", topic, language.display_name()),
                    });
                    if code_examples.len() >= 2 {
                        break; // Limit to 2 examples
                    }
                }
            }
            code_start = actual_pos + 3;
        }
        
        // If no step-by-step found, create default ones
        if step_by_step.is_empty() {
            step_by_step = vec![
                format!("Step 1: Understand the concept of {}.", topic),
                format!("Step 2: Review examples and syntax."),
                format!("Step 3: Practice with exercises."),
            ];
        }
        
        // Generate better syntax guide and code examples based on topic (call once)
        let (mut syntax_guide_final, topic_code_examples) = Self::generate_topic_specific_content(language, topic);
        
        // Use topic-specific code examples if we don't have any from extraction
        if code_examples.is_empty() && !topic_code_examples.is_empty() {
            code_examples = topic_code_examples;
        }
        
        // Ensure we have at least 2 code examples
        if code_examples.len() < 2 {
            // If we have 1, add a second one based on the first
            if let Some(first_example) = code_examples.first() {
                let second_code = format!("// Variation of the example above\n{}", first_example.code);
                code_examples.push(crate::ollama::formatter::CodeExample {
                    code: second_code,
                    explanation: format!("Another example demonstrating {} in {}.", topic, language.display_name()),
                });
            } else {
                // If we have none, create 2 basic examples
                code_examples.push(crate::ollama::formatter::CodeExample {
                    code: format!("// Basic example for {} in {}", topic, language.display_name()),
                    explanation: format!("Example code demonstrating {} in {}.", topic, language.display_name()),
                });
                code_examples.push(crate::ollama::formatter::CodeExample {
                    code: format!("// Another example for {} in {}", topic, language.display_name()),
                    explanation: format!("Another example showing {} in {}.", topic, language.display_name()),
                });
            }
        }
        
        // Generate syntax guide if we don't have one from topic-specific content
        if syntax_guide_final.is_empty() {
            if !code_examples.is_empty() {
                syntax_guide_final = format!("Basic syntax for {} in {}. Refer to the code examples above for specific syntax patterns.", topic, language.display_name());
            } else {
                syntax_guide_final = format!("Basic syntax for {} in {}.", topic, language.display_name());
            }
        }
        let syntax_guide = syntax_guide_final;
        
        // Create fallback exercise with better description based on topic
        let (description, hints, example_output) = match topic.to_lowercase().as_str() {
            t if t.contains("variable") || t.contains("mutability") => {
                match language {
                    crate::config::Language::Rust => (
                        format!("Declare a variable in Rust. Use `let` to create an immutable variable with a value, then print it using `println!()`. For example, declare a variable `name` with your name and print it."),
                        vec![
                            "Use `let variable_name = value;` to declare a variable".to_string(),
                            "Use `println!(\"text {{}}\", variable_name);` to print the variable".to_string(),
                            "Remember: variables without `mut` cannot be changed".to_string(),
                        ],
                        "Your name".to_string(),
                    ),
                    crate::config::Language::JavaScript => (
                        format!("Declare a variable in JavaScript using `let`, `const`, or `var`. Assign it a value and print it using `console.log()`."),
                        vec![
                            "Use `let variableName = value;` to declare a variable".to_string(),
                            "Use `console.log(variableName);` to print it".to_string(),
                        ],
                        "The value of your variable".to_string(),
                    ),
                    crate::config::Language::Cpp => (
                        format!("Declare a variable in C++. Use the appropriate type (int, string, etc.), assign it a value, and print it using `cout`."),
                        vec![
                            "Use `type variable_name = value;` to declare a variable".to_string(),
                            "Use `cout << variable_name << endl;` to print it".to_string(),
                        ],
                        "The value of your variable".to_string(),
                    ),
                }
            }
            _ => {
                let has_examples = !code_examples.is_empty();
                (
                    if has_examples {
                        format!("Write a simple program in {} that demonstrates {}. Use the examples above as a reference.", language.display_name(), topic)
                    } else {
                        format!("Write a simple program in {} that demonstrates {}.", language.display_name(), topic)
                    },
                    {
                        let mut hints = vec![
                            format!("Start with a basic {} program", language.display_name()),
                            "Make sure your code compiles and runs".to_string(),
                            format!("Focus on demonstrating {}", topic),
                        ];
                        if has_examples {
                            hints.insert(0, "Review the code examples above".to_string());
                        }
                        hints
                    },
                    format!("Output demonstrating {}", topic),
                )
            }
        };
        
        let test_cases = Self::generate_test_cases_for_exercise(language, &description, &example_output);
        let fallback_exercise = crate::ollama::formatter::Exercise {
            title: format!("Practice: {}", topic),
            description,
            hints,
            example_input: Some("".to_string()),
            example_output: Some(example_output),
            test_cases,
        };
        
        Ok(GeneratedContent {
            concept,
            step_by_step,
            code_examples,
            syntax_guide,
            common_patterns: vec![],
            exercises: vec![fallback_exercise],
        })
    }
    
    fn generate_topic_specific_content(language: crate::config::Language, topic: &str) -> (String, Vec<crate::ollama::formatter::CodeExample>) {
        let topic_lower = topic.to_lowercase();
        match language {
            crate::config::Language::Rust => {
                if topic_lower.contains("random") {
                    (
                        "To generate random numbers in Rust, use the `rand` crate. Use `use rand::Rng;` and `let mut rng = rand::thread_rng();` to create a random number generator. Generate random numbers with `rng.gen_range(1..=100)` for a range, or `rng.gen::<i32>()` for a random integer.".to_string(),
                        vec![
                            crate::ollama::formatter::CodeExample {
                                code: "use rand::Rng;\n\nfn main() {\n    let mut rng = rand::thread_rng();\n    let random_num = rng.gen_range(1..=100);\n    println!(\"Random number: {}\", random_num);\n}".to_string(),
                                explanation: "This example shows how to generate a random number between 1 and 100 using the rand crate. The rand dependency will be automatically added to Cargo.toml when you run your code.".to_string(),
                            },
                            crate::ollama::formatter::CodeExample {
                                code: "use rand::Rng;\n\nfn main() {\n    let mut rng = rand::thread_rng();\n    let random_float = rng.gen::<f64>();\n    println!(\"Random float: {}\", random_float);\n}".to_string(),
                                explanation: "This example shows how to generate a random float using gen::<f64>(). This generates a random floating-point number between 0.0 and 1.0.".to_string(),
                            },
                        ],
                    )
                } else if topic_lower.contains("variable") || topic_lower.contains("mutability") {
                    (
                        "In Rust, declare variables with `let`. By default, variables are immutable. Use `let mut` to make them mutable. For example: `let x = 5;` creates an immutable variable, while `let mut y = 5;` creates a mutable one. Attempting to modify an immutable variable will cause a compile error.".to_string(),
                        vec![
                            crate::ollama::formatter::CodeExample {
                                code: "fn main() {\n    let name = \"Alice\";\n    println!(\"Hello, {}!\", name);\n}".to_string(),
                                explanation: "This declares an immutable variable `name` and prints it. The variable cannot be changed after declaration.".to_string(),
                            },
                            crate::ollama::formatter::CodeExample {
                                code: "fn main() {\n    let mut count = 0;\n    count += 1;\n    println!(\"Count: {}\", count);\n}".to_string(),
                                explanation: "This example shows a mutable variable using `let mut`. The variable can be modified after declaration.".to_string(),
                            },
                        ],
                    )
                } else if topic_lower.contains("control flow") || topic_lower.contains("controlflow") || topic_lower.contains("condition") || topic_lower.contains("if") || topic_lower.contains("else") {
                    (
                        "Control flow in Rust uses `if`, `else if`, and `else` statements. The condition must be a boolean expression. You can also use `match` for pattern matching. For example: `if x > 5 { println!(\"Greater\"); } else { println!(\"Less or equal\"); }`".to_string(),
                        vec![
                            crate::ollama::formatter::CodeExample {
                                code: "fn main() {\n    let number = 7;\n    if number > 5 {\n        println!(\"The number is greater than 5\");\n    } else {\n        println!(\"The number is 5 or less\");\n    }\n}".to_string(),
                                explanation: "This example demonstrates a basic if-else statement that checks if a number is greater than 5.".to_string(),
                            },
                            crate::ollama::formatter::CodeExample {
                                code: "fn main() {\n    let score = 85;\n    if score >= 90 {\n        println!(\"Grade: A\");\n    } else if score >= 80 {\n        println!(\"Grade: B\");\n    } else {\n        println!(\"Grade: C\");\n    }\n}".to_string(),
                                explanation: "This example shows an if-else-if chain with multiple conditions to determine a grade based on score.".to_string(),
                            },
                        ],
                    )
                } else {
                    (String::new(), vec![])
                }
            }
            crate::config::Language::JavaScript => {
                if topic_lower.contains("random") {
                    (
                        "In JavaScript, use `Math.random()` to generate a random number between 0 and 1. Multiply by a range and use `Math.floor()` to get integers. For example: `Math.floor(Math.random() * 100) + 1` generates a number between 1 and 100.".to_string(),
                        vec![
                            crate::ollama::formatter::CodeExample {
                                code: "const randomNum = Math.floor(Math.random() * 100) + 1;\nconsole.log(`Random number: ${randomNum}`);".to_string(),
                                explanation: "This generates a random integer between 1 and 100 using Math.random().".to_string(),
                            },
                            crate::ollama::formatter::CodeExample {
                                code: "function getRandomInRange(min, max) {\n    return Math.floor(Math.random() * (max - min + 1)) + min;\n}\nconst num = getRandomInRange(10, 20);\nconsole.log(`Random number between 10 and 20: ${num}`);".to_string(),
                                explanation: "This example shows a reusable function to generate random numbers within a custom range.".to_string(),
                            },
                        ],
                    )
                } else if topic_lower.contains("control flow") || topic_lower.contains("controlflow") || topic_lower.contains("condition") || topic_lower.contains("if") || topic_lower.contains("else") {
                    (
                        "Control flow in JavaScript uses `if`, `else if`, and `else` statements. Conditions can be any expression that evaluates to a truthy or falsy value. For example: `if (x > 5) { console.log('Greater'); } else { console.log('Less or equal'); }`".to_string(),
                        vec![
                            crate::ollama::formatter::CodeExample {
                                code: "const number = 7;\nif (number > 5) {\n    console.log('The number is greater than 5');\n} else {\n    console.log('The number is 5 or less');\n}".to_string(),
                                explanation: "This example demonstrates a basic if-else statement that checks if a number is greater than 5.".to_string(),
                            },
                            crate::ollama::formatter::CodeExample {
                                code: "const age = 18;\nif (age >= 18) {\n    console.log('You are an adult');\n} else if (age >= 13) {\n    console.log('You are a teenager');\n} else {\n    console.log('You are a child');\n}".to_string(),
                                explanation: "This example shows an if-else-if chain with multiple conditions to categorize age groups.".to_string(),
                            },
                        ],
                    )
                } else {
                    (String::new(), vec![])
                }
            }
            crate::config::Language::Cpp => {
                if topic_lower.contains("random") {
                    (
                        "In C++, include `<random>` and `<ctime>`. Use `std::mt19937` for the random number generator, seed it with `std::random_device{}()`, and use `std::uniform_int_distribution<>` to generate numbers in a range. For example: `std::uniform_int_distribution<> dis(1, 100);` then `dis(gen)` to get a random number.".to_string(),
                        vec![
                            crate::ollama::formatter::CodeExample {
                                code: "#include <iostream>\n#include <random>\n\nint main() {\n    std::random_device rd;\n    std::mt19937 gen(rd());\n    std::uniform_int_distribution<> dis(1, 100);\n    int random_num = dis(gen);\n    std::cout << \"Random number: \" << random_num << std::endl;\n    return 0;\n}".to_string(),
                                explanation: "This example shows how to generate a random number between 1 and 100 using C++'s random library.".to_string(),
                            },
                            crate::ollama::formatter::CodeExample {
                                code: "#include <iostream>\n#include <random>\n\nint main() {\n    std::random_device rd;\n    std::mt19937 gen(rd());\n    std::uniform_real_distribution<double> dis(0.0, 1.0);\n    double random_float = dis(gen);\n    std::cout << \"Random float: \" << random_float << std::endl;\n    return 0;\n}".to_string(),
                                explanation: "This example shows how to generate a random floating-point number between 0.0 and 1.0 using uniform_real_distribution.".to_string(),
                            },
                        ],
                    )
                } else if topic_lower.contains("control flow") || topic_lower.contains("controlflow") || topic_lower.contains("condition") || topic_lower.contains("if") || topic_lower.contains("else") {
                    (
                        "Control flow in C++ uses `if`, `else if`, and `else` statements. Conditions must evaluate to a boolean value. For example: `if (x > 5) { std::cout << \"Greater\"; } else { std::cout << \"Less or equal\"; }`".to_string(),
                        vec![
                            crate::ollama::formatter::CodeExample {
                                code: "#include <iostream>\n\nint main() {\n    int number = 7;\n    if (number > 5) {\n        std::cout << \"The number is greater than 5\" << std::endl;\n    } else {\n        std::cout << \"The number is 5 or less\" << std::endl;\n    }\n    return 0;\n}".to_string(),
                                explanation: "This example demonstrates a basic if-else statement that checks if a number is greater than 5.".to_string(),
                            },
                            crate::ollama::formatter::CodeExample {
                                code: "#include <iostream>\n\nint main() {\n    int temperature = 25;\n    if (temperature > 30) {\n        std::cout << \"It's hot\" << std::endl;\n    } else if (temperature > 20) {\n        std::cout << \"It's warm\" << std::endl;\n    } else {\n        std::cout << \"It's cool\" << std::endl;\n    }\n    return 0;\n}".to_string(),
                                explanation: "This example shows an if-else-if chain with multiple conditions to categorize temperature ranges.".to_string(),
                            },
                        ],
                    )
                } else {
                    (String::new(), vec![])
                }
            }
        }
    }
    
    fn create_fallback_exercise_with_tests(language: Language, topic: &str) -> crate::ollama::formatter::Exercise {
        let (description, hints, example_output) = match topic.to_lowercase().as_str() {
            t if t.contains("variable") || t.contains("mutability") => {
                match language {
                    crate::config::Language::Rust => (
                        format!("Declare a variable in Rust. Use `let` to create an immutable variable with a value, then print it using `println!()`. For example, declare a variable `name` with your name and print it."),
                        vec![
                            "Use `let variable_name = value;` to declare a variable".to_string(),
                            "Use `println!(\"text {{}}\", variable_name);` to print the variable".to_string(),
                        ],
                        "Your name".to_string(),
                    ),
                    crate::config::Language::JavaScript => (
                        format!("Declare a variable in JavaScript using `let`, `const`, or `var`. Assign it a value and print it using `console.log()`."),
                        vec![
                            "Use `let variableName = value;` to declare a variable".to_string(),
                            "Use `console.log(variableName);` to print it".to_string(),
                        ],
                        "The value of your variable".to_string(),
                    ),
                    crate::config::Language::Cpp => (
                        format!("Declare a variable in C++ with a type and value, then print it using `cout`."),
                        vec![
                            "Use `type variableName = value;` to declare a variable".to_string(),
                            "Use `std::cout << variableName << std::endl;` to print it".to_string(),
                        ],
                        "The value of your variable".to_string(),
                    ),
                }
            }
            t if t.contains("random") => {
                match language {
                    crate::config::Language::Rust => (
                        format!("Generate a random number in Rust using the `rand` crate. Use `rand::Rng` and generate a random number between 1 and 100, then print it."),
                        vec![
                            "Use `use rand::Rng;` to import the Rng trait".to_string(),
                            "Use `let mut rng = rand::thread_rng();` to create a generator".to_string(),
                            "Use `rng.gen_range(1..=100)` to generate a number".to_string(),
                        ],
                        "Random number between 1 and 100: 42".to_string(),
                    ),
                    crate::config::Language::JavaScript => (
                        format!("Generate a random number in JavaScript using `Math.random()`. Generate a number between 1 and 100 and print it."),
                        vec![
                            "Use `Math.random()` to get a number between 0 and 1".to_string(),
                            "Multiply by 100 and use `Math.floor()` to get an integer".to_string(),
                        ],
                        "Random number between 1 and 100: 42".to_string(),
                    ),
                    crate::config::Language::Cpp => (
                        format!("Generate a random number in C++ using `<random>`. Generate a number between 1 and 100 and print it."),
                        vec![
                            "Include `<random>` header".to_string(),
                            "Use `std::mt19937` and `std::uniform_int_distribution`".to_string(),
                        ],
                        "Random number between 1 and 100: 42".to_string(),
                    ),
                }
            }
            _ => {
                (
                    format!("Write code in {} to demonstrate your understanding of {}. Refer to the explanations and examples above.", language.display_name(), topic),
                    vec![
                        "Review the code examples above".to_string(),
                        "Start with a simple implementation".to_string(),
                    ],
                    format!("Output demonstrating {}", topic),
                )
            }
        };
        
        let test_cases = Self::generate_test_cases_for_exercise(language, &description, &example_output);
        
        crate::ollama::formatter::Exercise {
            title: format!("Practice: {}", topic),
            description,
            hints,
            example_input: Some("".to_string()),
            example_output: Some(example_output),
            test_cases,
        }
    }
    
    fn generate_test_cases_for_exercise(
        _language: Language,
        description: &str,
        example_output: &str,
    ) -> Vec<crate::ollama::formatter::TestCase> {
        let desc_lower = description.to_lowercase();
        let has_input = desc_lower.contains("read input") || 
                       desc_lower.contains("read from stdin") ||
                       desc_lower.contains("input from");
        
        if has_input {
            // For exercises with input, create varied test cases
            vec![
                crate::ollama::formatter::TestCase {
                    input: "5".to_string(),
                    output: example_output.replace("42", "5").replace("test", "5"),
                },
                crate::ollama::formatter::TestCase {
                    input: "10".to_string(),
                    output: example_output.replace("42", "10").replace("test", "10"),
                },
                crate::ollama::formatter::TestCase {
                    input: "42".to_string(),
                    output: example_output.to_string(),
                },
            ]
        } else {
            // For exercises without input, use the example output pattern
            // Create 2-3 test cases with the same expected output
            vec![
                crate::ollama::formatter::TestCase {
                    input: "".to_string(),
                    output: example_output.to_string(),
                },
                crate::ollama::formatter::TestCase {
                    input: "".to_string(),
                    output: example_output.to_string(),
                },
                crate::ollama::formatter::TestCase {
                    input: "".to_string(),
                    output: example_output.to_string(),
                },
            ]
        }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}
