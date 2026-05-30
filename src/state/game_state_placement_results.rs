use super::*;

pub(crate) fn placement_result_reason(result: &PlacementResult) -> &'static str {
    match result {
        PlacementResult::Success(_) => "Placement succeeded.",
        PlacementResult::OutOfBounds => "Footprint leaves the map.",
        PlacementResult::AreaOccupied => "Footprint overlaps blocked or occupied space.",
    }
}
