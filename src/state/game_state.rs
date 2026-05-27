use crate::data::building::BuildingType;
use crate::data::colonist::{Colonist, ColonistState};
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::game_state::TimeSpeed;
use crate::data::grid::{Grid, CELL_SIZE};
use crate::data::mission::MissionType;
use crate::data::priority::ColonyPriority;
use crate::data::types::Position;
use crate::game::building_system::PlacementResult;
use crate::state::{State, StateTransition};
use crate::systems::advisor_system::AdvisorSystem;
use crate::systems::incident_system::IncidentSystem;
use crate::systems::mission_system::MissionSystem;
use crate::systems::planning_system::{BuildingPlacementFeedback, PlanningSystem};
use crate::systems::proximity_system::ProximitySystem;
use crate::systems::resource_system::ResourceSystem;
use crate::systems::scenario_system::ScenarioSystem;
use crate::systems::social_system::SocialSystem;
use crate::systems::summary_system::SummarySystem;
use crate::systems::time_events::TimeEventCollector;
use crate::systems::time_system::TimeSystem;
use crate::systems::work_system::WorkSystem;
use crate::ui::{
    draw_advisor_overlay, draw_bottom_toolbar, draw_colonist_inspector, draw_debug_overlay,
    draw_side_panel, draw_top_bar, restart_button_rect, side_panel_hit_at, top_bar_priority_at,
    top_bar_speed_at, Layout, PlaceholderArt, SidePanelHit,
};
use macroquad::prelude::*;

const SECONDS_PER_GAME_TICK: f32 = 0.25;

pub struct GameplayState {
    pub data: GameState,
    hovered_cell: Option<Position>,
    /// Currently selected building type for placement (None = not in build mode)
    selected_building: Option<BuildingType>,
    /// Selected colonist for relationship inspection.
    selected_colonist_id: Option<u32>,
    /// Time event collector for processing time-based events
    time_events: TimeEventCollector,
    /// Previous tick for event detection
    prev_tick: u64,
    /// Accumulates real time before advancing the simulation by game ticks
    time_accumulator: f32,
    /// UI layout configuration
    layout: Layout,
    /// Debug overlay visible
    debug_mode: bool,
    /// Placeholder visual assets extracted from the rebuild reference.
    art: PlaceholderArt,
}

impl GameplayState {
    pub fn new() -> Self {
        let mut data = GameState::new();
        data.tick = 420; // Start at 07:00 AM (Work time)
        crate::game::colonist_spawner::spawn_initial_colonists(&mut data);
        data.push_log(
            LogCategory::System,
            "Crash survivors assembled",
            format!(
                "Starting stockpile: {} supplies, {} salvage. Objective: survive to Day {} and unlock {} technologies.",
                data.resources.supplies,
                data.resources.salvage,
                data.scenario.target_day,
                data.scenario.required_tech_unlocks
            ),
        );

        Self {
            prev_tick: data.tick,
            data,
            hovered_cell: None,
            selected_building: None,
            selected_colonist_id: None,
            time_events: TimeEventCollector::new(),
            time_accumulator: 0.0,
            layout: Layout::default(),
            debug_mode: false,
            art: PlaceholderArt::new(),
        }
    }

    /// Handle building selection UI (keyboard)
    fn update_building_selection(&mut self) {
        // Number keys select buildings (Q, W, E, R, T for 5 buildings)
        if is_key_pressed(KeyCode::Q) {
            self.toggle_building(BuildingType::Habitat);
        }
        if is_key_pressed(KeyCode::W) {
            self.toggle_building(BuildingType::MessHall);
        }
        if is_key_pressed(KeyCode::E) {
            self.toggle_building(BuildingType::Workshop);
        }
        if is_key_pressed(KeyCode::R) {
            self.toggle_building(BuildingType::Storage);
        }
        if is_key_pressed(KeyCode::T) {
            self.toggle_building(BuildingType::ExplorationGate);
        }
        if is_key_pressed(KeyCode::M) {
            self.launch_recommended_mission();
        }

        // Escape to cancel building mode
        if is_key_pressed(KeyCode::Escape) {
            self.selected_building = None;
        }

        // Z for undo
        if is_key_pressed(KeyCode::Z) {
            self.undo_last_building();
        }
    }

    fn toggle_building(&mut self, building_type: BuildingType) {
        if self.selected_building == Some(building_type) {
            self.selected_building = None;
        } else {
            self.selected_building = Some(building_type);
        }
    }

    fn undo_last_building(&mut self) {
        let refund = self
            .data
            .building_system
            .last_placed_building()
            .map(|building| {
                (
                    building.id,
                    building.building_type,
                    building.building_type.salvage_cost(),
                )
            });

        if let Some(building_id) = self
            .data
            .building_system
            .undo_last_placement(&mut self.data.grid)
        {
            if let Some((refund_id, building_type, salvage_cost)) = refund {
                if refund_id == building_id {
                    self.data.resources.refund_salvage(salvage_cost);
                    self.data.push_log(
                        LogCategory::System,
                        "Building plan undone",
                        format!(
                            "Removed {} #{} and refunded {} salvage.",
                            building_type.name(),
                            building_id,
                            salvage_cost
                        ),
                    );
                    return;
                }
            }

            self.data.push_log(
                LogCategory::System,
                "Building plan undone",
                format!(
                    "Removed building #{} from the settlement plan.",
                    building_id
                ),
            );
        }
    }

    fn launch_recommended_mission(&mut self) {
        let mission_type = MissionSystem::recommended_mission_type(&self.data);
        self.launch_mission(mission_type);
    }

    fn launch_mission(&mut self, mission_type: MissionType) {
        if let Err(error) = MissionSystem::launch_mission(&mut self.data, mission_type) {
            let definition = mission_type.definition();
            let (title, detail) = match error {
                crate::systems::mission_system::LaunchMissionError::NoExplorationGate => (
                    "No Exploration Gate",
                    format!(
                        "Build an Exploration Gate before sending {}.",
                        definition.name
                    ),
                ),
                crate::systems::mission_system::LaunchMissionError::NoAvailableColonist => (
                    "No available mission crew",
                    format!(
                        "{} needs a colonist who is not away or hurt.",
                        definition.name
                    ),
                ),
                crate::systems::mission_system::LaunchMissionError::MissionCooldown {
                    remaining_ticks,
                } => (
                    "Mission crew regrouping",
                    format!(
                        "Wait {} more minutes before launching another mission.",
                        remaining_ticks
                    ),
                ),
            };

            self.data.push_log(LogCategory::Mission, title, detail);
        }
    }

    fn set_priority(&mut self, priority: ColonyPriority) {
        if self.data.priority.active == priority {
            return;
        }

        self.data.priority.active = priority;
        self.data.push_log(
            LogCategory::System,
            format!("Priority set: {}", priority.label()),
            priority.description(),
        );
    }

    /// Handle building placement via mouse click
    fn update_building_placement(&mut self) {
        // Only allow placement in the game area (not over UI)
        let (mouse_x, mouse_y) = mouse_position();
        let game_area = self.layout.game_area();

        if mouse_x < game_area.x || mouse_x > game_area.x + game_area.w {
            return;
        }
        if mouse_y < game_area.y || mouse_y > game_area.y + game_area.h {
            return;
        }

        let Some(building_type) = self.selected_building else {
            return;
        };

        if is_mouse_button_pressed(MouseButton::Left) {
            let pos = Grid::world_to_grid(mouse_x - game_area.x, mouse_y - game_area.y);
            let feedback = PlanningSystem::building_feedback(&self.data, building_type, pos);
            if let Some(reason) = feedback.invalid_reason.as_ref() {
                self.data.push_log(
                    LogCategory::System,
                    format!("Cannot place {}", building_type.name()),
                    format!(
                        "{} {} helps {}: {}",
                        reason,
                        building_type.name(),
                        feedback.helps,
                        feedback.purpose
                    ),
                );
                return;
            }

            let result = self.data.building_system.try_place_building(
                &mut self.data.grid,
                building_type,
                pos,
            );
            let result_reason = placement_result_reason(&result);

            match result {
                PlacementResult::Success(building_id) => {
                    self.data
                        .resources
                        .spend_salvage(building_type.salvage_cost());
                    self.data.push_log(
                        LogCategory::System,
                        format!("{} placed", building_type.name()),
                        PlanningSystem::placement_log_detail(
                            &feedback,
                            building_id,
                            self.data.resources.salvage,
                        ),
                    );
                }
                PlacementResult::OutOfBounds
                | PlacementResult::AreaOccupied
                | PlacementResult::InvalidBuilding => {
                    self.data.push_log(
                        LogCategory::System,
                        format!("Cannot place {}", building_type.name()),
                        result_reason.to_string(),
                    );
                }
            }
        }
    }

    fn update_pointer_ui_input(&mut self) {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        let (mouse_x, mouse_y) = mouse_position();

        if mouse_y <= self.layout.top_bar_height {
            self.update_top_bar_click(mouse_x, mouse_y);
            return;
        }

        let panel = self.layout.side_panel();
        if mouse_x >= panel.x
            && mouse_x <= panel.x + panel.w
            && mouse_y >= panel.y
            && mouse_y <= panel.y + panel.h
        {
            self.update_side_panel_click(mouse_x, mouse_y, panel);
        }
    }

    fn update_colonist_selection(&mut self) {
        if self.selected_building.is_some() || !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        let game_area = self.layout.game_area();
        let (mouse_x, mouse_y) = mouse_position();
        if mouse_x < game_area.x
            || mouse_x > game_area.x + game_area.w
            || mouse_y < game_area.y
            || mouse_y > game_area.y + game_area.h
        {
            return;
        }

        self.selected_colonist_id = self.colonist_id_at_mouse();
    }

    fn update_top_bar_click(&mut self, mouse_x: f32, mouse_y: f32) {
        if let Some(speed) = top_bar_speed_at(mouse_x, mouse_y) {
            self.data.time.speed = speed;
            return;
        }

        if let Some(priority) = top_bar_priority_at(mouse_x, mouse_y) {
            self.set_priority(priority);
        }
    }

    fn update_side_panel_click(&mut self, mouse_x: f32, mouse_y: f32, panel: Rect) {
        match side_panel_hit_at(panel, mouse_x, mouse_y) {
            Some(SidePanelHit::Building(building_type)) => self.toggle_building(building_type),
            Some(SidePanelHit::Undo) => {
                self.undo_last_building();
            }
            Some(SidePanelHit::Mission(mission_type)) => {
                self.launch_mission(mission_type);
            }
            None => {}
        }
    }

    fn scenario_restart_transition(&self) -> Option<StateTransition> {
        if !self.data.scenario.is_finished() {
            return None;
        }

        let restart_rect = restart_button_rect(screen_width(), screen_height());
        let mouse_pos: Vec2 = mouse_position().into();
        let clicked_restart =
            is_mouse_button_pressed(MouseButton::Left) && restart_rect.contains(mouse_pos);

        if clicked_restart || is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::Enter) {
            Some(StateTransition::ToGameplay(GameplayState::new()))
        } else {
            None
        }
    }

    fn advance_time(&mut self) -> u64 {
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

    fn process_time_events(&mut self) {
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

    fn average_mood(&self) -> f32 {
        if self.data.colonists.is_empty() {
            return 0.0;
        }

        self.data.colonists.iter().map(|c| c.mood).sum::<f32>() / self.data.colonists.len() as f32
    }

    fn draw_scenario_overlay(&self) {
        if !self.data.scenario.is_finished() {
            return;
        }

        let w = 520.0;
        let h = 190.0;
        let x = (screen_width() - w) * 0.5;
        let y = (screen_height() - h) * 0.5;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.55),
        );
        draw_rectangle(x, y, w, h, Color::new(0.08, 0.08, 0.1, 0.95));
        draw_rectangle_lines(x, y, w, h, 2.0, WHITE);

        let title = self.data.scenario.outcome.label();
        let title_width = measure_text(title, None, 28, 1.0).width;
        draw_text(title, x + (w - title_width) * 0.5, y + 42.0, 28.0, WHITE);

        let line = ScenarioSystem::objective_line(&self.data);
        let line_width = measure_text(&line, None, 16, 1.0).width;
        draw_text(&line, x + (w - line_width) * 0.5, y + 82.0, 16.0, LIGHTGRAY);

        let prompt = "Scenario complete. Review the log, then restart for another plan.";
        let prompt_width = measure_text(prompt, None, 14, 1.0).width;
        draw_text(prompt, x + (w - prompt_width) * 0.5, y + 116.0, 14.0, GRAY);

        let button = restart_button_rect(screen_width(), screen_height());
        let mouse_pos: Vec2 = mouse_position().into();
        let button_color = if button.contains(mouse_pos) {
            Color::new(0.25, 0.38, 0.48, 1.0)
        } else {
            Color::new(0.16, 0.22, 0.28, 1.0)
        };
        draw_rectangle(button.x, button.y, button.w, button.h, button_color);
        draw_rectangle_lines(button.x, button.y, button.w, button.h, 1.0, WHITE);
        let button_text = "Restart Run";
        let button_width = measure_text(button_text, None, 18, 1.0).width;
        draw_text(
            button_text,
            button.x + (button.w - button_width) * 0.5,
            button.y + 25.0,
            18.0,
            WHITE,
        );
        let restart_hint = "R or Enter";
        let hint_width = measure_text(restart_hint, None, 12, 1.0).width;
        draw_text(
            restart_hint,
            x + (w - hint_width) * 0.5,
            y + 170.0,
            12.0,
            LIGHTGRAY,
        );
    }

    /// Draw buildings on the grid
    fn draw_buildings(&self) {
        let game_area = self.layout.game_area();
        for building in self.data.building_system.buildings() {
            let (wx, wy) = Grid::grid_to_world(building.position.x, building.position.y);
            let (width, height) = building.size();
            let (r, g, b) = building.building_type.color();

            let color = Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
            draw_rectangle(
                game_area.x + wx,
                game_area.y + wy,
                width as f32 * CELL_SIZE,
                height as f32 * CELL_SIZE,
                color,
            );

            // Draw border
            draw_rectangle_lines(
                game_area.x + wx,
                game_area.y + wy,
                width as f32 * CELL_SIZE,
                height as f32 * CELL_SIZE,
                2.0,
                WHITE,
            );

            // Draw building name
            let name = building.building_type.name();
            let center_x =
                game_area.x + wx + (width as f32 * CELL_SIZE) / 2.0 - (name.len() as f32 * 3.0);
            let center_y = game_area.y + wy + (height as f32 * CELL_SIZE) / 2.0 + 5.0;
            draw_text(name, center_x, center_y, 14.0, WHITE);
        }
    }

    /// Draw ghost preview of building at cursor
    fn draw_ghost_preview(&self) {
        if let Some(building_type) = self.selected_building {
            let (mouse_x, mouse_y) = mouse_position();
            let game_area = self.layout.game_area();
            if mouse_x < game_area.x
                || mouse_x > game_area.x + game_area.w
                || mouse_y < game_area.y
                || mouse_y > game_area.y + game_area.h
            {
                return;
            }

            let pos = Grid::world_to_grid(mouse_x - game_area.x, mouse_y - game_area.y);
            let (wx, wy) = Grid::grid_to_world(pos.x, pos.y);
            let (width, height) = building_type.size();
            let feedback = PlanningSystem::building_feedback(&self.data, building_type, pos);
            let can_place = feedback.can_place();

            // Green if valid, red if invalid
            let color = if can_place {
                Color::new(0.0, 1.0, 0.0, 0.4)
            } else {
                Color::new(1.0, 0.0, 0.0, 0.4)
            };

            draw_rectangle(
                game_area.x + wx,
                game_area.y + wy,
                width as f32 * CELL_SIZE,
                height as f32 * CELL_SIZE,
                color,
            );

            // Draw outline
            let outline_color = if can_place { GREEN } else { RED };
            draw_rectangle_lines(
                game_area.x + wx,
                game_area.y + wy,
                width as f32 * CELL_SIZE,
                height as f32 * CELL_SIZE,
                2.0,
                outline_color,
            );

            draw_text(
                &format!(
                    "{} {}x{} | {} salvage",
                    building_type.name(),
                    width,
                    height,
                    building_type.salvage_cost()
                ),
                game_area.x + wx,
                game_area.y + wy - 4.0,
                14.0,
                outline_color,
            );

            self.draw_placement_feedback_panel(&feedback);
        }
    }

    fn draw_placement_feedback_panel(&self, feedback: &BuildingPlacementFeedback) {
        let game_area = self.layout.game_area();
        let width = (game_area.w - 24.0).clamp(260.0, 340.0);
        let height = 124.0;
        let (mouse_x, mouse_y) = mouse_position();
        let x = (mouse_x + 18.0)
            .min(game_area.x + game_area.w - width - 8.0)
            .max(game_area.x + 8.0);
        let y = (mouse_y + 18.0)
            .min(game_area.y + game_area.h - height - 8.0)
            .max(game_area.y + 8.0);
        let status_color = if feedback.can_place() { GREEN } else { ORANGE };

        draw_rectangle(x, y, width, height, Color::new(0.035, 0.04, 0.045, 0.94));
        draw_rectangle(x, y, 4.0, height, status_color);
        draw_rectangle_lines(x, y, width, height, 1.0, Color::new(0.45, 0.5, 0.55, 0.85));

        draw_text(
            &format!(
                "{} | {}x{} | {} salvage",
                feedback.building_type.name(),
                feedback.footprint.0,
                feedback.footprint.1,
                feedback.cost
            ),
            x + 12.0,
            y + 22.0,
            14.0,
            WHITE,
        );
        draw_text(
            &format!("Helps: {}", feedback.helps),
            x + 12.0,
            y + 43.0,
            12.0,
            LIGHTGRAY,
        );
        draw_text(
            &truncate_text(feedback.purpose, 48),
            x + 12.0,
            y + 63.0,
            11.0,
            Color::new(0.75, 0.78, 0.8, 1.0),
        );

        if let Some(reason) = feedback.invalid_reason.as_ref() {
            draw_text(
                &format!("Blocked: {}", truncate_text(reason, 39)),
                x + 12.0,
                y + 88.0,
                12.0,
                ORANGE,
            );
            draw_text(
                "Move the footprint or pick another building.",
                x + 12.0,
                y + 108.0,
                11.0,
                GRAY,
            );
        } else {
            draw_text(
                &format!("Impact: {}", truncate_text(feedback.impact, 42)),
                x + 12.0,
                y + 88.0,
                12.0,
                LIGHTGRAY,
            );
            draw_text("Click to place this plan.", x + 12.0, y + 108.0, 11.0, GRAY);
        }
    }

    /// Draw the grid with offset for top bar
    fn draw_grid_with_offset(&self) {
        let game_area = self.layout.game_area();

        for y in 0..self.data.grid.height {
            for x in 0..self.data.grid.width {
                let (wx, wy) = Grid::grid_to_world(x as i32, y as i32);

                let cell = self.data.grid.get_cell(x as i32, y as i32);
                let color = match cell {
                    Some(c) => match c.cell_type {
                        crate::data::grid::CellType::Empty => Color::new(0.19, 0.17, 0.11, 1.0),
                        crate::data::grid::CellType::Floor => Color::new(0.24, 0.22, 0.15, 1.0),
                        crate::data::grid::CellType::Wall => Color::new(0.16, 0.18, 0.16, 1.0),
                    },
                    None => BLACK,
                };

                draw_rectangle(
                    game_area.x + wx,
                    game_area.y + wy,
                    CELL_SIZE,
                    CELL_SIZE,
                    color,
                );
                draw_rectangle_lines(
                    game_area.x + wx,
                    game_area.y + wy,
                    CELL_SIZE,
                    CELL_SIZE,
                    1.0,
                    Color::new(0.12, 0.13, 0.11, 0.45),
                );
            }
        }

        // Highlight hovered cell
        if let Some(pos) = self.hovered_cell {
            let (wx, wy) = Grid::grid_to_world(pos.x, pos.y);
            draw_rectangle_lines(
                game_area.x + wx,
                game_area.y + wy,
                CELL_SIZE,
                CELL_SIZE,
                2.0,
                YELLOW,
            );
        }
    }

    /// Draw colonists with offset for top bar
    fn draw_colonists_with_offset(&self, hovered_colonist_id: Option<u32>) {
        let game_area = self.layout.game_area();

        for colonist in &self.data.colonists {
            if colonist.is_on_mission() {
                continue;
            }

            let x = game_area.x + colonist.visual_x;
            let y = game_area.y + colonist.visual_y;
            let size = 24.0;

            // Colonist color based on state
            let color = match colonist.state {
                ColonistState::Idle => SKYBLUE,
                ColonistState::Moving { .. } => GREEN,
                ColonistState::Working => ORANGE,
                ColonistState::Eating => YELLOW,
                ColonistState::Sleeping => Color::new(0.5, 0.5, 0.8, 1.0),
                ColonistState::OnMission { .. } => PURPLE,
            };

            let center_x = x + 16.0;
            let center_y = y + 16.0;
            draw_ellipse(
                center_x,
                center_y + 12.0,
                12.0,
                4.0,
                0.0,
                Color::new(0.0, 0.0, 0.0, 0.25),
            );
            if let Some(sprite) = self.art.colonist_sprite(colonist.id) {
                draw_texture_ex(
                    sprite,
                    center_x - 18.0,
                    center_y - 37.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(36.0, 70.0)),
                        ..Default::default()
                    },
                );
            } else {
                draw_rectangle(center_x - 8.0, center_y + 2.0, 16.0, 15.0, color);
                draw_rectangle_lines(center_x - 8.0, center_y + 2.0, 16.0, 15.0, 1.0, WHITE);
                draw_circle(
                    center_x,
                    center_y - 5.0,
                    8.0,
                    Color::new(0.78, 0.68, 0.56, 1.0),
                );
                draw_circle_lines(center_x, center_y - 5.0, 8.0, 1.0, WHITE);
                draw_rectangle(center_x - 5.0, center_y - 10.0, 10.0, 3.0, color);
                draw_line(
                    center_x - 5.0,
                    center_y + 17.0,
                    center_x - 9.0,
                    center_y + 24.0,
                    2.0,
                    LIGHTGRAY,
                );
                draw_line(
                    center_x + 5.0,
                    center_y + 17.0,
                    center_x + 9.0,
                    center_y + 24.0,
                    2.0,
                    LIGHTGRAY,
                );
            }
            draw_circle(
                center_x + 8.0,
                center_y + 5.0,
                3.0,
                job_color(colonist.job_preference),
            );
            if Some(colonist.id) == hovered_colonist_id
                || Some(colonist.id) == self.selected_colonist_id
            {
                draw_circle_lines(
                    center_x,
                    center_y,
                    size / 2.0 + 4.0,
                    2.0,
                    Color::new(1.0, 1.0, 1.0, 0.8),
                );
            }

            // Name label
            let name_width = measure_text(&colonist.name, None, 12, 1.0).width;
            draw_text(
                &colonist.name,
                center_x - name_width / 2.0,
                y + 40.0,
                12.0,
                WHITE,
            );
        }
    }

    fn colonist_id_at_mouse(&self) -> Option<u32> {
        let game_area = self.layout.game_area();
        let (mouse_x, mouse_y) = mouse_position();
        if mouse_x < game_area.x
            || mouse_x > game_area.x + game_area.w
            || mouse_y < game_area.y
            || mouse_y > game_area.y + game_area.h
        {
            return None;
        }

        self.data
            .colonists
            .iter()
            .filter(|colonist| !colonist.is_on_mission())
            .filter_map(|colonist| {
                let center_x = game_area.x + colonist.visual_x + 16.0;
                let center_y = game_area.y + colonist.visual_y + 16.0;
                let dx = mouse_x - center_x;
                let dy = mouse_y - center_y;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq <= 18.0 * 18.0 {
                    Some((colonist.id, distance_sq))
                } else {
                    None
                }
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(id, _)| id)
    }

    fn colonist_by_id(&self, id: u32) -> Option<&Colonist> {
        self.data
            .colonists
            .iter()
            .find(|colonist| colonist.id == id)
    }

    fn inspected_colonist(&self, hovered_colonist_id: Option<u32>) -> Option<&Colonist> {
        hovered_colonist_id
            .and_then(|id| self.colonist_by_id(id))
            .or_else(|| {
                self.selected_colonist_id
                    .and_then(|id| self.colonist_by_id(id))
            })
    }
}

fn placement_result_reason(result: &PlacementResult) -> &'static str {
    match result {
        PlacementResult::Success(_) => "Placement succeeded.",
        PlacementResult::OutOfBounds => "Footprint leaves the map.",
        PlacementResult::AreaOccupied => "Footprint overlaps blocked or occupied space.",
        PlacementResult::InvalidBuilding => "Building configuration is invalid.",
    }
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let mut truncated = text
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn job_color(job_preference: crate::data::colonist::JobPreference) -> Color {
    match job_preference {
        crate::data::colonist::JobPreference::Explorer => PURPLE,
        crate::data::colonist::JobPreference::Builder => YELLOW,
        crate::data::colonist::JobPreference::Cook => GREEN,
        crate::data::colonist::JobPreference::Hauler => GRAY,
        crate::data::colonist::JobPreference::None => WHITE,
    }
}

impl State for GameplayState {
    fn update(&mut self) -> StateTransition {
        // Debug toggle
        if is_key_pressed(KeyCode::F3) {
            self.debug_mode = !self.debug_mode;
        }

        if let Some(transition) = self.scenario_restart_transition() {
            return transition;
        }

        // Time speed and priority controls (keyboard)
        if is_key_pressed(KeyCode::Space) {
            self.data.time.speed = if self.data.time.speed == TimeSpeed::Paused {
                TimeSpeed::Normal
            } else {
                TimeSpeed::Paused
            };
        }
        if is_key_pressed(KeyCode::Key1) {
            self.set_priority(ColonyPriority::Recovery);
        }
        if is_key_pressed(KeyCode::Key2) {
            self.set_priority(ColonyPriority::Stockpile);
        }
        if is_key_pressed(KeyCode::Key3) {
            self.set_priority(ColonyPriority::Survey);
        }

        self.update_pointer_ui_input();
        self.update_colonist_selection();

        let elapsed_ticks = self.advance_time();
        if elapsed_ticks > 0 {
            MissionSystem::process_completed_missions(&mut self.data);
            MissionSystem::recover_injured_colonists(&mut self.data);
            self.process_time_events();
            crate::game::colonist_ai::update_colonists(&mut self.data, elapsed_ticks);
            ScenarioSystem::evaluate(&mut self.data);
        } else {
            crate::game::colonist_ai::update_colonists(&mut self.data, 0);
        }

        // Update hovered cell based on mouse position (account for UI offset)
        let (mouse_x, mouse_y) = mouse_position();
        let game_area = self.layout.game_area();
        let grid_pos = Grid::world_to_grid(mouse_x - game_area.x, mouse_y - game_area.y);
        if self.data.grid.is_in_bounds(grid_pos.x, grid_pos.y) {
            self.hovered_cell = Some(grid_pos);
        } else {
            self.hovered_cell = None;
        }

        // Building system updates (keyboard)
        self.update_building_selection();
        self.update_building_placement();

        StateTransition::None
    }

    fn draw(&self) {
        let hovered_colonist_id = self.colonist_id_at_mouse();

        // Draw game area (grid, buildings, colonists)
        self.draw_grid_with_offset();
        self.draw_buildings();
        self.draw_ghost_preview();
        self.draw_colonists_with_offset(hovered_colonist_id);
        let advisor_plan = AdvisorSystem::plan(&self.data);
        let objective_line = ScenarioSystem::objective_line(&self.data);
        draw_advisor_overlay(&self.layout, &objective_line, &advisor_plan);
        draw_colonist_inspector(
            &self.layout,
            self.inspected_colonist(hovered_colonist_id),
            &self.data.colonists,
            self.data.tick,
        );

        // Draw UI components (on top)
        let _ = draw_top_bar(
            &self.layout,
            self.data.tick,
            self.data.time.speed,
            self.data.colonists.len(),
            self.average_mood(),
            &self.data.resources,
            self.data.priority.active,
        );

        let colony_summary = SummarySystem::colony_pressure_summary(&self.data);
        let mission_plans = MissionSystem::mission_plans(&self.data);
        let _panel_result = draw_side_panel(
            &self.layout,
            self.selected_building,
            self.data.building_system.building_count(),
            self.data.colonists.len(),
            &self.data.resources,
            ResourceSystem::storage_capacity(&self.data),
            ResourceSystem::daily_supply_need(&self.data),
            &objective_line,
            self.data.scenario.outcome,
            self.data.missions.active_count(),
            &mission_plans,
            &self.data.technology,
            &colony_summary,
            &self.data.event_log,
        );
        draw_bottom_toolbar(&self.layout, self.selected_building);

        // Debug overlay
        if self.debug_mode {
            draw_debug_overlay(
                self.data.tick,
                &self.data.colonists,
                self.hovered_cell,
                self.data.building_system.building_count(),
                &self.data.resources,
                ResourceSystem::storage_capacity(&self.data),
                ResourceSystem::daily_supply_need(&self.data),
                &ScenarioSystem::objective_line(&self.data),
                self.data.scenario.outcome,
                self.data.missions.active_count(),
                &self.data.technology,
                self.data.priority.active,
            );
        }

        self.draw_scenario_overlay();
    }
}
