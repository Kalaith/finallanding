use crate::data::colonist::Colonist;
use crate::data::event_log::LogCategory;
use crate::game::colonist_ai::types::{PendingLog, SocialLocation};
use crate::game::colonist_ai::SOCIAL_STRAIN_LOG_COOLDOWN_TICKS;
use std::collections::HashMap;

pub(super) fn social_score_for_building(
    colonist: &Colonist,
    building_id: u32,
    social_locations: &[SocialLocation],
) -> i32 {
    social_locations
        .iter()
        .filter(|(other_id, location)| {
            *other_id != colonist.id && location.building_id() == Some(building_id)
        })
        .map(|(other_id, _)| {
            let relationship = colonist.relationships.get(other_id).copied().unwrap_or(0);
            let directive = if colonist.preferred_partner_id == Some(*other_id) {
                80
            } else if colonist.avoided_partner_id == Some(*other_id) {
                -80
            } else {
                0
            };

            relationship + directive
        })
        .sum()
}

pub(super) fn log_social_strain(
    colonist: &mut Colonist,
    other_id: u32,
    colonist_names: &HashMap<u32, String>,
    current_tick: u64,
    pending_logs: &mut Vec<PendingLog>,
) {
    if current_tick.saturating_sub(colonist.last_social_strain_tick)
        < SOCIAL_STRAIN_LOG_COOLDOWN_TICKS
    {
        return;
    }

    colonist.last_social_strain_tick = current_tick;
    let other_name = colonist_names
        .get(&other_id)
        .cloned()
        .unwrap_or_else(|| format!("Colonist {}", other_id));

    pending_logs.push((
        LogCategory::Social,
        format!("{} avoided {}", colonist.name, other_name),
        "A strained relationship made a crowded path stressful. Mood dropped.".to_string(),
    ));
}
