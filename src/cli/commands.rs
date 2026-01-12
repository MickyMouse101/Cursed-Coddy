use crate::cli::colors::Colors;
use crate::config::{Difficulty, Language, LessonType};
use crate::lessons::{JourneyManager, LessonManager};
use crate::progress::Tracker;
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use inquire::{Select, Text};
use rand::Rng;

#[derive(Parser)]
#[command(name = "cursed-coddy")]
#[command(about = "A CLI coding education platform", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new lesson
    Start,
    /// Continue from where you left off
    Continue,
    /// Start or continue learning journey (campaign mode)
    Journey,
    /// Learn how to compile/build programs
    Compile,
    /// Show your progress
    Progress,
    /// Show help
    Help,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Start) => handle_start()?,
        Some(Commands::Continue) => handle_continue()?,
        Some(Commands::Journey) => handle_journey()?,
        Some(Commands::Compile) => handle_compile()?,
        Some(Commands::Progress) => handle_progress()?,
        Some(Commands::Help) | None => handle_help(),
    }

    Ok(())
}

fn handle_start() -> Result<()> {
    println!("{}", Colors::primary("Welcome to Cursed Coddy!").bold());
    println!("{}", Colors::primary(&"=".repeat(60)));

    // Select language
    let language_options = vec![
        Language::JavaScript,
        Language::Cpp,
        Language::Rust,
    ];
    let language = Select::new("Select a language:", language_options)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;

    // Select difficulty
    let difficulty_options = vec![
        Difficulty::Beginner,
        Difficulty::Intermediate,
        Difficulty::Advanced,
    ];
    let difficulty = Select::new("Select difficulty:", difficulty_options)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;

    // Select lesson type
    let lesson_type_options = vec![LessonType::Short, LessonType::Medium, LessonType::Long];
    let lesson_type = Select::new("Select lesson type:", lesson_type_options)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;

    // Enter topic (or leave blank for random selection)
    let topic = {
        let input = Text::new("Enter a topic (e.g., 'variables', 'functions', 'loops') or leave blank for random:")
            .prompt()
            .map_err(|e| anyhow::anyhow!("Input cancelled: {}", e))?;
        
        let trimmed = input.trim();
        if trimmed.is_empty() {
            // Select a random topic from common programming topics
            let random_topics = vec![
                "variables",
                "functions",
                "loops",
                "conditionals",
                "arrays",
                "strings",
                "data types",
                "operators",
                "control flow",
                "error handling",
                "recursion",
                "object-oriented programming",
                "memory management",
                "pointers",
                "iterators",
                "closures",
                "modules",
                "packages",
                "testing",
                "debugging",
            ];
            let mut rng = rand::thread_rng();
            let random_topic = random_topics[rng.gen_range(0..random_topics.len())].to_string();
            println!("{}", Colors::warning(&format!("No topic entered. Selected random topic: {}", random_topic)).bold());
            random_topic
        } else {
            trimmed.to_string()
        }
    };

    // Start lesson
    let manager = LessonManager::new()?;
    manager.start_lesson(language, difficulty, lesson_type, topic)?;

    Ok(())
}

fn handle_continue() -> Result<()> {
    let tracker = Tracker::new()?;
    let progress = tracker.load()?;

    if let Some(lesson) = progress.current_lesson {
        println!("{}", Colors::primary("Resuming lesson..."));
        println!("Language: {}", lesson.language.display_name());
        println!("Difficulty: {}", lesson.difficulty.display_name());
        println!("Topic: {}", lesson.topic);
        println!(
            "Progress: {}/{} exercises",
            lesson.current_exercise, lesson.total_exercises
        );
        println!("{}", Colors::warning("Resume functionality coming soon!"));
    } else {
        println!("{}", Colors::warning("No lesson in progress. Start a new lesson with 'start'."));
    }

    Ok(())
}

fn handle_progress() -> Result<()> {
    let tracker = Tracker::new()?;
    let progress = tracker.load()?;

    println!("{}", Colors::primary("Your Progress").bold());
    println!("{}", Colors::primary(&"=".repeat(60)));
    println!(
        "Total lessons completed: {}",
        Colors::success(&progress.statistics.total_lessons_completed.to_string())
    );

    if !progress.statistics.lessons_by_language.is_empty() {
        println!("\n{}", Colors::warning("By Language:"));
        for (lang, count) in &progress.statistics.lessons_by_language {
            println!("  {}: {}", Colors::primary(lang), count);
        }
    }

    if !progress.statistics.lessons_by_difficulty.is_empty() {
        println!("\n{}", Colors::warning("By Difficulty:"));
        for (diff, count) in &progress.statistics.lessons_by_difficulty {
            println!("  {}: {}", Colors::primary(diff), count);
        }
    }

    if !progress.completed_lessons.is_empty() {
        println!("\n{}", Colors::warning("Recent Lessons:"));
        for lesson in progress.completed_lessons.iter().rev().take(5) {
            println!(
                "  {} - {} ({})",
                Colors::primary(&lesson.topic),
                lesson.language.display_name(),
                lesson.difficulty.display_name()
            );
        }
    }

    if progress.journey_progress.is_some() {
        println!("\n{}", Colors::primary("Learning Journey:").bold());
        let journey_manager = JourneyManager::new()?;
        journey_manager.show_journey_status()?;
    }

    Ok(())
}

fn handle_journey() -> Result<()> {
    println!("{}", Colors::primary("Learning Journey Mode").bold());
    println!("{}", Colors::primary(&"=".repeat(60)));

    // Check if there's an existing journey
    let tracker = Tracker::new()?;
    let existing_journey = tracker.get_journey_progress()?;

    let language = if let Some(ref journey) = existing_journey {
        println!("Found existing journey for: {}", Colors::warning(journey.language.display_name()));
        
        // Show journey status
        let journey_manager = JourneyManager::new()?;
        journey_manager.show_journey_status()?;
        println!();
        
        // Give user options
        let options = vec!["Continue existing journey", "Reset and start fresh", "Start new journey (different language)"];
        let choice = Select::new("What would you like to do?", options)
            .prompt()
            .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;
        
        match choice {
            "Continue existing journey" => {
                journey.language
            }
            "Reset and start fresh" => {
                let confirm = inquire::Confirm::new("Are you sure you want to reset your journey progress? This cannot be undone.")
                    .with_default(false)
                    .prompt()
                    .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;
                
                if confirm {
                    tracker.reset_journey()?;
                    println!("{}", Colors::success("Journey progress reset!"));
                    println!();
                    
                    // Select language for new journey
                    let language_options = vec![
                        Language::JavaScript,
                        Language::Cpp,
                        Language::Rust,
                    ];
                    Select::new("Select a language for your new journey:", language_options)
                        .prompt()
                        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?
                } else {
                    println!("Reset cancelled.");
                    return Ok(());
                }
            }
            "Start new journey (different language)" => {
                // Select new language (this will automatically reset the old one)
                let language_options = vec![
                    Language::JavaScript,
                    Language::Cpp,
                    Language::Rust,
                ];
                Select::new("Select a language for your journey:", language_options)
                    .prompt()
                    .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?
            }
            _ => {
                return Ok(());
            }
        }
    } else {
        // Select language
        let language_options = vec![
            Language::JavaScript,
            Language::Cpp,
            Language::Rust,
        ];
        Select::new("Select a language for your learning journey:", language_options)
            .prompt()
            .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?
    };

    let journey_manager = JourneyManager::new()?;
    journey_manager.start_or_continue_journey(language)?;

    Ok(())
}

fn handle_compile() -> Result<()> {
    use crate::cli::banner;
    use crate::lessons::LessonManager;
    
    banner::display_banner();
    println!("{}", Colors::primary("Compilation & Build Guides").bold());
    println!("{}", Colors::primary(&"=".repeat(60)));
    println!();
    println!("Learn how to compile and build programs for each language.");
    println!();

    // Select language
    let language_options = vec![
        Language::JavaScript,
        Language::Cpp,
        Language::Rust,
    ];
    let language = Select::new("Select a language:", language_options)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;

    // Generate topic based on language
    let topic = match language {
        Language::JavaScript => "JavaScript execution with Node.js and running JavaScript programs",
        Language::Cpp => "C++ compilation using g++ compiler and CMake build system",
        Language::Rust => "Rust compilation with rustc compiler and Cargo package manager",
    };

    // Start lesson with compilation topic
    let manager = LessonManager::new()?;
    manager.start_lesson(
        language,
        Difficulty::Beginner,
        LessonType::Short,
        topic.to_string(),
    )?;

    Ok(())
}

fn handle_help() {
    println!("{}", Colors::primary("Cursed Coddy - CLI Coding Education Platform").bold());
    println!();
    println!("Commands:");
    println!("  start     - Start a new lesson (free mode)");
    println!("  journey   - Start or continue learning journey (campaign mode)");
    println!("  compile   - Learn how to compile/build programs for each language");
    println!("  continue  - Continue from where you left off");
    println!("  progress  - Show your learning progress");
    println!("  help      - Show this help message");
    println!();
    println!("{}", Colors::warning("Learning Journey:"));
    println!("  A structured curriculum that guides you from basics to advanced topics.");
    println!("  Progresses automatically through stages, scaling difficulty over time.");
    println!();
    println!("Make sure Ollama is running on http://localhost:11434");
}
