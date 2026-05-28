use crate::data::building::{Building, BuildingType};
use crate::data::colonist::{Colonist, ColonistState};
use crate::data::grid::CellType;
use crate::data::text::truncate_text;
use crate::data::types::Position;
use crate::state::game_state::{
    adjacent_positions, average_relationship_between, shared_assignment_pin,
    shared_social_location, GameplayState,
};
use crate::systems::planning_system::{BuildingPlacementFeedback, PlanningSystem};
use crate::systems::scenario_system::ScenarioSystem;
use crate::ui::font::{draw_text, measure_text};
use crate::ui::style;
use crate::ui::{
    draw_iso_diamond, draw_iso_diamond_lines, draw_iso_prism, draw_tooltip_at, restart_button_rect,
    IsoView, SpritePose, ToolbarMode,
};
use macroquad::prelude::*;
use macroquad_toolkit::input::mouse_position_vec2;

mod building_visuals;
mod buildings;
mod colonist_visuals;
mod colonists;
mod overlay;
mod picking;
mod placement_preview;
mod social;
mod terrain;
mod terrain_visuals;

pub(crate) use building_visuals::*;
pub(crate) use colonist_visuals::*;
pub(crate) use terrain_visuals::*;
