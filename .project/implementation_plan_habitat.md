# Habitat Assignment Implementation Plan

## Goal
Implement logic where colonists must select a specific Habitat building to sleep in. Habitats have a capacity of 2. If no habitat is available, colonists sleep on the ground (no building).

## Proposed Changes

### Data Model
#### [MODIFY] [colonist.rs](file:///h:/RustGames/finallanding/src/data/colonist.rs)
- Add `assigned_habitat: Option<u32>` to `Colonist` struct.
- Initialize as `None` in `new()`.

### Building System
#### [MODIFY] [building_system.rs](file:///h:/RustGames/finallanding/src/game/building_system.rs)
- Add a helper function `get_building_occupant_count(building_id: u32, colonists: &[Colonist]) -> usize`.
- Or better, handle this in a `HabitatSystem` or within `ColonistAI` to avoid circular dependencies (BuildingSystem shouldn't know about Colonists directly if possible, or pass slice).

### AI Logic
#### [MODIFY] [colonist_ai.rs](file:///h:/RustGames/finallanding/src/game/colonist_ai.rs)
- **Time to Sleep**:
    - Check `colonist.assigned_habitat`.
    - If `Some(id)`:
        - Check if building exists. If not, set to `None`.
        - If exists, target it.
    - If `None`:
        - Search for a `BuildingType::Habitat` where `occupant_count < 2`.
        - If found, set `assigned_habitat = Some(id)` and target it.
        - If not found, target current position (Sleep on Ground) or a random "Camp" spot.
- **Sleep on Ground**:
    - If no habitat, `ColonistState::Sleeping` logic should just stay in place or finding a clear spot.

## Verification Plan

### Automated Verification
- Spawn 3 colonists.
- Place 1 Habitat.
- Fast forward to Night (22:00).
- **Check**:
    - Colonist 1 and 2 should list `assigned_habitat = Some(1)`.
    - Colonist 3 should have `assigned_habitat = None` and be in `Sleeping` state at a non-building position.

### Manual Verification
- Visual check: 2 colonists go into the box, 1 stays outside.
