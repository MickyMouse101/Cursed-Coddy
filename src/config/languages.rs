use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    JavaScript,
    Cpp,
    Rust,
}

impl Language {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::JavaScript => "js",
            Language::Cpp => "cpp",
            Language::Rust => "rs",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::JavaScript => "JavaScript",
            Language::Cpp => "C++",
            Language::Rust => "Rust",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
