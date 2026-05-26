use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    Sleep,
    Work,
    Relax,
    Eat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub start_hour: u32,
    pub end_hour: u32,
    pub activity: ActivityType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub entries: Vec<ScheduleEntry>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            entries: vec![
                ScheduleEntry {
                    start_hour: 22,
                    end_hour: 6,
                    activity: ActivityType::Sleep,
                },
                ScheduleEntry {
                    start_hour: 6,
                    end_hour: 7,
                    activity: ActivityType::Eat,
                },
                ScheduleEntry {
                    start_hour: 7,
                    end_hour: 18,
                    activity: ActivityType::Work,
                },
                ScheduleEntry {
                    start_hour: 18,
                    end_hour: 20,
                    activity: ActivityType::Relax,
                },
                ScheduleEntry {
                    start_hour: 20,
                    end_hour: 22,
                    activity: ActivityType::Eat,
                },
            ],
        }
    }
}

impl Schedule {
    pub fn new_randomized() -> Self {
        use macroquad::rand::gen_range;

        // Base times with variance (+/- 1-2 hours)
        let wake_variance = gen_range(-1, 2); // -1, 0, or 1
        let sleep_variance = gen_range(-1, 2);

        let wake_time = (6i32 + wake_variance).clamp(4, 9) as u32;
        let sleep_time = (22i32 + sleep_variance).clamp(20, 24) as u32;

        let work_start = wake_time + 1;
        let work_end = 18;

        // Lunch/Dinner shifts
        let _lunch_start = 12 + gen_range(0, 2); // 12 or 13
        let dinner_start = 19 + gen_range(0, 2); // 19 or 20

        // Construct entries (simplified for now, overwrites overlap logic)
        Self {
            entries: vec![
                ScheduleEntry {
                    start_hour: sleep_time,
                    end_hour: wake_time,
                    activity: ActivityType::Sleep,
                }, // Wraps
                ScheduleEntry {
                    start_hour: wake_time,
                    end_hour: work_start,
                    activity: ActivityType::Eat,
                }, // Breakfast
                ScheduleEntry {
                    start_hour: work_start,
                    end_hour: work_end,
                    activity: ActivityType::Work,
                },
                ScheduleEntry {
                    start_hour: work_end,
                    end_hour: dinner_start,
                    activity: ActivityType::Relax,
                },
                ScheduleEntry {
                    start_hour: dinner_start,
                    end_hour: sleep_time,
                    activity: ActivityType::Eat,
                }, // Dinner/Evening
            ],
        }
    }

    pub fn get_activity_for_hour(&self, hour: u32) -> ActivityType {
        for entry in &self.entries {
            if entry.start_hour <= entry.end_hour {
                if hour >= entry.start_hour && hour < entry.end_hour {
                    return entry.activity.clone();
                }
            } else {
                // Wraps around midnight (e.g. 22 to 6)
                if hour >= entry.start_hour || hour < entry.end_hour {
                    return entry.activity.clone();
                }
            }
        }
        ActivityType::Relax // Default
    }
}
