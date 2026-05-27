use serde::{Deserialize, Serialize};

pub const BASE_STORAGE_CAPACITY: i32 = 36;
pub const STORAGE_CAPACITY_BONUS: i32 = 18;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColonyCondition {
    Stable,
    Strained,
    Critical,
    Collapsed,
}

impl ColonyCondition {
    pub fn label(&self) -> &'static str {
        match self {
            ColonyCondition::Stable => "Stable",
            ColonyCondition::Strained => "Strained",
            ColonyCondition::Critical => "Critical",
            ColonyCondition::Collapsed => "Collapsed",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceState {
    pub supplies: i32,
    pub salvage: i32,
    pub prepared_meals: i32,
    pub exploration_progress: u32,
    pub workshop_progress: u32,
    pub kitchen_progress: u32,
    pub hauling_progress: u32,
    pub stable_days: u32,
    pub condition: ColonyCondition,
}

impl Default for ResourceState {
    fn default() -> Self {
        Self {
            supplies: 30,
            salvage: 52,
            prepared_meals: 0,
            exploration_progress: 0,
            workshop_progress: 0,
            kitchen_progress: 0,
            hauling_progress: 0,
            stable_days: 0,
            condition: ColonyCondition::Strained,
        }
    }
}

impl ResourceState {
    pub fn spend_salvage(&mut self, amount: i32) -> bool {
        if amount < 0 || self.salvage < amount {
            return false;
        }

        self.salvage -= amount;
        true
    }

    pub fn refund_salvage(&mut self, amount: i32) {
        self.salvage += amount.max(0);
    }

    pub fn consume_supplies(&mut self, amount: i32) -> i32 {
        let amount = amount.max(0);
        let consumed = self.supplies.min(amount);
        self.supplies -= consumed;
        amount - consumed
    }

    pub fn add_supplies(&mut self, amount: i32, capacity: i32) -> i32 {
        let amount = amount.max(0);
        let space = (capacity - self.supplies).max(0);
        let stored = amount.min(space);
        self.supplies += stored;
        amount - stored
    }

    pub fn add_salvage(&mut self, amount: i32) {
        self.salvage += amount.max(0);
    }
}
