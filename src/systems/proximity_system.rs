use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, ColonistState};
use crate::data::game_state::GameState;
use crate::systems::social_system::SocialSystem;
use std::collections::HashMap;

pub struct ProximitySystem;

impl ProximitySystem {
    /// Checks sleeping proximity and updates relationships.
    /// Should be called once per night or at the start of a new day.
    pub fn check_sleeping_proximity(state: &mut GameState) {
        if state.colonists.len() < 2 {
            return;
        }

        let mut sleeping_locations: HashMap<u32, Vec<u32>> = HashMap::new();

        for colonist in &state.colonists {
            if colonist.state != ColonistState::Sleeping {
                continue;
            }

            if let ActivityLocation::Building {
                building_id,
                building_type,
            } = colonist.activity_location
            {
                if building_type == BuildingType::Habitat {
                    sleeping_locations
                        .entry(building_id)
                        .or_default()
                        .push(colonist.id);
                }
            }
        }

        SocialSystem::apply_shared_habitat_relationships(state, &sleeping_locations);
    }
}
