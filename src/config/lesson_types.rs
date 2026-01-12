use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LessonType {
    Short,
    Medium,
    Long,
}

impl LessonType {
    pub fn display_name(&self) -> &'static str {
        match self {
            LessonType::Short => "Short",
            LessonType::Medium => "Medium",
            LessonType::Long => "Long",
        }
    }

    pub fn concept_count(&self) -> usize {
        match self {
            LessonType::Short => 1,
            LessonType::Medium => 2,
            LessonType::Long => 3,
        }
    }

    pub fn exercise_count(&self) -> usize {
        match self {
            LessonType::Short => 1,
            LessonType::Medium => 3,
            LessonType::Long => 5,
        }
    }
}

impl std::fmt::Display for LessonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
