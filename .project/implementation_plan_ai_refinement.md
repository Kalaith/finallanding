# AI Refinement Implementation Plan

## Goal
Reduce "sheep-like" behavior where colonists perform tasks in perfect unison. Introduce individual variance in schedules and target selection.

## User Review Required
> [!NOTE]
> This change introduces randomness to colonization. Some colonists might stay up later or wake up earlier than others. This is intentional to break the "synchronous robot" feel.

## Proposed Changes

### Schedule System
#### [MODIFY] [schedule.rs](file:///h:/RustGames/finallanding/src/data/schedule.rs)
- Remove `Default` impl that returns a hardcoded vector.
- Create `Schedule::new_randomized()`:
    - Base schedule similar to current default.
    - Apply random offsets (+/- 1-2 hours) to start/end times for each colonist.
    - Shift lunch/dinner times slightly.

#### [MODIFY] [colonist_spawner.rs](file:///h:/RustGames/finallanding/src/game/colonist_spawner.rs)
- Use `Schedule::new_randomized()` when spawning new colonists instead of default.

### AI Decision Logic
#### [MODIFY] [colonist_ai.rs](file:///h:/RustGames/finallanding/src/game/colonist_ai.rs)
- **Target Selection**:
    - Instead of picking the *first* building found, find *all* valid buildings.
    - Filter out buildings occupied by "enemies" (relationship < -20) if possible.
    - Pick a random building from the remaining list (weighted by distance?).
- **Wandering**:
    - Increase wander chance if the "best" building is crowded.

### Social Friction (Optional/Stretch)
#### [MODIFY] [proximity_system.rs](file:///h:/RustGames/finallanding/src/systems/proximity_system.rs)
- Add logic to "bump" colonists if they try to occupy the same cell, maybe causing a small mood drop or pause.

## Verification Plan

### Automated/Logged Verification
- **Run Simulation**:
    - Start game with 5+ colonists.
    - Log their activities at 07:00, 08:00, 12:00.
    - **Success Criteria**: Logs should show different states (e.g., at 07:00, 3 are Working, 1 is Sleeping, 1 is Eating), rather than 5/5 doing the same thing.

### Manual Verification
- Observe visual movement. Colonists should not move in a single "conga line" to the same building.
