use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct FileManager;

impl FileManager {
    pub fn create_exercise_file(
        language: &crate::config::Language,
        exercise_number: usize,
    ) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir().join("cursed-coddy");
        std::fs::create_dir_all(&temp_dir)
            .context("Failed to create temp directory")?;

        let filename = format!("exercise_{}.{}", exercise_number, language.file_extension());
        let file_path = temp_dir.join(&filename);

        // Create file with basic template
        let template = Self::get_template(language);
        std::fs::write(&file_path, template)
            .context("Failed to write exercise file")?;

        Ok(file_path)
    }

    fn get_template(language: &crate::config::Language) -> &'static str {
        match language {
            crate::config::Language::JavaScript => "// Write your solution here\n\n",
            crate::config::Language::Cpp => "#include <iostream>\nusing namespace std;\n\nint main() {\n    // Write your solution here\n    return 0;\n}\n",
            crate::config::Language::Rust => "fn main() {\n    // Write your solution here\n}\n",
        }
    }
}
