use crate::cli::colors::Colors;
use crate::config::Language;
use crate::lessons::{HumanLessons, LessonManager};
use crate::progress::Tracker;
use anyhow::Result;
use colored::Colorize;
use inquire::Confirm;

pub struct JourneyManager {
    lesson_manager: LessonManager,
    tracker: Tracker,
}

impl JourneyManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            lesson_manager: LessonManager::new()?,
            tracker: Tracker::new()?,
        })
    }

    pub fn start_or_continue_journey(&self, language: Language) -> Result<()> {
        let mut journey_progress = self.tracker.get_journey_progress()?;

        // Start new journey if none exists or different language
        let needs_new_journey = journey_progress.is_none() || 
            journey_progress.as_ref().map(|j| j.language != language).unwrap_or(true);
        if needs_new_journey {
            println!();
            println!("{}", Colors::primary("Starting Learning Journey!").bold());
            println!("{}", Colors::primary(&"=".repeat(60)));
            println!("Language: {}", Colors::warning(language.display_name()));
            println!("{}", Colors::info("Human-made lessons with guided progression\n"));
            
            self.tracker.start_journey(language)?;
        }

        // Continue from current position
        loop {
            // Reload progress at start of each iteration
            journey_progress = self.tracker.get_journey_progress()?;
            let journey = match journey_progress.as_ref() {
                Some(j) => j,
                None => break,
            };

            // Use current_stage as the last lesson index
            let last_index = if journey.current_stage == 0 && journey.completed_topics.is_empty() {
                None // First lesson
            } else {
                Some(journey.current_stage)
            };

            // Get next human lesson
            let (lesson, lesson_index) = match HumanLessons::get_next_lesson(last_index, language) {
                Some((lesson, idx)) => (lesson, idx),
                None => {
                    println!();
                    println!("{}", Colors::warning("No human-made lessons available for this language."));
                    return Ok(());
                }
            };

            // Extract lesson title before moving lesson.content
            let lesson_title_short = lesson.content.concept.split('.').next().unwrap_or("Human-made lesson").to_string();
            let lesson_title = format!("Lesson {}: {}", lesson_index + 1, lesson_title_short);

            println!("\n{}", Colors::primary(&"=".repeat(60)));
            println!("{}", Colors::primary(&format!("Lesson {}: {}", lesson_index + 1, lesson_title_short)).bold());
            println!("{}", Colors::warning(&format!("Difficulty: {}", lesson.difficulty.display_name())));
            println!("{}", Colors::primary(&"=".repeat(60)));

            // Start the lesson
            let topic = format!("Human-made lesson {}", lesson_index + 1);
            let lesson_result = self.lesson_manager.start_lesson_with_content(
                lesson.language,
                lesson.difficulty,
                lesson.lesson_type,
                topic,
                lesson.content,
            );
            
            // Mark lesson as completed if finished successfully
            match lesson_result {
                Ok(_) => {
                    self.tracker.complete_journey_lesson(lesson_index, lesson_title)?;
                }
                Err(_e) => {
                    println!("\n{}", Colors::warning("Lesson not completed. Progress not saved."));
                    println!("{}", Colors::info("Use 'cursed-coddy journey' to continue from where you left off."));
                    return Ok(());
                }
            }

            match Confirm::new("Continue with next lesson?")
                .with_default(true)
                .prompt() {
                Ok(true) => {}
                Ok(false) => {
                    println!("\n{}", Colors::info("Journey paused. Use 'cursed-coddy journey' to continue."));
                    return Ok(());
                }
                Err(_) => {
                    println!("\n{}", Colors::warning("Exiting journey..."));
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub fn show_journey_status(&self) -> Result<()> {
        let journey_progress = self.tracker.get_journey_progress()?;

        if let Some(journey) = journey_progress {
            println!("\n{}", Colors::primary("Learning Journey Status").bold());
            println!("{}", Colors::primary(&"=".repeat(60)));
            println!("Language: {}", Colors::warning(journey.language.display_name()));
            println!("Current Lesson: {}", Colors::success(&format!("Lesson {}", journey.current_stage + 1)));
            println!("Lessons Completed: {}", Colors::success(&journey.completed_topics.len().to_string()));
            
            if !journey.completed_topics.is_empty() {
                println!("\n{}", Colors::success("Completed Lessons:"));
                for lesson in &journey.completed_topics {
                    println!("  {} {}", Colors::label_pass("OK"), Colors::success(lesson));
                }
            }
        } else {
            println!("{}", Colors::warning("No active journey. Start one with 'cursed-coddy journey'."));
        }

        Ok(())
    }
}
