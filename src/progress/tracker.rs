use crate::config::{Difficulty, Language, LessonType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub completed_lessons: Vec<LessonRecord>,
    pub current_lesson: Option<LessonState>,
    pub statistics: Statistics,
    #[serde(default)]
    pub journey_progress: Option<JourneyProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonRecord {
    pub language: Language,
    pub difficulty: Difficulty,
    pub lesson_type: LessonType,
    pub topic: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonState {
    pub language: Language,
    pub difficulty: Difficulty,
    pub lesson_type: LessonType,
    pub topic: String,
    pub current_exercise: usize,
    pub total_exercises: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_lessons_completed: usize,
    pub lessons_by_language: std::collections::HashMap<String, usize>,
    pub lessons_by_difficulty: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyProgress {
    pub language: Language,
    pub current_stage: usize,
    pub current_topic_index: usize,
    pub completed_topics: Vec<String>,
    pub started_at: String,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            completed_lessons: Vec::new(),
            current_lesson: None,
            statistics: Statistics {
                total_lessons_completed: 0,
                lessons_by_language: std::collections::HashMap::new(),
                lessons_by_difficulty: std::collections::HashMap::new(),
            },
            journey_progress: None,
        }
    }
}

pub struct Tracker {
    progress_file: PathBuf,
}

impl Tracker {
    pub fn new() -> Result<Self> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not find home directory")?;
        let progress_dir = PathBuf::from(home_dir).join(".cursed-coddy");
        std::fs::create_dir_all(&progress_dir)
            .context("Failed to create progress directory")?;
        let progress_file = progress_dir.join("progress.json");

        Ok(Self { progress_file })
    }

    pub fn load(&self) -> Result<Progress> {
        if !self.progress_file.exists() {
            return Ok(Progress::default());
        }

        let content = std::fs::read_to_string(&self.progress_file)
            .context("Failed to read progress file")?;
        
        if content.trim().is_empty() {
            return Ok(Progress::default());
        }
        
        serde_json::from_str::<Progress>(&content)
            .map_err(|_| anyhow::anyhow!("Failed to parse progress file. File may be corrupted. Try deleting it and starting fresh."))
    }

    pub fn save(&self, progress: &Progress) -> Result<()> {
        let content = serde_json::to_string_pretty(progress)
            .context("Failed to serialize progress")?;
        std::fs::write(&self.progress_file, content)
            .context("Failed to write progress file")?;
        Ok(())
    }

    pub fn start_lesson(
        &self,
        language: Language,
        difficulty: Difficulty,
        lesson_type: LessonType,
        topic: String,
        total_exercises: usize,
    ) -> Result<()> {
        let mut progress = self.load()?;
        progress.current_lesson = Some(LessonState {
            language,
            difficulty,
            lesson_type,
            topic,
            current_exercise: 0,
            total_exercises,
        });
        self.save(&progress)
    }

    pub fn complete_exercise(&self) -> Result<()> {
        let mut progress = self.load()?;
        if let Some(ref mut lesson) = progress.current_lesson {
            lesson.current_exercise += 1;
        }
        self.save(&progress)
    }

    pub fn complete_lesson(&self) -> Result<()> {
        let mut progress = self.load()?;
        if let Some(lesson) = progress.current_lesson.take() {
            let record = LessonRecord {
                language: lesson.language,
                difficulty: lesson.difficulty,
                lesson_type: lesson.lesson_type,
                topic: lesson.topic.clone(),
                completed_at: format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
            };
            progress.completed_lessons.push(record);
            progress.statistics.total_lessons_completed += 1;
            *progress
                .statistics
                .lessons_by_language
                .entry(lesson.language.display_name().to_string())
                .or_insert(0) += 1;
            *progress
                .statistics
                .lessons_by_difficulty
                .entry(lesson.difficulty.display_name().to_string())
                .or_insert(0) += 1;
        }
        self.save(&progress)
    }

    pub fn start_journey(&self, language: Language) -> Result<()> {
        let mut progress = self.load()?;
        progress.journey_progress = Some(JourneyProgress {
            language,
            current_stage: 0,
            current_topic_index: 0,
            completed_topics: Vec::new(),
            started_at: format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
        });
        self.save(&progress)
    }

    pub fn get_journey_progress(&self) -> Result<Option<JourneyProgress>> {
        let progress = self.load()?;
        Ok(progress.journey_progress.clone())
    }

    pub fn complete_journey_lesson(&self, lesson_index: usize, lesson_title: String) -> Result<()> {
        let mut progress = self.load()?;
        if let Some(ref mut journey) = progress.journey_progress {
            journey.current_stage = lesson_index;
            if !journey.completed_topics.contains(&lesson_title) {
                journey.completed_topics.push(lesson_title);
            }
        }
        self.save(&progress)
    }

    pub fn reset_journey(&self) -> Result<()> {
        let mut progress = self.load()?;
        progress.journey_progress = None;
        self.save(&progress)
    }

}
