use macroquad_toolkit::pathfinding::{find_path_with, Heuristic, Pos};
use serde::{Deserialize, Serialize};

use super::types::Position;

// Grid configuration constants
pub const GRID_WIDTH: usize = 26;
pub const GRID_HEIGHT: usize = 24;
#[cfg(test)]
const CELL_SIZE: f32 = 32.0;

/// Represents the type of terrain in a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CellType {
    #[default]
    Empty,
    Floor,
    Wall,
}

/// A single cell in the grid.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub cell_type: CellType,
    /// If Some, contains the ID of the building occupying this cell
    pub building_id: Option<u32>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            cell_type: CellType::Empty,
            building_id: None,
        }
    }
}

impl Cell {
    pub fn is_walkable(&self) -> bool {
        // Cell is walkable if terrain allows AND no building occupies it
        matches!(self.cell_type, CellType::Floor | CellType::Empty) && self.building_id.is_none()
    }
}

/// The main grid structure for the game world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(GRID_WIDTH, GRID_HEIGHT)
    }
}

impl Grid {
    /// Creates a new grid with the specified dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); width * height],
        }
    }

    /// Converts a 2D grid position to a 1D index.
    fn get_index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    /// Returns true if the given grid coordinates are within bounds.
    pub fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    /// Gets a reference to the cell at the given grid coordinates.
    pub fn get_cell(&self, x: i32, y: i32) -> Option<&Cell> {
        if self.is_in_bounds(x, y) {
            self.get_index(x as usize, y as usize)
                .map(|idx| &self.cells[idx])
        } else {
            None
        }
    }

    /// Gets a mutable reference to the cell at the given grid coordinates.
    pub fn get_cell_mut(&mut self, x: i32, y: i32) -> Option<&mut Cell> {
        if self.is_in_bounds(x, y) {
            self.get_index(x as usize, y as usize)
                .map(|idx| &mut self.cells[idx])
        } else {
            None
        }
    }

    /// Sets the cell type at the given grid coordinates.
    #[cfg(test)]
    pub fn set_cell_type(&mut self, x: i32, y: i32, cell_type: CellType) {
        if let Some(cell) = self.get_cell_mut(x, y) {
            cell.cell_type = cell_type;
        }
    }

    // ----- Coordinate Conversion -----

    /// Converts world (pixel) coordinates to grid coordinates.
    #[cfg(test)]
    pub fn world_to_grid(world_x: f32, world_y: f32) -> Position {
        Position::new(
            (world_x / CELL_SIZE).floor() as i32,
            (world_y / CELL_SIZE).floor() as i32,
        )
    }

    /// Converts grid coordinates to world (pixel) coordinates (top-left of cell).
    #[cfg(test)]
    pub fn grid_to_world(grid_x: i32, grid_y: i32) -> (f32, f32) {
        (grid_x as f32 * CELL_SIZE, grid_y as f32 * CELL_SIZE)
    }

    // ----- A* Pathfinding -----

    /// Finds a path from start to goal using the A* algorithm.
    /// Returns None if no path exists.
    pub fn find_path(&self, start: Position, goal: Position) -> Option<Vec<Position>> {
        find_path_with(
            Pos::new(start.x, start.y),
            Pos::new(goal.x, goal.y),
            self.width,
            self.height,
            |pos| {
                self.get_cell(pos.x, pos.y)
                    .is_some_and(|cell| cell.is_walkable())
            },
            |_| 1.0,
            Heuristic::Manhattan,
            false,
        )
        .map(|path| {
            path.waypoints
                .into_iter()
                .map(|pos| Position::new(pos.x, pos.y))
                .collect()
        })
    }

    // ----- Building Placement -----

    /// Check if an area is free for building placement
    /// (all cells must be Floor/Empty, in bounds, and not occupied by another building)
    pub fn is_area_free_for_building(&self, top_left: Position, width: u32, height: u32) -> bool {
        for dx in 0..width as i32 {
            for dy in 0..height as i32 {
                let pos = Position::new(top_left.x + dx, top_left.y + dy);
                match self.get_cell(pos.x, pos.y) {
                    Some(cell) if cell.is_walkable() && cell.building_id.is_none() => continue,
                    _ => return false,
                }
            }
        }
        true
    }

    /// Occupy an area with a building ID
    pub fn occupy_area(
        &mut self,
        top_left: Position,
        width: u32,
        height: u32,
        building_id: u32,
    ) -> bool {
        if !self.is_area_free_for_building(top_left, width, height) {
            return false;
        }

        for dx in 0..width as i32 {
            for dy in 0..height as i32 {
                let pos = Position::new(top_left.x + dx, top_left.y + dy);
                if let Some(cell) = self.get_cell_mut(pos.x, pos.y) {
                    cell.building_id = Some(building_id);
                }
            }
        }
        true
    }

    /// Clear all cells occupied by a specific building ID
    pub fn clear_building(&mut self, building_id: u32) {
        for cell in self.cells.iter_mut() {
            if cell.building_id == Some(building_id) {
                cell.building_id = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_conversion() {
        let pos = Grid::world_to_grid(50.0, 70.0);
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 2);

        let (wx, wy) = Grid::grid_to_world(1, 2);
        assert_eq!(wx, 32.0);
        assert_eq!(wy, 64.0);
    }

    #[test]
    fn test_bounds_checking() {
        let grid = Grid::new(10, 10);
        assert!(grid.is_in_bounds(0, 0));
        assert!(grid.is_in_bounds(9, 9));
        assert!(!grid.is_in_bounds(-1, 0));
        assert!(!grid.is_in_bounds(10, 10));
    }

    #[test]
    fn test_pathfinding_simple() {
        let mut grid = Grid::new(5, 5);
        // Make all cells walkable
        for y in 0..5 {
            for x in 0..5 {
                grid.set_cell_type(x, y, CellType::Floor);
            }
        }

        let path = grid.find_path(Position::new(0, 0), Position::new(4, 4));
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&Position::new(0, 0)));
        assert_eq!(path.last(), Some(&Position::new(4, 4)));
    }

    #[test]
    fn test_pathfinding_blocked() {
        let mut grid = Grid::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                grid.set_cell_type(x, y, CellType::Floor);
            }
        }
        // Block the middle row
        grid.set_cell_type(0, 1, CellType::Wall);
        grid.set_cell_type(1, 1, CellType::Wall);
        grid.set_cell_type(2, 1, CellType::Wall);

        let path = grid.find_path(Position::new(1, 0), Position::new(1, 2));
        assert!(path.is_none());
    }
}
