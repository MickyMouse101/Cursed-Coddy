use crate::config::{Difficulty, Language, LessonType};
use crate::ollama::formatter::{CodeExample, Exercise, GeneratedContent, TestCase};

#[derive(Clone)]
pub struct HumanLesson {
    pub content: GeneratedContent,
    pub language: Language,
    pub difficulty: Difficulty,
    pub lesson_type: LessonType,
}

pub struct HumanLessons;

impl HumanLessons {
    fn get_all_lessons() -> Vec<HumanLesson> {
        vec![
            HumanLesson {
                content: Self::rust_hello_world(),
                language: Language::Rust,
                difficulty: Difficulty::Beginner,
                lesson_type: LessonType::Short,
            },
            HumanLesson {
                content: Self::rust_leeson_two(),
                language: Language::Rust,
                difficulty: Difficulty::Beginner,
                lesson_type: LessonType::Short,
            },
            HumanLesson {
                content: Self::javascript_hello_world(),
                language: Language::JavaScript,
                difficulty: Difficulty::Beginner,
                lesson_type: LessonType::Short,
            },
            HumanLesson {
                content: Self::cpp_hello_world(),
                language: Language::Cpp,
                difficulty: Difficulty::Beginner,
                lesson_type: LessonType::Short,
            },
        ]
    }

    pub fn get_next_lesson(last_index: Option<usize>, language: Language) -> Option<(HumanLesson, usize)> {
        let all_lessons = Self::get_all_lessons();

        if all_lessons.is_empty() {
            return None;
        }

        // Filter lessons by language
        let language_lessons: Vec<(usize, &HumanLesson)> = all_lessons
            .iter()
            .enumerate()
            .filter(|(_, lesson)| lesson.language == language)
            .collect();

        if language_lessons.is_empty() {
            return None;
        }

        // Find the next lesson index within this language
        let next_index = match last_index {
            Some(idx) => {
                // Find the index in the language-specific list
                if let Some(pos) = language_lessons.iter().position(|(i, _)| *i == idx) {
                    // Move to next lesson in same language
                    if pos + 1 < language_lessons.len() {
                        language_lessons[pos + 1].0
                    } else {
                        // Wrap around to first lesson of this language
                        language_lessons[0].0
                    }
                } else {
                    // Last index doesn't match, start from first of this language
                    language_lessons[0].0
                }
            }
            None => language_lessons[0].0, // Start with first lesson of this language
        };

        Some((all_lessons[next_index].clone(), next_index))
    }

    fn rust_hello_world() -> GeneratedContent {
        GeneratedContent {
            concept: "Welcome to Rust! In this lesson, you'll learn how to write your first Rust program. The 'Hello, World!' program is a traditional first program that prints a message to the console.".to_string(),
            step_by_step: vec![
                "Rust programs start with a main function, which is the entry point of your program.".to_string(),
                "The println! macro is used to print text to the console. The exclamation mark indicates it's a macro, not a regular function.".to_string(),
                "String literals in Rust are enclosed in double quotes. The println! macro will output this text followed by a newline.".to_string(),
            ],
            code_examples: vec![
                CodeExample {
                    code: "fn main() {\n    println!(\"Hello, World!\");\n}".to_string(),
                    explanation: "This is the simplest Rust program. It defines a main function and prints 'Hello, World!' to the console.".to_string(),
                },
                CodeExample {
                    code: "fn main() {\n    let message = \"Hello, Rust!\";\n    println!(\"{}\", message);\n}".to_string(),
                    explanation: "This example shows how to store a string in a variable and then print it using println! with formatting.".to_string(),
                },
            ],
            syntax_guide: "Key Rust syntax:\n- fn main() defines the entry point\n- println!() is a macro for printing\n- Semicolons end statements\n- Curly braces {} define code blocks".to_string(),
            common_patterns: vec![
                "Always start with fn main()".to_string(),
                "Use println! for console output".to_string(),
                "End statements with semicolons".to_string(),
            ],
            exercises: vec![
                Exercise {
                    title: "Print Your Name".to_string(),
                    description: "Write a program that prints your name to the console. Use println! to output the text.".to_string(),
                    hints: vec![
                        "Start with fn main()".to_string(),
                        "Use println!(\"Your Name\") inside main".to_string(),
                        "Don't forget the semicolon at the end".to_string(),
                    ],
                    example_input: None,
                    example_output: Some("Your Name".to_string()),
                    test_cases: vec![
                        TestCase {
                            input: "".to_string(),
                            output: "Your Name".to_string(),
                        },
                    ],
                },
                Exercise {
                    title: "Print Multiple Messages".to_string(),
                    description: "Create a program that prints three different messages, each on a new line.".to_string(),
                    hints: vec![
                        "You can use multiple println! statements".to_string(),
                        "Each println! will print on a new line automatically".to_string(),
                    ],
                    example_input: None,
                    example_output: Some("First message\nSecond message\nThird message".to_string()),
                    test_cases: vec![
                        TestCase {
                            input: "".to_string(),
                            output: "First message\nSecond message\nThird message".to_string(),
                        },
                    ],
                },
            ],
        }
    }

    fn rust_leeson_two() -> GeneratedContent {
        GeneratedContent {
            concept: "Welcome to Rust! TEST TEST console.".to_string(),
            step_by_step: vec![
                "Rust programs start with a main function, which is the entry point of your program.".to_string(),
                "The println! macro is used to print text to the console. The exclamation mark indicates it's a macro, not a regular function.".to_string(),
                "String literals in Rust are enclosed in double quotes. The println! macro will output this text followed by a newline.".to_string(),
            ],
            code_examples: vec![
                CodeExample {
                    code: "fn main() {\n    println!(\"Hello, World!\");\n}".to_string(),
                    explanation: "This is the simplest Rust program. It defines a main function and prints 'Hello, World!' to the console.".to_string(),
                },
                CodeExample {
                    code: "fn main() {\n    let message = \"Hello, Rust!\";\n    println!(\"{}\", message);\n}".to_string(),
                    explanation: "This example shows how to store a string in a variable and then print it using println! with formatting.".to_string(),
                },
            ],
            syntax_guide: "Key Rust syntax:\n- fn main() defines the entry point\n- println!() is a macro for printing\n- Semicolons end statements\n- Curly braces {} define code blocks".to_string(),
            common_patterns: vec![
                "Always start with fn main()".to_string(),
                "Use println! for console output".to_string(),
                "End statements with semicolons".to_string(),
            ],
            exercises: vec![
                Exercise {
                    title: "Print Your Name".to_string(),
                    description: "Write a program that prints your name to the console. Use println! to output the text.".to_string(),
                    hints: vec![
                        "Start with fn main()".to_string(),
                        "Use println!(\"Your Name\") inside main".to_string(),
                        "Don't forget the semicolon at the end".to_string(),
                    ],
                    example_input: None,
                    example_output: Some("Your Name".to_string()),
                    test_cases: vec![
                        TestCase {
                            input: "".to_string(),
                            output: "Your Name".to_string(),
                        },
                    ],
                },
                Exercise {
                    title: "Print Multiple Messages".to_string(),
                    description: "Create a program that prints three different messages, each on a new line.".to_string(),
                    hints: vec![
                        "You can use multiple println! statements".to_string(),
                        "Each println! will print on a new line automatically".to_string(),
                    ],
                    example_input: None,
                    example_output: Some("First message\nSecond message\nThird message".to_string()),
                    test_cases: vec![
                        TestCase {
                            input: "".to_string(),
                            output: "First message\nSecond message\nThird message".to_string(),
                        },
                    ],
                },
            ],
        }
    }

    fn javascript_hello_world() -> GeneratedContent {
        GeneratedContent {
            concept: "Welcome to JavaScript! In this lesson, you'll learn how to write your first JavaScript program. JavaScript is a versatile language that runs in browsers and on servers with Node.js.".to_string(),
            step_by_step: vec![
                "JavaScript programs can be written directly without a main function (though Node.js scripts typically start executing from the top).".to_string(),
                "The console.log() function is used to print text to the console in JavaScript.".to_string(),
                "JavaScript statements end with semicolons, though they are often optional.".to_string(),
            ],
            code_examples: vec![
                CodeExample {
                    code: "console.log('Hello, World!');".to_string(),
                    explanation: "This is the simplest JavaScript program. It uses console.log to print 'Hello, World!' to the console.".to_string(),
                },
                CodeExample {
                    code: "const message = 'Hello, JavaScript!';\nconsole.log(message);".to_string(),
                    explanation: "This example shows how to store a string in a constant variable and then print it using console.log.".to_string(),
                },
            ],
            syntax_guide: "Key JavaScript syntax:\n- console.log() prints to console\n- const declares a constant variable\n- Semicolons are optional but recommended\n- Strings can use single or double quotes".to_string(),
            common_patterns: vec![
                "Use console.log() for output".to_string(),
                "const for constants, let for variables".to_string(),
                "Strings can use 'single' or \"double\" quotes".to_string(),
            ],
            exercises: vec![
                Exercise {
                    title: "Print Your Name".to_string(),
                    description: "Write a program that prints your name to the console using console.log.".to_string(),
                    hints: vec![
                        "Use console.log('Your Name')".to_string(),
                        "Don't forget the semicolon".to_string(),
                    ],
                    example_input: None,
                    example_output: Some("Your Name".to_string()),
                    test_cases: vec![
                        TestCase {
                            input: "".to_string(),
                            output: "Your Name".to_string(),
                        },
                    ],
                },
                Exercise {
                    title: "Print Multiple Messages".to_string(),
                    description: "Create a program that prints three different messages, each on a new line.".to_string(),
                    hints: vec![
                        "You can use multiple console.log() statements".to_string(),
                        "Each console.log() will print on a new line automatically".to_string(),
                    ],
                    example_input: None,
                    example_output: Some("First message\nSecond message\nThird message".to_string()),
                    test_cases: vec![
                        TestCase {
                            input: "".to_string(),
                            output: "First message\nSecond message\nThird message".to_string(),
                        },
                    ],
                },
            ],
        }
    }

    fn cpp_hello_world() -> GeneratedContent {
        GeneratedContent {
            concept: "Welcome to C++! In this lesson, you'll learn how to write your first C++ program. C++ is a powerful systems programming language with a rich standard library.".to_string(),
            step_by_step: vec![
                "C++ programs need to include necessary headers. For input/output, we use <iostream>.".to_string(),
                "The main() function is the entry point of every C++ program. It returns an int (0 for success).".to_string(),
                "std::cout is used to output text to the console. The << operator is used to send data to cout.".to_string(),
                "std::endl or '\\n' is used to add a newline after the output.".to_string(),
            ],
            code_examples: vec![
                CodeExample {
                    code: "#include <iostream>\n\nint main() {\n    std::cout << \"Hello, World!\" << std::endl;\n    return 0;\n}".to_string(),
                    explanation: "This is the simplest C++ program. It includes iostream, defines main, and uses std::cout to print 'Hello, World!'.".to_string(),
                },
                CodeExample {
                    code: "#include <iostream>\n\nint main() {\n    std::string message = \"Hello, C++!\";\n    std::cout << message << std::endl;\n    return 0;\n}".to_string(),
                    explanation: "This example shows how to store a string in a variable and then print it using std::cout.".to_string(),
                },
            ],
            syntax_guide: "Key C++ syntax:\n- #include <iostream> for input/output\n- int main() is the entry point\n- std::cout << for output\n- return 0 indicates success\n- Semicolons end statements".to_string(),
            common_patterns: vec![
                "Always include necessary headers".to_string(),
                "Use std::cout << for output".to_string(),
                "Return 0 from main() on success".to_string(),
            ],
            exercises: vec![
                Exercise {
                    title: "Print Your Name".to_string(),
                    description: "Write a program that prints your name to the console using std::cout.".to_string(),
                    hints: vec![
                        "Start with #include <iostream>".to_string(),
                        "Use int main() and return 0".to_string(),
                        "Use std::cout << \"Your Name\" << std::endl;".to_string(),
                    ],
                    example_input: None,
                    example_output: Some("Your Name".to_string()),
                    test_cases: vec![
                        TestCase {
                            input: "".to_string(),
                            output: "Your Name".to_string(),
                        },
                    ],
                },
                Exercise {
                    title: "Print Multiple Messages".to_string(),
                    description: "Create a program that prints three different messages, each on a new line.".to_string(),
                    hints: vec![
                        "You can use multiple std::cout statements".to_string(),
                        "Use std::endl or '\\n' for newlines".to_string(),
                    ],
                    example_input: None,
                    example_output: Some("First message\nSecond message\nThird message".to_string()),
                    test_cases: vec![
                        TestCase {
                            input: "".to_string(),
                            output: "First message\nSecond message\nThird message".to_string(),
                        },
                    ],
                },
            ],
        }
    }
}
