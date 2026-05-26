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
