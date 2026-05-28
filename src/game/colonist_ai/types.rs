use crate::data::building::BuildingType;
use crate::data::colonist::ActivityLocation;
use crate::data::event_log::LogCategory;
use crate::data::types::Position;

pub(super) type PendingLog = (LogCategory, String, String);
pub(super) type SocialLocation = (u32, ActivityLocation);
pub(super) type BuildingSnapshot = (u32, BuildingType, Position, (u32, u32));

#[derive(Clone, Copy, Debug)]
pub(super) struct BuildingTarget {
    pub(super) building_id: u32,
    pub(super) building_type: BuildingType,
    pub(super) entrance: Position,
}
