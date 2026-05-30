//! Building placement system
//!
//! Handles placing buildings on the grid with validation and undo support.

use crate::data::building::{Building, BuildingType};
use crate::data::grid::Grid;
use crate::data::types::Position;

/// Result of attempting to place a building
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlacementResult {
    /// Building was placed successfully
    Success(u32), // Returns the building ID
    /// Position is outside the grid bounds
    OutOfBounds,
    /// Area overlaps with existing building or unwalkable terrain
    AreaOccupied,
}

/// Manages building placement, storage, and undo operations
#[derive(Clone, Debug, Default)]
pub struct BuildingSystem {
    /// All placed buildings
    buildings: Vec<Building>,
    /// Next building ID to assign
    next_id: u32,
    /// Stack of building IDs for undo (most recent last)
    undo_stack: Vec<u32>,
}

impl BuildingSystem {
    pub fn new() -> Self {
        Self {
            buildings: Vec::new(),
            next_id: 1,
            undo_stack: Vec::new(),
        }
    }

    /// Attempt to place a building at the given position
    pub fn try_place_building(
        &mut self,
        grid: &mut Grid,
        building_type: BuildingType,
        position: Position,
    ) -> PlacementResult {
        let (width, height) = building_type.size();

        // Validate: check bounds
        if position.x < 0 || position.y < 0 {
            return PlacementResult::OutOfBounds;
        }

        if position.x as u32 + width > grid.width as u32
            || position.y as u32 + height > grid.height as u32
        {
            return PlacementResult::OutOfBounds;
        }

        // Validate: check if area is free
        if !grid.is_area_free_for_building(position, width, height) {
            return PlacementResult::AreaOccupied;
        }

        // Place the building
        let building_id = self.next_id;
        self.next_id += 1;

        // Occupy grid cells
        if !grid.occupy_area(position, width, height, building_id) {
            return PlacementResult::AreaOccupied;
        }

        // Create and store building
        let building = Building::new(building_id, building_type, position);
        self.buildings.push(building);
        self.undo_stack.push(building_id);

        PlacementResult::Success(building_id)
    }

    /// Undo the last placement
    pub fn undo_last_placement(&mut self, grid: &mut Grid) -> Option<u32> {
        if let Some(building_id) = self.undo_stack.pop() {
            if self.remove_building_internal(grid, building_id) {
                return Some(building_id);
            }
        }
        None
    }

    /// Internal remove that doesn't modify undo stack
    fn remove_building_internal(&mut self, grid: &mut Grid, building_id: u32) -> bool {
        if let Some(idx) = self.buildings.iter().position(|b| b.id == building_id) {
            grid.clear_building(building_id);
            self.buildings.remove(idx);
            true
        } else {
            false
        }
    }

    /// Get a building by ID
    pub fn get_building(&self, building_id: u32) -> Option<&Building> {
        self.buildings.iter().find(|b| b.id == building_id)
    }

    pub fn last_placed_building(&self) -> Option<&Building> {
        self.undo_stack
            .last()
            .and_then(|building_id| self.get_building(*building_id))
    }

    /// Get the building at a specific grid position
    pub fn get_building_at(&self, pos: Position) -> Option<&Building> {
        self.buildings.iter().find(|b| b.occupies(pos))
    }

    /// Get all buildings
    pub fn buildings(&self) -> &[Building] {
        &self.buildings
    }

    /// Get the number of placed buildings
    pub fn building_count(&self) -> usize {
        self.buildings.len()
    }

    /// Check if a placement would be valid (for preview)
    #[cfg(test)]
    pub fn can_place_building(
        &self,
        grid: &Grid,
        building_type: BuildingType,
        position: Position,
    ) -> bool {
        let (width, height) = building_type.size();

        // Check bounds
        if position.x < 0 || position.y < 0 {
            return false;
        }

        if position.x as u32 + width > grid.width as u32
            || position.y as u32 + height > grid.height as u32
        {
            return false;
        }

        // Check if area is free
        grid.is_area_free_for_building(position, width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::grid::CellType;

    fn create_test_grid() -> Grid {
        let mut grid = Grid::new(20, 20);
        // Make all cells Floor (walkable)
        for y in 0..20 {
            for x in 0..20 {
                grid.set_cell_type(x, y, CellType::Floor);
            }
        }
        grid
    }

    #[test]
    fn test_place_building() {
        let mut grid = create_test_grid();
        let mut system = BuildingSystem::new();

        let result =
            system.try_place_building(&mut grid, BuildingType::Habitat, Position::new(5, 5));

        assert!(matches!(result, PlacementResult::Success(1)));
        assert_eq!(system.building_count(), 1);
    }

    #[test]
    fn test_placement_overlap() {
        let mut grid = create_test_grid();
        let mut system = BuildingSystem::new();

        // Place first building
        system.try_place_building(&mut grid, BuildingType::Habitat, Position::new(5, 5));

        // Try to place overlapping building
        let result =
            system.try_place_building(&mut grid, BuildingType::Storage, Position::new(6, 6));

        assert_eq!(result, PlacementResult::AreaOccupied);
        assert_eq!(system.building_count(), 1);
    }

    #[test]
    fn test_placement_out_of_bounds() {
        let mut grid = create_test_grid();
        let mut system = BuildingSystem::new();

        let result = system.try_place_building(
            &mut grid,
            BuildingType::Habitat,
            Position::new(19, 19), // 2x2 would be out of bounds
        );

        assert_eq!(result, PlacementResult::OutOfBounds);
    }

    #[test]
    fn test_undo_placement() {
        let mut grid = create_test_grid();
        let mut system = BuildingSystem::new();

        system.try_place_building(&mut grid, BuildingType::Habitat, Position::new(5, 5));
        assert_eq!(system.building_count(), 1);
        assert_eq!(
            system.last_placed_building().map(|b| b.building_type),
            Some(BuildingType::Habitat)
        );

        let undone_id = system.undo_last_placement(&mut grid);
        assert_eq!(undone_id, Some(1));
        assert_eq!(system.building_count(), 0);

        // Area should be free again
        assert!(system.can_place_building(&grid, BuildingType::Habitat, Position::new(5, 5)));
    }

    #[test]
    fn test_can_place_preview() {
        let mut grid = create_test_grid();
        let mut system = BuildingSystem::new();

        assert!(system.can_place_building(&grid, BuildingType::MessHall, Position::new(5, 5)));

        system.try_place_building(&mut grid, BuildingType::MessHall, Position::new(5, 5));

        assert!(!system.can_place_building(&grid, BuildingType::Habitat, Position::new(6, 5)));
    }
}
