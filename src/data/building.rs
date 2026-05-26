use crate::data::types::Position;
use serde::{Deserialize, Serialize};

/// The 5 building types for the MVP
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    Habitat,         // Sleeping, recovery
    MessHall,        // Eating, social
    Workshop,        // Building/crafting
    Storage,         // Resource buffer
    ExplorationGate, // Sends colonists out
}

impl BuildingType {
    /// Returns the size (width, height) in grid cells for this building type
    pub fn size(&self) -> (u32, u32) {
        match self {
            BuildingType::Habitat => (2, 2),
            BuildingType::MessHall => (3, 2),
            BuildingType::Workshop => (2, 3),
            BuildingType::Storage => (2, 2),
            BuildingType::ExplorationGate => (2, 2),
        }
    }

    /// Returns the display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            BuildingType::Habitat => "Habitat",
            BuildingType::MessHall => "Mess Hall",
            BuildingType::Workshop => "Workshop",
            BuildingType::Storage => "Storage",
            BuildingType::ExplorationGate => "Exploration Gate",
        }
    }

    pub fn salvage_cost(&self) -> i32 {
        match self {
            BuildingType::Habitat => 8,
            BuildingType::MessHall => 12,
            BuildingType::Workshop => 10,
            BuildingType::Storage => 6,
            BuildingType::ExplorationGate => 14,
        }
    }

    /// Returns the color for rendering (RGBA as u32)
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            BuildingType::Habitat => (100, 149, 237), // Cornflower blue
            BuildingType::MessHall => (255, 165, 0),  // Orange
            BuildingType::Workshop => (139, 69, 19),  // Saddle brown
            BuildingType::Storage => (128, 128, 128), // Gray
            BuildingType::ExplorationGate => (147, 112, 219), // Medium purple
        }
    }

    /// All building types for iteration
    pub fn all() -> &'static [BuildingType] {
        &[
            BuildingType::Habitat,
            BuildingType::MessHall,
            BuildingType::Workshop,
            BuildingType::Storage,
            BuildingType::ExplorationGate,
        ]
    }
}

/// A placed building in the world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub id: u32,
    pub building_type: BuildingType,
    pub position: Position, // Top-left corner of the building
}

impl Building {
    pub fn new(id: u32, building_type: BuildingType, position: Position) -> Self {
        Self {
            id,
            building_type,
            position,
        }
    }

    /// Get the size of this building
    pub fn size(&self) -> (u32, u32) {
        self.building_type.size()
    }

    /// Check if this building occupies a specific grid cell
    pub fn occupies(&self, pos: Position) -> bool {
        let (width, height) = self.size();
        pos.x >= self.position.x
            && pos.x < self.position.x + width as i32
            && pos.y >= self.position.y
            && pos.y < self.position.y + height as i32
    }

    /// Get all grid cells occupied by this building
    pub fn occupied_cells(&self) -> Vec<Position> {
        let (width, height) = self.size();
        let mut cells = Vec::new();
        for dx in 0..width as i32 {
            for dy in 0..height as i32 {
                cells.push(Position::new(self.position.x + dx, self.position.y + dy));
            }
        }
        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_sizes() {
        assert_eq!(BuildingType::Habitat.size(), (2, 2));
        assert_eq!(BuildingType::MessHall.size(), (3, 2));
        assert_eq!(BuildingType::Workshop.size(), (2, 3));
        assert_eq!(BuildingType::Storage.size(), (2, 2));
        assert_eq!(BuildingType::ExplorationGate.size(), (2, 2));
    }

    #[test]
    fn test_building_salvage_costs() {
        assert_eq!(BuildingType::Habitat.salvage_cost(), 8);
        assert_eq!(BuildingType::MessHall.salvage_cost(), 12);
        assert_eq!(BuildingType::Workshop.salvage_cost(), 10);
        assert_eq!(BuildingType::Storage.salvage_cost(), 6);
        assert_eq!(BuildingType::ExplorationGate.salvage_cost(), 14);
    }

    #[test]
    fn test_building_occupies() {
        let building = Building::new(1, BuildingType::Habitat, Position::new(5, 5));

        // Should occupy 2x2 area starting at (5,5)
        assert!(building.occupies(Position::new(5, 5)));
        assert!(building.occupies(Position::new(6, 5)));
        assert!(building.occupies(Position::new(5, 6)));
        assert!(building.occupies(Position::new(6, 6)));

        // Should not occupy outside area
        assert!(!building.occupies(Position::new(4, 5)));
        assert!(!building.occupies(Position::new(7, 5)));
        assert!(!building.occupies(Position::new(5, 7)));
    }

    #[test]
    fn test_occupied_cells() {
        let building = Building::new(1, BuildingType::Habitat, Position::new(0, 0));
        let cells = building.occupied_cells();

        assert_eq!(cells.len(), 4);
        assert!(cells.contains(&Position::new(0, 0)));
        assert!(cells.contains(&Position::new(1, 0)));
        assert!(cells.contains(&Position::new(0, 1)));
        assert!(cells.contains(&Position::new(1, 1)));
    }
}
