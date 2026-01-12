use crate::cli::colors::Colors;
use crate::config::{curriculum::Curriculum, Language};
use crate::lessons::LessonManager;
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
        let curriculum = Curriculum::get_for_language(language);
        let mut journey_progress = self.tracker.get_journey_progress()?;

        // Start new journey if none exists or different language
        let needs_new_journey = journey_progress.is_none() || 
            journey_progress.as_ref().map(|j| j.language != language).unwrap_or(true);
        if needs_new_journey {
            println!();
            println!("{}", Colors::primary("Starting Learning Journey!").bold());
            println!("{}", Colors::primary(&"=".repeat(60)));
            println!("Language: {}", Colors::warning(language.display_name()));
            println!("Total Stages: {}\n", Colors::success(&curriculum.total_stages().to_string()));
            
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

            let stage = match curriculum.get_stage(journey.current_stage) {
                Some(s) => s,
                None => {
                    println!();
                    println!("{}", Colors::label_pass("SUCCESS").bold());
                    println!("{}", Colors::success("Congratulations! You've completed the entire journey!").bold());
                    self.tracker.reset_journey()?;
                    break;
                }
            };

            // Check if we've completed all topics in this stage
            if journey.current_topic_index >= stage.topics.len() {
                println!();
                println!("{}", Colors::label_pass("SUCCESS"));
                println!("{}", Colors::success(&format!("Stage {}: {} completed!", journey.current_stage + 1, stage.name)).bold());
                
                // Move to next stage
                if journey.current_stage + 1 < curriculum.total_stages() {
                    let continue_journey = match Confirm::new("Continue to next stage?")
                        .with_default(true)
                        .prompt() {
                        Ok(true) => true,
                        Ok(false) => false,
                        Err(_) => {
                            println!("\n{}", Colors::warning("Exiting journey..."));
                            return Ok(());
                        }
                    };
                    
                    if continue_journey {
                        self.tracker.advance_journey_stage()?;
                        continue;
                    } else {
                        break;
                    }
                } else {
                    println!("\n{}", Colors::success("You've completed all stages! Amazing work!").bold());
                    self.tracker.reset_journey()?;
                    break;
                }
            }

            if journey.current_topic_index >= stage.topics.len() {
                if journey.current_stage + 1 < curriculum.total_stages() {
                    self.tracker.advance_journey_stage()?;
                    continue;
                } else {
                    println!("\n{}", Colors::success("You've completed all stages! Amazing work!").bold());
                    self.tracker.reset_journey()?;
                    break;
                }
            }

            let topic = &stage.topics[journey.current_topic_index];
            
            println!("\n{}", Colors::primary(&"=".repeat(60)));
            println!("{}", Colors::primary(&format!("Stage {}/{}: {}", journey.current_stage + 1, curriculum.total_stages(), stage.name)).bold());
            println!("{}", Colors::warning(&format!("Topic {}/{}: {}", journey.current_topic_index + 1, stage.topics.len(), topic)).bold());
            println!("{}", Colors::warning(&format!("Difficulty: {}", stage.difficulty.display_name())));
            println!("{}", Colors::primary(&"=".repeat(60)));

            // Start the lesson
            let lesson_result = self.lesson_manager.start_lesson(
                language,
                stage.difficulty,
                stage.lesson_type,
                topic.clone(),
            );
            
            // Only mark topic as completed if lesson finished successfully
            match lesson_result {
                Ok(_) => {
                    self.tracker.complete_journey_topic(topic.clone())?;
                }
                Err(_e) => {
                    println!("\n{}", Colors::warning("Lesson not completed. Topic progress not saved."));
                    println!("{}", Colors::info("Use 'cursed-coddy journey' to continue from where you left off."));
                    return Ok(());
                }
            }

            match Confirm::new("Continue with next topic?")
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
            let curriculum = Curriculum::get_for_language(journey.language);
            
            println!("\n{}", Colors::primary("Learning Journey Status").bold());
            println!("{}", Colors::primary(&"=".repeat(60)));
            println!("Language: {}", Colors::warning(journey.language.display_name()));
            println!("Current Stage: {}/{}", journey.current_stage + 1, curriculum.total_stages());
            
            if let Some(stage) = curriculum.get_stage(journey.current_stage) {
                println!("Stage: {}", Colors::primary(&stage.name));
                println!("Progress: {}/{} topics completed", journey.current_topic_index, stage.topics.len());
                
                if !journey.completed_topics.is_empty() {
                    println!("\n{}", Colors::success("Completed Topics:"));
                    for topic in &journey.completed_topics {
                        println!("  {} {}", Colors::label_pass("OK"), Colors::success(topic));
                    }
                }
            }
        } else {
            println!("{}", Colors::warning("No active journey. Start one with 'cursed-coddy journey'."));
        }

        Ok(())
    }
}
