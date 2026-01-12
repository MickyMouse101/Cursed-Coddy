use crate::config::{Difficulty, Language, LessonType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curriculum {
    pub language: Language,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub name: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub topics: Vec<String>,
    pub lesson_type: LessonType,
}

impl Curriculum {
    pub fn get_for_language(language: Language) -> Self {
        match language {
            Language::JavaScript => Self::javascript_curriculum(),
            Language::Cpp => Self::cpp_curriculum(),
            Language::Rust => Self::rust_curriculum(),
        }
    }

    fn javascript_curriculum() -> Self {
        Self {
            language: Language::JavaScript,
            stages: vec![
                Stage {
                    name: "Getting Started".to_string(),
                    description: "Learn the basics of JavaScript".to_string(),
                    difficulty: Difficulty::Beginner,
                    topics: vec![
                        "variables".to_string(),
                        "data types".to_string(),
                        "operators".to_string(),
                        "console output".to_string(),
                    ],
                    lesson_type: LessonType::Short,
                },
                Stage {
                    name: "Control Flow".to_string(),
                    description: "Learn conditionals and loops".to_string(),
                    difficulty: Difficulty::Beginner,
                    topics: vec![
                        "if statements".to_string(),
                        "for loops".to_string(),
                        "while loops".to_string(),
                        "switch statements".to_string(),
                    ],
                    lesson_type: LessonType::Medium,
                },
                Stage {
                    name: "Functions".to_string(),
                    description: "Learn to write reusable code".to_string(),
                    difficulty: Difficulty::Beginner,
                    topics: vec![
                        "function basics".to_string(),
                        "parameters and arguments".to_string(),
                        "return values".to_string(),
                        "arrow functions".to_string(),
                    ],
                    lesson_type: LessonType::Medium,
                },
                Stage {
                    name: "Arrays and Objects".to_string(),
                    description: "Work with data structures".to_string(),
                    difficulty: Difficulty::Intermediate,
                    topics: vec![
                        "arrays".to_string(),
                        "array methods".to_string(),
                        "objects".to_string(),
                        "object methods".to_string(),
                    ],
                    lesson_type: LessonType::Medium,
                },
                Stage {
                    name: "Advanced Concepts".to_string(),
                    description: "Master advanced JavaScript".to_string(),
                    difficulty: Difficulty::Intermediate,
                    topics: vec![
                        "closures".to_string(),
                        "promises".to_string(),
                        "async/await".to_string(),
                        "classes".to_string(),
                    ],
                    lesson_type: LessonType::Long,
                },
                Stage {
                    name: "Expert Level".to_string(),
                    description: "Become a JavaScript expert".to_string(),
                    difficulty: Difficulty::Advanced,
                    topics: vec![
                        "design patterns".to_string(),
                        "algorithm optimization".to_string(),
                        "advanced data structures".to_string(),
                        "performance optimization".to_string(),
                    ],
                    lesson_type: LessonType::Long,
                },
            ],
        }
    }

    fn cpp_curriculum() -> Self {
        Self {
            language: Language::Cpp,
            stages: vec![
                Stage {
                    name: "Getting Started".to_string(),
                    description: "Learn the basics of C++".to_string(),
                    difficulty: Difficulty::Beginner,
                    topics: vec![
                        "variables and types".to_string(),
                        "input and output".to_string(),
                        "operators".to_string(),
                        "basic syntax".to_string(),
                    ],
                    lesson_type: LessonType::Short,
                },
                Stage {
                    name: "Control Structures".to_string(),
                    description: "Learn conditionals and loops".to_string(),
                    difficulty: Difficulty::Beginner,
                    topics: vec![
                        "if-else statements".to_string(),
                        "for loops".to_string(),
                        "while loops".to_string(),
                        "switch statements".to_string(),
                    ],
                    lesson_type: LessonType::Medium,
                },
                Stage {
                    name: "Functions".to_string(),
                    description: "Learn to write functions".to_string(),
                    difficulty: Difficulty::Beginner,
                    topics: vec![
                        "function definition".to_string(),
                        "parameters".to_string(),
                        "return types".to_string(),
                        "function overloading".to_string(),
                    ],
                    lesson_type: LessonType::Medium,
                },
                Stage {
                    name: "Arrays and Pointers".to_string(),
                    description: "Work with arrays and memory".to_string(),
                    difficulty: Difficulty::Intermediate,
                    topics: vec![
                        "arrays".to_string(),
                        "pointers".to_string(),
                        "references".to_string(),
                        "dynamic memory".to_string(),
                    ],
                    lesson_type: LessonType::Long,
                },
                Stage {
                    name: "Object-Oriented Programming".to_string(),
                    description: "Learn OOP in C++".to_string(),
                    difficulty: Difficulty::Intermediate,
                    topics: vec![
                        "classes".to_string(),
                        "inheritance".to_string(),
                        "polymorphism".to_string(),
                        "templates".to_string(),
                    ],
                    lesson_type: LessonType::Long,
                },
                Stage {
                    name: "Advanced C++".to_string(),
                    description: "Master advanced C++ features".to_string(),
                    difficulty: Difficulty::Advanced,
                    topics: vec![
                        "STL containers".to_string(),
                        "smart pointers".to_string(),
                        "move semantics".to_string(),
                        "concurrency".to_string(),
                    ],
                    lesson_type: LessonType::Long,
                },
            ],
        }
    }

    fn rust_curriculum() -> Self {
        Self {
            language: Language::Rust,
            stages: vec![
                Stage {
                    name: "Getting Started".to_string(),
                    description: "Learn the basics of Rust".to_string(),
                    difficulty: Difficulty::Beginner,
                    topics: vec![
                        "variables and mutability".to_string(),
                        "data types".to_string(),
                        "ownership basics".to_string(),
                        "functions".to_string(),
                    ],
                    lesson_type: LessonType::Short,
                },
                Stage {
                    name: "Control Flow".to_string(),
                    description: "Learn conditionals and loops".to_string(),
                    difficulty: Difficulty::Beginner,
                    topics: vec![
                        "if expressions".to_string(),
                        "loops".to_string(),
                        "match expressions".to_string(),
                        "pattern matching".to_string(),
                    ],
                    lesson_type: LessonType::Medium,
                },
                Stage {
                    name: "Ownership and Borrowing".to_string(),
                    description: "Master Rust's unique features".to_string(),
                    difficulty: Difficulty::Intermediate,
                    topics: vec![
                        "ownership".to_string(),
                        "borrowing".to_string(),
                        "references".to_string(),
                        "lifetimes basics".to_string(),
                    ],
                    lesson_type: LessonType::Long,
                },
                Stage {
                    name: "Structs and Enums".to_string(),
                    description: "Work with custom types".to_string(),
                    difficulty: Difficulty::Intermediate,
                    topics: vec![
                        "structs".to_string(),
                        "enums".to_string(),
                        "methods".to_string(),
                        "associated functions".to_string(),
                    ],
                    lesson_type: LessonType::Medium,
                },
                Stage {
                    name: "Collections".to_string(),
                    description: "Work with data structures".to_string(),
                    difficulty: Difficulty::Intermediate,
                    topics: vec![
                        "vectors".to_string(),
                        "strings".to_string(),
                        "hash maps".to_string(),
                        "iterators".to_string(),
                    ],
                    lesson_type: LessonType::Medium,
                },
                Stage {
                    name: "Advanced Rust".to_string(),
                    description: "Master advanced Rust".to_string(),
                    difficulty: Difficulty::Advanced,
                    topics: vec![
                        "error handling".to_string(),
                        "generics".to_string(),
                        "traits".to_string(),
                        "concurrency".to_string(),
                    ],
                    lesson_type: LessonType::Long,
                },
            ],
        }
    }

    pub fn get_stage(&self, stage_index: usize) -> Option<&Stage> {
        self.stages.get(stage_index)
    }

    pub fn total_stages(&self) -> usize {
        self.stages.len()
    }
}
