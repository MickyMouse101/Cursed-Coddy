use crate::cli::{banner, colors::{Borders, Colors}};
use crate::config::{Difficulty, Language, LessonType};
use crate::execution::{Executor, FileManager};
use crate::ollama::{formatter::GeneratedContent, Generator};
use crate::progress::Tracker;
use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, Text};
use std::process::Command;
use std::thread;
use std::time::Duration;

// Terminal width for text wrapping (default to 78, leaving margin)
const TERMINAL_WIDTH: usize = 78;

// Helper function to wrap text to terminal width
fn wrap_text(text: &str, width: usize, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    let mut result = String::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
        let word_chars = word.chars().count();
        let current_chars = current_line.chars().count();
        
        if current_line.is_empty() {
            current_line = format!("{}{}", indent_str, word);
        } else if current_chars + word_chars + 1 <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            result.push_str(&current_line);
            result.push('\n');
            current_line = format!("{}{}", indent_str, word);
        }
    }
    
    if !current_line.is_empty() {
        result.push_str(&current_line);
    }
    
    result
}

// Helper to print wrapped text with proper formatting
fn print_wrapped(text: &str, width: usize, indent: usize) {
    let wrapped = wrap_text(text, width, indent);
    for line in wrapped.lines() {
        println!("{}", line);
    }
}

// Helper to print section with proper spacing
fn print_section_header(title: &str, color_fn: fn(&str) -> colored::ColoredString) {
    println!();
    // Use consistent width for all section headers
    let width = TERMINAL_WIDTH.min(78);
    let title_chars = title.chars().count();
    let title_padding = width.saturating_sub(title_chars + 2);
    let left_pad = title_padding / 2;
    let right_pad = title_padding - left_pad;
    let border = format!("╔{}╗", "═".repeat(width.saturating_sub(2)));
    let title_line = format!("║{}{}{}║", " ".repeat(left_pad), title, " ".repeat(right_pad));
    println!("{}", color_fn(&border).bold());
    println!("{}", color_fn(&title_line).bold());
    println!("{}", color_fn(&format!("╚{}╝", "═".repeat(width.saturating_sub(2)))).bold());
    println!();
}

pub struct LessonManager {
    generator: Generator,
    tracker: Tracker,
}

impl LessonManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            generator: Generator::new(),
            tracker: Tracker::new()?,
        })
    }

    fn clear_screen() {
        let _ = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", "cls"]).status()
        } else {
            Command::new("clear").status()
        };
        // Redisplay banner after clearing
        banner::display_banner();
    }

    pub fn start_lesson(
        &self,
        language: Language,
        difficulty: Difficulty,
        lesson_type: LessonType,
        topic: String,
    ) -> Result<()> {
        println!("\n{}", Colors::primary(&"=".repeat(60)));
        println!("{}", Colors::primary(&format!("Starting {} Lesson", lesson_type.display_name())).bold());
        println!("{}", Colors::primary(&"=".repeat(60)));
        println!("Language: {}", Colors::warning(language.display_name()));
        println!("Difficulty: {}", Colors::warning(difficulty.display_name()));
        println!("Topic: {}\n", Colors::warning(&topic));

        println!("{}", Colors::info("Generating lesson content..."));
        let content = self
            .generator
            .generate(language, difficulty, lesson_type, &topic)?;

        self.start_lesson_with_content(language, difficulty, lesson_type, topic, content)
    }

    pub fn start_lesson_with_content(
        &self,
        language: Language,
        difficulty: Difficulty,
        lesson_type: LessonType,
        topic: String,
        mut content: GeneratedContent,
    ) -> Result<()> {
        // Clear screen for clean view
        Self::clear_screen();

        // Display concept introduction
        print_section_header("CONCEPT INTRODUCTION", |s| Colors::success(s));
        print_wrapped(&content.concept, TERMINAL_WIDTH, 0);
        println!();

        // Display step-by-step explanation
        if !content.step_by_step.is_empty() {
            print_section_header("STEP-BY-STEP EXPLANATION", |s| Colors::primary(s));
            for (idx, step) in content.step_by_step.iter().enumerate() {
                print!("  {}. ", Colors::primary(&(idx + 1).to_string()).bold());
                print_wrapped(step, TERMINAL_WIDTH - 4, 4);
                println!();
            }
        }

        // Display code examples
        if !content.code_examples.is_empty() {
            print_section_header("CODE EXAMPLES", |s| Colors::warning(s));
            for (idx, example) in content.code_examples.iter().enumerate() {
                println!("  {}", Colors::warning(&format!("Example {}:", idx + 1)).bold());
                // Calculate box width: find longest line, add padding, but cap at terminal width
                let max_line_len = example.code.lines()
                    .map(|l| l.chars().count())
                    .max()
                    .unwrap_or(0);
                // Cap at reasonable width (60 chars max for code), leaving margin for box borders and indentation
                let effective_max = max_line_len.min(60).min(TERMINAL_WIDTH - 8);
                let box_width = effective_max + 4; // Add padding for box borders (2 chars on each side)
                println!("  {}", Colors::text(&Borders::top(box_width)));
                for line in example.code.lines() {
                    // Truncate line if it's too long
                    let display_line = if line.chars().count() > effective_max {
                        &line[..effective_max.min(line.len())]
                    } else {
                        line
                    };
                    println!("  {}", Borders::box_line_left(display_line, box_width));
                }
                println!("  {}", Colors::text(&Borders::bottom(box_width)));
                print!("  {} ", Colors::label_info("TIP"));
                print_wrapped(&example.explanation, TERMINAL_WIDTH - 4, 4);
                println!();
            }
        }

        // Display syntax guide
        if !content.syntax_guide.is_empty() {
            print_section_header("SYNTAX GUIDE", |s| Colors::accent(s));
            print_wrapped(&content.syntax_guide, TERMINAL_WIDTH, 0);
            println!();
        }

        // Display common patterns
        if !content.common_patterns.is_empty() {
            print_section_header("COMMON PATTERNS", |s| Colors::info(s));
            for (idx, pattern) in content.common_patterns.iter().enumerate() {
                print!("  {}. ", Colors::info(&(idx + 1).to_string()).bold());
                print_wrapped(pattern, TERMINAL_WIDTH - 4, 4);
                println!();
            }
        }

        // Check if exercises were generated
        if content.exercises.is_empty() {
            println!();
            println!("{}", Colors::label_warn("WARN"));
            println!("{}", Colors::warning("No exercises were generated. Creating a simple practice exercise..."));
            // Create a simple fallback exercise
            let fallback_exercise = crate::ollama::formatter::Exercise {
                title: format!("Practice: {}", topic),
                description: format!("Apply what you learned about {} by writing a simple program that demonstrates the concept.", topic),
                hints: vec![
                    "Review the code examples above".to_string(),
                    "Start with a simple implementation".to_string(),
                ],
                example_input: Some("".to_string()),
                example_output: Some("(Your code should demonstrate the concept)".to_string()),
                test_cases: vec![],
            };
            content.exercises.push(fallback_exercise);
        }
        
        // Auto-fill missing example_input/example_output from test cases if needed
        for exercise in &mut content.exercises {
            // If example_output is missing but we have test cases, use first test case output
            if exercise.example_output.is_none() || exercise.example_output.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                if let Some(first_test) = exercise.test_cases.first() {
                    exercise.example_output = Some(first_test.output.clone());
                }
            }
            
            // If example_input is missing but we have test cases, use first test case input
            if exercise.example_input.is_none() || exercise.example_input.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                if let Some(first_test) = exercise.test_cases.first() {
                    if !first_test.input.trim().is_empty() {
                        exercise.example_input = Some(first_test.input.clone());
                    } else {
                        exercise.example_input = Some("".to_string());
                    }
                } else {
                    exercise.example_input = Some("".to_string());
                }
            }
        }

        // Start tracking
        self.tracker.start_lesson(
            language,
            difficulty,
            lesson_type,
            topic.clone(),
            content.exercises.len(),
        )?;

        // Process exercises - don't clear screen before first exercise, show it right after lesson content
        for (idx, exercise) in content.exercises.iter().enumerate() {
            let clear_before = idx > 0; // Only clear screen for exercises after the first one
            self.handle_exercise(language, idx + 1, exercise, &content, clear_before)?;
            self.tracker.complete_exercise()?;
        }

        // Complete lesson
        self.tracker.complete_lesson()?;
        println!();
        println!("{}", Colors::label_pass("SUCCESS").bold());
        println!("{}", Colors::success("Lesson completed!").bold());

        Ok(())
    }

    fn handle_exercise(
        &self,
        language: Language,
        exercise_number: usize,
        exercise: &crate::ollama::formatter::Exercise,
        content: &crate::ollama::formatter::GeneratedContent,
        clear_screen: bool,
    ) -> Result<()> {
        // Clear screen before exercise if requested (not for first exercise)
        if clear_screen {
            Self::clear_screen();
            println!("\n{}", Colors::primary(&"=".repeat(60)));
        } else {
            println!();
            println!("{}", Colors::primary(&"=".repeat(60)));
        }
        println!("{}", Colors::primary(&format!("Exercise {}: {}", exercise_number, exercise.title)).bold());
        println!("{}", Colors::primary(&"=".repeat(60)));
        
        // Display quick reference section with key concepts
        print_section_header("QUICK REFERENCE", |s| Colors::accent(s));
        
        // Show the concept first (especially important if user skipped previous exercises)
        if !content.concept.trim().is_empty() {
            println!("{}", Colors::success("Concept:").bold());
            print_wrapped(&content.concept, TERMINAL_WIDTH - 4, 4);
            println!();
            println!("{}", Colors::primary(&Borders::separator(TERMINAL_WIDTH - 4)));
            println!();
        }
        
        // Show relevant code examples FIRST (if available) - they're more useful than syntax text
        if !content.code_examples.is_empty() {
            println!("{}", Colors::warning("Example Code:").bold());
            for (idx, example) in content.code_examples.iter().take(2).enumerate() {
                println!("\n  {}", Colors::primary(&format!("Example {}:", idx + 1)));
                // Calculate box width: find longest line, add padding, but cap at terminal width
                let max_line_len = example.code.lines()
                    .map(|l| l.chars().count())
                    .max()
                    .unwrap_or(0);
                // Cap at reasonable width (60 chars max for code), leaving margin for box borders and indentation
                let effective_max = max_line_len.min(60).min(TERMINAL_WIDTH - 8);
                let box_width = effective_max + 4; // Add padding for box borders (2 chars on each side)
                println!("  {}", Colors::text(&Borders::top(box_width)));
                for line in example.code.lines() {
                    // Truncate line if it's too long
                    let display_line = if line.chars().count() > effective_max {
                        &line[..effective_max.min(line.len())]
                    } else {
                        line
                    };
                    println!("  {}", Borders::box_line_left(display_line, box_width));
                }
                println!("  {}", Colors::text(&Borders::bottom(box_width)));
                // Show brief explanation (first sentence)
                if let Some(first_sentence) = example.explanation.split('.').next() {
                    println!("  {} {}\n", Colors::label_info("TIP"), first_sentence.trim());
                }
            }
        }
        
        println!("{}", Colors::primary(&Borders::separator(TERMINAL_WIDTH)));
        print_section_header("EXERCISE INSTRUCTIONS", |s| Colors::success(s));
        
        print_wrapped(&exercise.description, TERMINAL_WIDTH, 0);
        println!();

        // Check if input is expected (has test cases with input or example_input)
        let expects_input = !exercise.test_cases.is_empty() && 
            exercise.test_cases.iter().any(|tc| !tc.input.trim().is_empty()) ||
            exercise.example_input.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
        
        // Check if output is expected (has test cases with output or example_output)
        let expects_output = !exercise.test_cases.is_empty() && 
            exercise.test_cases.iter().any(|tc| !tc.output.trim().is_empty()) ||
            exercise.example_output.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
        
        // Detect if test cases have different inputs (indicates stdin reading needed)
        let has_different_inputs = if !exercise.test_cases.is_empty() {
            let inputs: std::collections::HashSet<_> = exercise.test_cases.iter()
                .filter(|tc| !tc.input.trim().is_empty())
                .map(|tc| tc.input.trim())
                .collect();
            inputs.len() > 1
        } else {
            false
        };
        
        // Detect if test cases have different outputs but no input (likely an error in generation)
        let has_different_outputs_no_input = if !exercise.test_cases.is_empty() && !expects_input {
            let outputs: std::collections::HashSet<_> = exercise.test_cases.iter()
                .map(|tc| tc.output.trim())
                .collect();
            outputs.len() > 1
        } else {
            false
        };
        
        // Show warning if input is required but not mentioned in description
        if expects_input || has_different_inputs {
            let desc_lower = exercise.description.to_lowercase();
            if !desc_lower.contains("read") && !desc_lower.contains("input") && !desc_lower.contains("stdin") {
                println!();
                println!("{}", Colors::label_warn("CLARIFICATION").bold());
                println!("{}", Colors::warning("This exercise requires reading input from stdin.").bold());
                println!("{}", Colors::muted("Your program should read the input provided by each test case, not hardcode values."));
                println!();
            }
        } else if has_different_outputs_no_input {
            // This is likely a generation error - test cases have different outputs but no input
            // This shouldn't happen for simple exercises
            println!();
            println!("{}", Colors::label_warn("NOTE").bold());
            println!("{}", Colors::warning("The test cases expect different outputs. This may indicate the exercise should use hardcoded values instead."));
            println!("{}", Colors::muted("If this is a beginner exercise, you can use the same hardcoded values for all test cases."));
            println!();
        }

        // Display example input/output if provided and non-empty
        if let Some(ref example_input) = exercise.example_input {
            if !example_input.trim().is_empty() {
                println!();
                println!("{}", Colors::label_input("INPUT"));
                println!("{}", Colors::warning("Example Input:").bold());
                println!("  {}", Colors::text(example_input));
                println!();
                
                // Show how to read input if input is expected
                if expects_input {
                    println!("{}", Colors::label_warn("IMPORTANT").bold());
                    println!("{}", Colors::error("Your program must read this input!").bold());
                    match language {
                        crate::config::Language::JavaScript => {
                            println!("   Use {} to read input:", Colors::primary("readline() or process.stdin"));
                            println!("   {}", Colors::text("const readline = require('readline');"));
                            println!("   {}", Colors::text("const rl = readline.createInterface({ input: process.stdin });"));
                            println!("   {}", Colors::text("rl.on('line', (line) => { /* use line */ });"));
                            println!("   {}", Colors::text("Or use: const input = require('fs').readFileSync(0, 'utf-8').trim();"));
                        }
                        crate::config::Language::Cpp => {
                            println!("   Use {} to read input:", Colors::primary("cin or getline()"));
                            println!("   {}", Colors::text("#include <iostream>"));
                            println!("   {}", Colors::text("std::string input;"));
                            println!("   {}", Colors::text("std::getline(std::cin, input);"));
                            println!("   {}", Colors::text("Or: std::cin >> variable;"));
                        }
                        crate::config::Language::Rust => {
                            println!("   Use {} to read input:", Colors::primary("std::io::stdin()"));
                            println!("   {}", Colors::text("use std::io;"));
                            println!("   {}", Colors::text("let mut input = String::new();"));
                            println!("   {}", Colors::text("io::stdin().read_line(&mut input).expect(\"Failed to read\");"));
                            println!("   {}", Colors::text("let input = input.trim(); // Remove newline"));
                        }
                    }
                    println!();
                }
            }
        }

        if let Some(ref example_output) = exercise.example_output {
            if !example_output.trim().is_empty() {
                println!();
                println!("{}", Colors::label_output("OUTPUT"));
                println!("{}", Colors::warning("Expected Output:").bold());
                println!("  {}", Colors::text(example_output));
                println!();
                // Add a note about printing if output is expected
                if expects_output {
                    println!("{}", Colors::label_warn("IMPORTANT").bold());
                    println!("{}", Colors::error("Your code must produce this output!").bold());
                    match language {
                        crate::config::Language::JavaScript => {
                            println!("   Use {} to print output", Colors::primary("console.log()"));
                        }
                        crate::config::Language::Cpp => {
                            println!("   Use {} to print output", Colors::primary("cout << ... << endl;"));
                        }
                        crate::config::Language::Rust => {
                            println!("   Use {} to print output", Colors::primary("println!()"));
                        }
                    }
                    println!();
                }
            }
        }
        
        // If no input/output examples but there are test cases, show the first test case as an example
        if exercise.example_input.is_none() && exercise.example_output.is_none() && !exercise.test_cases.is_empty() {
            let first_test = &exercise.test_cases[0];
            println!("{}", Colors::label_info("EXAMPLE"));
            println!("{}", Colors::primary("Example (from test case):").bold());
            if !first_test.input.trim().is_empty() {
                println!("{}", Colors::label_input("INPUT"));
                println!("{}", Colors::warning("Input:").bold());
                println!("{}\n", Colors::text(&first_test.input));
                
                // Show how to read input if input is expected
                if expects_input {
                    println!("{}", Colors::label_warn("IMPORTANT").bold());
                    println!("{}", Colors::error("Your program must read this input!").bold());
                    match language {
                        crate::config::Language::JavaScript => {
                            println!("   Use {} to read input:", Colors::primary("readline() or process.stdin"));
                            println!("   {}", Colors::text("const readline = require('readline');"));
                            println!("   {}", Colors::text("const rl = readline.createInterface({ input: process.stdin });"));
                            println!("   {}", Colors::text("rl.on('line', (line) => { /* use line */ });"));
                            println!("   {}", Colors::text("Or use: const input = require('fs').readFileSync(0, 'utf-8').trim();"));
                        }
                        crate::config::Language::Cpp => {
                            println!("   Use {} to read input:", Colors::primary("cin or getline()"));
                            println!("   {}", Colors::text("#include <iostream>"));
                            println!("   {}", Colors::text("std::string input;"));
                            println!("   {}", Colors::text("std::getline(std::cin, input);"));
                            println!("   {}", Colors::text("Or: std::cin >> variable;"));
                        }
                        crate::config::Language::Rust => {
                            println!("   Use {} to read input:", Colors::primary("std::io::stdin()"));
                            println!("   {}", Colors::text("use std::io;"));
                            println!("   {}", Colors::text("let mut input = String::new();"));
                            println!("   {}", Colors::text("io::stdin().read_line(&mut input).expect(\"Failed to read\");"));
                            println!("   {}", Colors::text("let input = input.trim(); // Remove newline"));
                        }
                    }
                    println!();
                }
            }
            if !first_test.output.trim().is_empty() {
                println!("{}", Colors::label_output("OUTPUT"));
                println!("{}", Colors::warning("Expected Output:").bold());
                println!("{}\n", Colors::text(&first_test.output));
                // Add a note about printing if output is expected
                if expects_output {
                    println!("{}", Colors::label_warn("IMPORTANT").bold());
                    println!("{}", Colors::error("Your code must produce this output!").bold());
                    match language {
                        crate::config::Language::JavaScript => {
                            println!("   Use {} to print output", Colors::primary("console.log()"));
                        }
                        crate::config::Language::Cpp => {
                            println!("   Use {} to print output", Colors::primary("cout << ... << endl;"));
                        }
                        crate::config::Language::Rust => {
                            println!("   Use {} to print output", Colors::primary("println!()"));
                        }
                    }
                    println!();
                }
            }
        }

        // Show all test cases that will be used
        if !exercise.test_cases.is_empty() {
            print_section_header("TEST CASES", |s| Colors::primary(s));
            
            // Check if test cases have different inputs (indicates input reading is needed)
            let has_different_inputs = exercise.test_cases.iter()
                .filter(|tc| !tc.input.trim().is_empty())
                .map(|tc| tc.input.trim())
                .collect::<std::collections::HashSet<_>>()
                .len() > 1;
            
            if expects_input || has_different_inputs {
                println!("{}", Colors::text("Your code will be tested with the following inputs:"));
                println!("{}", Colors::label_warn("IMPORTANT").bold());
                println!("{}", Colors::error("Your program must READ INPUT from stdin!").bold());
            } else {
                println!("{}", Colors::text("Your code will be tested with the following cases:"));
            }
            println!();
            
            for (idx, test_case) in exercise.test_cases.iter().enumerate() {
                if !test_case.input.trim().is_empty() {
                    print!("  {} ", Colors::label_input(&format!("TEST {}", idx + 1)));
                    print!("{}", Colors::warning(&format!("Input = ")));
                    print!("{}", Colors::primary(&format!("\"{}\"", test_case.input)));
                    if !test_case.output.trim().is_empty() {
                        print!(" {} ", Colors::text("→"));
                        print!("{}", Colors::label_output("OUTPUT"));
                        println!(" {}", Colors::success(&format!("\"{}\"", test_case.output)));
                    } else {
                        println!();
                    }
                } else if !test_case.output.trim().is_empty() {
                    print!("  {} ", Colors::label_output(&format!("TEST {}", idx + 1)));
                    print!("{}", Colors::success(&format!("Expected Output = \"{}\"", test_case.output)));
                    println!();
                }
            }
            println!();
            
            // If test cases have different expected outputs but no input shown,
            // it likely means input is needed but not displayed - add a note
            if !expects_input && exercise.test_cases.len() > 1 {
                let outputs: std::collections::HashSet<_> = exercise.test_cases.iter()
                    .map(|tc| tc.output.trim())
                    .collect();
                if outputs.len() > 1 {
                    println!();
                    println!("{}", Colors::label_warn("NOTE"));
                    println!("{}", Colors::warning("Different test cases expect different outputs."));
                    println!("{}", Colors::muted("This suggests your program should read input from stdin."));
                    println!();
                }
            }
        }

        // Display hints if available
        if !exercise.hints.is_empty() {
            print_section_header("HINTS", |s| Colors::warning(s));
            for (idx, hint) in exercise.hints.iter().enumerate() {
                println!("  {}. {}", idx + 1, hint);
            }
            let mut hint_num = exercise.hints.len() + 1;
            
            // Add hint about input if expected
            if expects_input {
                match language {
                    crate::config::Language::JavaScript => {
                        println!("  {}. Your program needs to read input. Use {} or {}", hint_num, Colors::primary("readline()"), Colors::primary("process.stdin"));
                        hint_num += 1;
                    }
                    crate::config::Language::Cpp => {
                        println!("  {}. Your program needs to read input. Use {} or {}", hint_num, Colors::primary("std::cin"), Colors::primary("std::getline()"));
                        hint_num += 1;
                    }
                    crate::config::Language::Rust => {
                        println!("  {}. Your program needs to read input. Use {} to read from stdin", hint_num, Colors::primary("io::stdin().read_line()"));
                        hint_num += 1;
                    }
                }
            }
            
            if expects_output {
                match language {
                    crate::config::Language::JavaScript => {
                        println!("  {}. Remember to use {} to display your result", hint_num, Colors::primary("console.log()"));
                    }
                    crate::config::Language::Cpp => {
                        println!("  {}. Remember to use {} to display your result", hint_num, Colors::primary("cout"));
                    }
                    crate::config::Language::Rust => {
                        println!("  {}. Remember to use {} to display your result", hint_num, Colors::primary("println!()"));
                    }
                }
            }
            println!();
        } else {
            if expects_input || expects_output {
                println!("{}", Colors::label_tip("HINTS"));
                let mut hint_num = 1;
                
                if expects_input {
                    match language {
                        crate::config::Language::JavaScript => {
                            println!("  {}. Your program needs to read input. Use {} or {}", hint_num, Colors::primary("readline()"), Colors::primary("process.stdin"));
                            hint_num += 1;
                        }
                        crate::config::Language::Cpp => {
                            println!("  {}. Your program needs to read input. Use {} or {}", hint_num, Colors::primary("std::cin"), Colors::primary("std::getline()"));
                            hint_num += 1;
                        }
                        crate::config::Language::Rust => {
                            println!("  {}. Your program needs to read input. Use {} to read from stdin", hint_num, Colors::primary("io::stdin().read_line()"));
                            hint_num += 1;
                        }
                    }
                }
                
                if expects_output {
                    match language {
                        crate::config::Language::JavaScript => {
                            println!("  {}. Remember to use {} to display your result", hint_num, Colors::primary("console.log()"));
                        }
                        crate::config::Language::Cpp => {
                            println!("  {}. Remember to use {} to display your result", hint_num, Colors::primary("cout"));
                        }
                        crate::config::Language::Rust => {
                            println!("  {}. Remember to use {} to display your result", hint_num, Colors::primary("println!()"));
                        }
                    }
                }
                println!();
            }
        }

        // Create exercise file
        let file_path = FileManager::create_exercise_file(&language, exercise_number)?;
        
        // Retry loop - keep program open until tests pass or user skips
        let mut retry_count = 0;
        loop {
            retry_count += 1;
            
            println!("{}", Colors::info(&format!("Write your solution in: {}", file_path.display())));
            println!("{}", Colors::muted("Press Enter when you're ready to test your solution, or type 'skip' to skip this exercise (or Ctrl+C to exit)..."));

            let user_input = Text::new("").prompt();
            
            // Check if user wants to skip
            if let Ok(input) = &user_input {
                if input.trim().to_lowercase() == "skip" {
                    println!("{}", Colors::warning("Exercise skipped. Moving to next..."));
                    thread::sleep(Duration::from_millis(1000)); // Brief pause to show message
                    break; // Exit retry loop and skip to next exercise
                }
            }
            
            // If user cancelled or wants to continue, proceed with testing
            if user_input.is_err() {
                return Ok(()); // User cancelled
            }

            // Test the solution
            let mut all_passed = true;
            let mut errors = Vec::new();
            
            // Handle case where there are no test cases
            if exercise.test_cases.is_empty() {
                println!("{}", Colors::label_warn("WARN"));
                println!("{}", Colors::warning("No test cases provided for this exercise. Code will be executed but not validated."));
                // Just try to execute the code to check for syntax errors
                match Executor::execute(language, &file_path, None) {
                    Ok(_) => {
                        println!("{}", Colors::label_pass("PASS"));
                        println!("{}", Colors::success("Code executed successfully (no test cases to validate)"));
                        all_passed = true;
                    }
                    Err(e) => {
                        println!("{}", Colors::label_fail("FAIL"));
                        let error_msg = format!("{}", e);
                        println!("{}", Colors::error(&error_msg));
                        errors.push(error_msg);
                        all_passed = false;
                    }
                }
            } else {
                for (test_idx, test_case) in exercise.test_cases.iter().enumerate() {
                    match Executor::execute(language, &file_path, Some(&test_case.input)) {
                        Ok(result) => {
                            let passed = Executor::compare_output(&result.output, &test_case.output);
                            
                            if passed {
                                println!("{}", Colors::label_pass(&format!("TEST {} PASSED", test_idx + 1)));
                            } else {
                                println!("{}", Colors::label_fail(&format!("TEST {} FAILED", test_idx + 1)));
                                print!("Expected: ");
                                println!("{}", Colors::warning(&test_case.output));
                                if result.output.trim().is_empty() {
                                    print!("Got: ");
                                    println!("{}", Colors::error("(empty) (no output)"));
                                    println!();
                                    println!("{}", Colors::label_info("TIP"));
                                    println!("{}", Colors::info("Your code ran successfully but produced no output."));
                                    println!("{}", Colors::muted("If the exercise requires output, make sure to use console.log() (JS), cout (C++), or println!() (Rust)."));
                                } else {
                                    print!("Got: ");
                                    println!("{}", Colors::error(&result.output));
                                }
                                all_passed = false;
                            }
                        }
                        Err(e) => {
                            println!("{}", Colors::label_fail(&format!("TEST {} ERROR", test_idx + 1)));
                            let error_msg = format!("{}", e);
                            println!("{}", Colors::error(&error_msg));
                            errors.push(error_msg);
                            println!();
                            println!("{}", Colors::label_info("TIP"));
                            println!("{}", Colors::info("Check your code for syntax errors or missing output statements."));
                            all_passed = false;
                        }
                    }
                }
            }

            if all_passed {
                println!();
                println!("{}", Colors::label_pass("SUCCESS").bold());
                println!("{}", Colors::success("All tests passed!").bold());
                break; // Exit retry loop and move to next exercise
            } else {
                // Clear screen and re-display exercise context for clean view
                Self::clear_screen();
                
                // Re-display exercise header
                let exercise_header = format!("Exercise {}: {} (Attempt {})", exercise_number, exercise.title, retry_count + 1);
                let header_width = TERMINAL_WIDTH.min(exercise_header.chars().count() + 4);
                println!();
                println!("{}", Colors::primary(&Borders::top(header_width)));
                println!("{}", Colors::primary(&Borders::box_line(&exercise_header, header_width)));
                println!("{}", Colors::primary(&Borders::bottom(header_width)));
                
                // Re-display quick reference
                print_section_header("QUICK REFERENCE", |s| Colors::accent(s));
                
                // Show code examples FIRST
                if !content.code_examples.is_empty() {
                    println!("{}", Colors::warning("Example Code:").bold());
                    for (idx, example) in content.code_examples.iter().take(2).enumerate() {
                        println!("\n  {}", Colors::primary(&format!("Example {}:", idx + 1)));
                        // Calculate box width: find longest line, add padding, but cap at terminal width
                        let max_line_len = example.code.lines()
                            .map(|l| l.chars().count())
                            .max()
                            .unwrap_or(0);
                        // Cap at reasonable width (60 chars max for code), leaving margin for box borders and indentation
                        let effective_max = max_line_len.min(60).min(TERMINAL_WIDTH - 8);
                        let box_width = effective_max + 4; // Add padding for box borders (2 chars on each side)
                        println!("  {}", Colors::text(&Borders::top(box_width)));
                        for line in example.code.lines() {
                            // Truncate line if it's too long
                            let display_line = if line.chars().count() > effective_max {
                                &line[..effective_max.min(line.len())]
                            } else {
                                line
                            };
                            println!("  {}", Borders::box_line_left(display_line, box_width));
                        }
                        println!("  {}", Colors::text(&Borders::bottom(box_width)));
                        if let Some(first_sentence) = example.explanation.split('.').next() {
                            println!("  {} {}\n", Colors::label_info("TIP"), first_sentence.trim());
                        }
                    }
                }
                
                println!("{}", Colors::primary(&Borders::separator(TERMINAL_WIDTH)));
                print_section_header("EXERCISE INSTRUCTIONS", |s| Colors::success(s));
                print_wrapped(&exercise.description, TERMINAL_WIDTH, 0);
                println!();
                
                // Show test results summary
                print_section_header("TEST RESULTS", |s| Colors::error(s));
                println!("{}", Colors::error("Some tests failed. Try again!").bold());
                
                // Show errors if any
                if !errors.is_empty() {
                    println!();
                    println!("{}", Colors::label_fail("ERRORS").bold());
                    for (idx, error) in errors.iter().enumerate() {
                        println!("  {}. {}", idx + 1, Colors::error(error));
                    }
                }
                
                // Show helpful debugging info
                print_section_header("DEBUGGING TIPS", |s| Colors::info(s));
                println!("  1. Make sure your code produces the expected output");
                println!("  2. Check that you're using the correct syntax for your language");
                println!("  3. Verify your code runs without errors");
                
                if !exercise.hints.is_empty() {
                    println!("\n{}", Colors::warning("Remember the hints:").bold());
                    for (idx, hint) in exercise.hints.iter().enumerate() {
                        print!("  {}. ", Colors::warning(&(idx + 1).to_string()).bold());
                        print_wrapped(hint, TERMINAL_WIDTH - 4, 4);
                        println!();
                    }
                }
                
                println!();
                println!("{}", Colors::info(&format!("Write your solution in: {}", file_path.display())));
                println!("{}", Colors::muted("Review the syntax guide and examples above, then try again."));
                
                // Ask if user wants to retry or skip
                let retry = Confirm::new("Would you like to try again? (Edit your code and press Enter to test)")
                    .with_default(true)
                    .prompt();
                
                match retry {
                    Ok(true) => {
                        println!();
                        println!("{}", Colors::info("Edit your code and press Enter when ready to test again..."));
                        // Loop will continue
                    }
                    Ok(false) => {
                        println!();
                        println!("{}", Colors::warning("Skipping this exercise. Moving to next..."));
                        break; // Skip this exercise and move to next
                    }
                    Err(_) => {
                        // User cancelled (Ctrl+C or similar)
                        println!();
                        println!("{}", Colors::warning("Lesson interrupted. Progress not saved."));
                        return Err(anyhow::anyhow!("Lesson interrupted by user")); // Return error so journey mode knows lesson wasn't completed
                    }
                }
            }
        }

        Ok(())
    }
}
