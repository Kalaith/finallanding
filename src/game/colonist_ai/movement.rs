use crate::data::colonist::Colonist;
use crate::data::grid::Grid;
use crate::data::simulation_rng::SimulationRng;
use crate::data::types::Position;
use crate::game::colonist_ai::social::log_social_strain;
use crate::game::colonist_ai::types::PendingLog;
use std::collections::HashMap;

pub(super) fn find_wander_target(
    current: Position,
    occupied: &HashMap<Position, u32>,
    grid: &Grid,
    rng: &mut SimulationRng,
) -> Position {
    for _ in 0..10 {
        let target_x = rng.range_i32(0, grid.width as i32);
        let target_y = rng.range_i32(0, grid.height as i32);
        let target = Position::new(target_x, target_y);
        if is_step_open(target, occupied, grid) {
            return target;
        }
    }
    current
}

pub(super) fn get_next_move_position(
    colonist: &mut Colonist,
    target: Position,
    occupied: &HashMap<Position, u32>,
    grid: &Grid,
    colonist_names: &HashMap<u32, String>,
    current_tick: u64,
    pending_logs: &mut Vec<PendingLog>,
) -> Position {
    let current = colonist.position;
    let Some(next) = grid
        .find_path(current, target)
        .and_then(|path| path.into_iter().find(|step| *step != current))
    else {
        return current;
    };

    if next != current {
        if let Some(other_id) = occupied.get(&next) {
            let relationship = colonist.relationships.get(other_id).copied().unwrap_or(0);
            if relationship < -20 {
                colonist.mood = (colonist.mood - 5.0).clamp(0.0, 100.0);
                log_social_strain(
                    colonist,
                    *other_id,
                    colonist_names,
                    current_tick,
                    pending_logs,
                );
            }

            let horiz = Position::new(
                if current.x < target.x {
                    current.x + 1
                } else if current.x > target.x {
                    current.x - 1
                } else {
                    current.x
                },
                current.y,
            );
            if horiz != current && is_step_open(horiz, occupied, grid) {
                return horiz;
            }

            let vert = Position::new(
                current.x,
                if current.y < target.y {
                    current.y + 1
                } else if current.y > target.y {
                    current.y - 1
                } else {
                    current.y
                },
            );
            if vert != current && is_step_open(vert, occupied, grid) {
                return vert;
            }

            return current;
        }
    }

    next
}

pub(super) fn is_step_open(
    position: Position,
    occupied: &HashMap<Position, u32>,
    grid: &Grid,
) -> bool {
    !occupied.contains_key(&position)
        && grid
            .get_cell(position.x, position.y)
            .is_some_and(|cell| cell.is_walkable())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{JobPreference, Trait};
    use crate::data::grid::CellType;

    #[test]
    fn test_movement_uses_grid_pathfinding_around_blocked_cells() {
        let mut grid = Grid::new(4, 3);
        grid.set_cell_type(1, 0, CellType::Wall);

        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        let occupied = HashMap::new();
        let colonist_names = HashMap::new();
        let mut pending_logs = Vec::new();

        let next = get_next_move_position(
            &mut colonist,
            Position::new(2, 0),
            &occupied,
            &grid,
            &colonist_names,
            0,
            &mut pending_logs,
        );

        assert_ne!(next, Position::new(1, 0));
        assert_ne!(next, colonist.position);
        assert!(is_step_open(next, &occupied, &grid));
    }
}
