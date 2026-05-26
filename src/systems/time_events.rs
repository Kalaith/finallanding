use serde::{Deserialize, Serialize};

/// Events that are triggered by the time system
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeEvent {
    NewDay { day: u32 },
    HourChanged { hour: u32 },
    DawnBreak, // 6:00
    Dusk,      // 20:00
}

/// Collects time events for a given tick update.
/// Other systems can consume these events.
pub struct TimeEventCollector {
    pub events: Vec<TimeEvent>,
}

impl TimeEventCollector {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn push(&mut self, event: TimeEvent) {
        self.events.push(event);
    }
}

impl Default for TimeEventCollector {
    fn default() -> Self {
        Self::new()
    }
}

// Placeholder: Agent 1 or 3 can expand this to actually dispatch events
// to colonists or other game systems.
