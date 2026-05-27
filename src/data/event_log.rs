use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogCategory {
    Time,
    Social,
    Work,
    Mood,
    Resource,
    Mission,
    Technology,
    Colony,
    System,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyLogEntry {
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub category: LogCategory,
    pub title: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SocialHistoryEntry {
    pub day: u32,
    pub title: String,
    pub detail: String,
    pub recommendation: String,
    pub average_mood: f32,
    pub average_relationship: f32,
    pub close_pairs: u32,
    pub strained_pairs: u32,
}

impl SocialHistoryEntry {
    pub fn new(
        day: u32,
        title: impl Into<String>,
        detail: impl Into<String>,
        recommendation: impl Into<String>,
        average_mood: f32,
        average_relationship: f32,
        close_pairs: u32,
        strained_pairs: u32,
    ) -> Self {
        Self {
            day,
            title: title.into(),
            detail: detail.into(),
            recommendation: recommendation.into(),
            average_mood,
            average_relationship,
            close_pairs,
            strained_pairs,
        }
    }
}

impl ColonyLogEntry {
    pub fn new(
        day: u32,
        hour: u32,
        minute: u32,
        category: LogCategory,
        title: String,
        detail: String,
    ) -> Self {
        Self {
            day,
            hour,
            minute,
            category,
            title,
            detail,
        }
    }
}
