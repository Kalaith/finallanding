use crate::systems::time_events::{TimeEvent, TimeEventCollector};

pub struct TimeSystem;

impl TimeSystem {
    pub const TICKS_PER_DAY: u64 = 1440; // Default
    pub const TICKS_PER_HOUR: u64 = 60;

    pub fn get_time_of_day(tick: u64) -> (u32, u32, u32) {
        let day = (tick / Self::TICKS_PER_DAY) as u32 + 1; // Day starts at 1
        let tick_in_day = tick % Self::TICKS_PER_DAY;
        let hour = (tick_in_day / Self::TICKS_PER_HOUR) as u32;
        let minute = (tick_in_day % Self::TICKS_PER_HOUR) as u32;
        (day, hour, minute)
    }

    pub fn is_day(tick: u64) -> bool {
        let (_, hour, _) = Self::get_time_of_day(tick);
        hour >= 6 && hour < 20 // 6 AM to 8 PM is day
    }

    pub fn is_night(tick: u64) -> bool {
        !Self::is_day(tick)
    }

    /// Call this each frame to collect any time-based events that occurred
    /// between prev_tick and current_tick.
    pub fn collect_events(prev_tick: u64, current_tick: u64, collector: &mut TimeEventCollector) {
        if current_tick <= prev_tick {
            return;
        }

        let (mut last_day, mut last_hour, _) = Self::get_time_of_day(prev_tick);

        for tick in (prev_tick + 1)..=current_tick {
            let (day, hour, _) = Self::get_time_of_day(tick);

            if day > last_day {
                collector.push(TimeEvent::NewDay { day });
                last_day = day;
            }

            if hour != last_hour {
                collector.push(TimeEvent::HourChanged { hour });

                if hour == 6 {
                    collector.push(TimeEvent::DawnBreak);
                }

                if hour == 20 {
                    collector.push(TimeEvent::Dusk);
                }

                last_hour = hour;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_conversion() {
        assert_eq!(TimeSystem::get_time_of_day(0), (1, 0, 0));
        assert_eq!(TimeSystem::get_time_of_day(30), (1, 0, 30));
        assert_eq!(TimeSystem::get_time_of_day(60), (1, 1, 0));
        assert_eq!(TimeSystem::get_time_of_day(1439), (1, 23, 59));
        assert_eq!(TimeSystem::get_time_of_day(1440), (2, 0, 0));
    }
}
