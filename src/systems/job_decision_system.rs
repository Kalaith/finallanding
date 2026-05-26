use crate::data::colonist::{Colonist, JobPreference};

pub fn calculate_refusal_chance(colonist: &Colonist, _job_type: JobPreference) -> f32 {
    let mut chance: f32 = 0.0;

    // Mood Factor
    if colonist.mood < 20.0 {
        chance += 50.0;
    } else if colonist.mood < 40.0 {
        chance += 20.0;
    }

    // Trait Factor (Example)
    /*
    match colonist.trait_data {
        crate::data::colonist::Trait::Lazy => chance += 10.0,
        crate::data::colonist::Trait::HardWorker => chance -= 10.0,
        _ => {}
    }
    */

    // Relationship Factor (Hypothetical: Check if any "coworker" is hated)
    // This would require passing context (coworkers) which we don't strictly have here yet.
    // For MVP, if they have *any* very bad relationship (-30 or less), they are grumpy.
    for val in colonist.relationships.values() {
        if *val < -30 {
            chance += 10.0;
            break; // Don't stack hatred for now
        }
    }

    chance.clamp(0.0, 100.0)
}
