use crate::data::colonist::{Colonist, JobPreference, Trait};
use crate::data::game_state::GameState;
use crate::data::types::Position;

pub fn spawn_initial_colonists(state: &mut GameState) {
    let colonists_data = vec![
        ("Alice", Trait::HardWorker, JobPreference::Builder),
        ("Bob", Trait::Lazy, JobPreference::Cook),
        ("Charlie", Trait::FastWalker, JobPreference::Explorer),
        ("Diana", Trait::Gourmet, JobPreference::Hauler),
        ("Evan", Trait::HardWorker, JobPreference::Explorer),
        ("Fiona", Trait::Lazy, JobPreference::Builder),
    ];

    for (i, (name, trait_data, job_pref)) in colonists_data.iter().enumerate() {
        let id = i as u32;
        // Simple spawn layout for now, in a row
        let position = Position::new(5 + i as i32, 5);

        let mut colonist = Colonist::new(id, name.to_string(), position, *trait_data, *job_pref);
        colonist.schedule = crate::data::schedule::Schedule::new_randomized();

        state.colonists.push(colonist);
    }
}
