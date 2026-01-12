use crate::config::Language;
use crate::execution::{CppRunner, JsRunner, RustRunner};
use anyhow::Result;
use std::path::Path;

pub struct Executor;

#[derive(Debug)]
pub struct ExecutionResult {
    pub output: String,
}

impl Executor {
    pub fn execute(
        language: Language,
        file_path: &Path,
        input: Option<&str>,
    ) -> Result<ExecutionResult> {
        let output_result = match language {
            Language::JavaScript => JsRunner::execute(file_path, input),
            Language::Cpp => CppRunner::execute(file_path, input),
            Language::Rust => RustRunner::execute(file_path, input),
        };
        
        match output_result {
            Ok(output) => Ok(ExecutionResult { output }),
            Err(e) => Err(e),
        }
    }

    pub fn compare_output(actual: &str, expected: &str) -> bool {
        actual.trim() == expected.trim()
    }
}
