use super::*;

impl GameplayState {
    pub(super) fn scenario_restart_transition(
        &self,
        input: &InputState,
    ) -> Option<StateTransition> {
        if !self.data.scenario.is_finished() {
            return None;
        }

        let restart_rect = restart_button_rect(screen_width(), screen_height());
        let clicked_restart = input.left_pressed_rect(restart_rect);

        if clicked_restart || is_key_pressed(KeyCode::R) || input.enter_pressed {
            Some(StateTransition::ToGameplay(GameplayState::new()))
        } else {
            None
        }
    }

    pub(super) fn advance_time(&mut self) -> u64 {
        let speed_multiplier = match self.data.time.speed {
            TimeSpeed::Paused => 0.0,
            TimeSpeed::Normal => 1.0,
            TimeSpeed::Fast => 2.0,
            TimeSpeed::SuperFast => 4.0,
        };

        if speed_multiplier == 0.0 {
            self.time_accumulator = 0.0;
            return 0;
        }

        self.time_accumulator += get_frame_time() * speed_multiplier;
        let ticks_to_advance = (self.time_accumulator / SECONDS_PER_GAME_TICK).floor() as u64;

        if ticks_to_advance == 0 {
            return 0;
        }

        self.time_accumulator -= ticks_to_advance as f32 * SECONDS_PER_GAME_TICK;
        self.prev_tick = self.data.tick;
        self.data.tick += ticks_to_advance;

        self.time_events.clear();
        TimeSystem::collect_events(self.prev_tick, self.data.tick, &mut self.time_events);
        ticks_to_advance
    }

    pub(super) fn process_time_events(&mut self) {
        let events = self.time_events.events.clone();

        for event in events {
            match event {
                crate::systems::time_events::TimeEvent::NewDay { day } => {
                    ProximitySystem::check_sleeping_proximity(&mut self.data);
                    SummarySystem::summarize_previous_day(&mut self.data, day);
                    ResourceSystem::handle_new_day(&mut self.data);
                }
                crate::systems::time_events::TimeEvent::DawnBreak => {
                    self.data.push_log(
                        LogCategory::Time,
                        "Dawn breaks",
                        "Colonists begin shifting toward the day's work.",
                    );
                }
                crate::systems::time_events::TimeEvent::Dusk => {
                    self.data.push_log(
                        LogCategory::Time,
                        "Dusk falls",
                        "The settlement starts moving toward meals and recovery.",
                    );
                }
                crate::systems::time_events::TimeEvent::HourChanged { hour: _ } => {
                    IncidentSystem::process_hourly_incidents(&mut self.data);
                    WorkSystem::process_hourly_work(&mut self.data);
                    SocialSystem::check_working_together(&mut self.data);
                    SocialSystem::check_eating_together(&mut self.data);
                }
            }
        }
    }

    pub(super) fn average_mood(&self) -> f32 {
        if self.data.colonists.is_empty() {
            return 0.0;
        }

        self.data.colonists.iter().map(|c| c.mood).sum::<f32>() / self.data.colonists.len() as f32
    }
}
