use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub code: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub title: String,
    pub description: String,
    pub hints: Vec<String>,
    pub example_input: Option<String>,
    pub example_output: Option<String>,
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedContent {
    pub concept: String,
    #[serde(default)]
    pub step_by_step: Vec<String>,
    #[serde(default)]
    pub code_examples: Vec<CodeExample>,
    #[serde(default)]
    pub syntax_guide: String,
    #[serde(default)]
    pub common_patterns: Vec<String>,
    #[serde(default)]
    pub exercises: Vec<Exercise>,
}

impl GeneratedContent {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}
