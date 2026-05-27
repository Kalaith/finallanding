use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentType {
    RationSpoilage,
    HabitatConflict,
    ToolBreakage,
    MoraleDip,
    StormWarning,
}

impl IncidentType {
    pub fn title(self) -> &'static str {
        match self {
            IncidentType::RationSpoilage => "Rations spoiled",
            IncidentType::HabitatConflict => "Habitat conflict",
            IncidentType::ToolBreakage => "Tools damaged",
            IncidentType::MoraleDip => "Morale dipped",
            IncidentType::StormWarning => "Storm warning",
        }
    }

    pub fn advisor_title(self) -> &'static str {
        match self {
            IncidentType::RationSpoilage => "Replace lost rations",
            IncidentType::HabitatConflict => "Ease habitat tension",
            IncidentType::ToolBreakage => "Recover repair stock",
            IncidentType::MoraleDip => "Stabilize morale",
            IncidentType::StormWarning => "Prepare for the storm",
        }
    }

    pub fn advisor_detail(self) -> &'static str {
        match self {
            IncidentType::RationSpoilage => {
                "Use Stockpile priority, meal prep, or a Supply run before the next dawn."
            }
            IncidentType::HabitatConflict => {
                "Recovery priority and more habitat capacity reduce nightly relationship strain."
            }
            IncidentType::ToolBreakage => {
                "Workshop or storage labor can replace the salvage lost to broken tools."
            }
            IncidentType::MoraleDip => {
                "Recovery priority lowers work pressure before refusals cascade."
            }
            IncidentType::StormWarning => {
                "Keep supplies above daily need and avoid risky mission timing tonight."
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveIncident {
    pub incident_type: IncidentType,
    pub expires_tick: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IncidentState {
    #[serde(default)]
    pub triggered: Vec<IncidentType>,
    #[serde(default)]
    pub active: Vec<ActiveIncident>,
}

impl IncidentState {
    pub fn has_triggered(&self, incident_type: IncidentType) -> bool {
        self.triggered.contains(&incident_type)
    }

    pub fn mark_triggered(&mut self, incident_type: IncidentType) {
        if !self.has_triggered(incident_type) {
            self.triggered.push(incident_type);
        }
    }

    pub fn activate(&mut self, incident_type: IncidentType, current_tick: u64, duration: u64) {
        self.active
            .retain(|incident| incident.incident_type != incident_type);
        self.active.push(ActiveIncident {
            incident_type,
            expires_tick: current_tick + duration,
        });
    }

    pub fn clear_expired(&mut self, current_tick: u64) {
        self.active
            .retain(|incident| incident.expires_tick > current_tick);
    }

    pub fn active_incident(&self, current_tick: u64) -> Option<IncidentType> {
        self.active
            .iter()
            .filter(|incident| incident.expires_tick > current_tick)
            .min_by_key(|incident| incident.expires_tick)
            .map(|incident| incident.incident_type)
    }
}
