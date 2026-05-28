# The Final Landing

The Final Landing is a playable colony relationship manager about guiding a small crashed settlement by shaping where people live, work, recover, and connect.

You influence the colony through buildings, priorities, missions, role assignments, and space planning. Colonists follow daily routines, build relationships through proximity and shared work, and show pressure through visible tension/support signals.

## Current Status

Playable alpha loop:

- Isometric crash-site map with building placement and undo.
- Bottom toolbar modes for Build, Rooms, Objects, Colony, Research, Assign, and Log.
- Colony priorities for recovery, stockpiling, and surveying.
- Survivor routines for work, meals, sleep, movement, mission absence, and recovery.
- Relationship pressure driven by shared rooms, work locations, proximity, priorities, and directives.
- Assign mode for role cycling, room/work pins, pair/separate directives, roster filters, sorting, and batch copy controls.
- Research missions that return supplies, salvage, and technology progress.
- Daily social history, searchable Log mode, report drilldowns, and Markdown export.
- Day 7 victory/failure scenario checks backed by headless playthrough tests.

## Quick Start

Requirements:

- Rust stable toolchain.
- The sibling `macroquad-toolkit` crate at `..\macroquad-toolkit`, as referenced by `Cargo.toml`.

Run the native game:

```powershell
cargo run
```

Run tests:

```powershell
cargo test
```

Build only:

```powershell
cargo build
```

## How To Play

Goal: keep the settlement alive after the crash, stabilize food and shelter, recover field technology, and reach the Day 7 victory check as a functioning community.

Core loop:

1. Place essential rooms and work objects.
2. Set the colony priority to match the current pressure.
3. Launch missions from Research once the Exploration Gate exists.
4. Use Assign mode to separate tense colonists or protect supportive pairs.
5. Watch the daily reports and adjust the plan before pressure becomes failure.

The settlement can limp or fail if it has no food plan, no habitats, or no mission/technology progress.

## Controls

- Mouse: use UI buttons, select colonists, place buildings, and pin compatible rooms/workspaces in Assign mode.
- Bottom toolbar: switch between Build, Rooms, Objects, Colony, Research, Assign, and Log.
- `Q`, `W`, `E`, `R`, `T`: select Habitat, Mess Hall, Workshop, Storage, or Exploration Gate.
- `1`, `2`, `3`: set Recovery, Stockpile, or Survey priority.
- `Space`: pause or resume time.
- Top bar speed buttons: pause, normal, fast, or super fast.
- `Z`: undo last placement.
- `Esc`: cancel the current building/search interaction.
- `M`: launch the recommended mission when possible.
- `F3`: toggle debug overlay.
- After victory/failure, `Enter`, `R`, or the restart button starts another run.

Assign mode:

- Click a survivor card to cycle their work role.
- Click a compatible map building to pin or clear a selected survivor's recovery/work space.
- Right-click a room or work building to filter the roster to survivors pinned there.
- Use relationship filters, role filters, sorting, and page controls to inspect pressure.
- Use `P-H`, `P-W`, `ALL-H`, and `ALL-W` to copy home/work pins across visible or full compatible rosters.
- Use pair/separate directives to encourage supportive pairs or keep tense colonists apart.

Log mode:

- Search, filter, and page daily social reports.
- Click a report row for a recommendation drilldown.
- Export the social archive to `docs\exports\social_archive.md`.

## Major Systems

- **Buildings:** Habitat, Mess Hall, Workshop, Storage, and Exploration Gate define shelter, meals, salvage, storage, and mission access.
- **Resources:** food days, salvage, meals, survey, repair, hauling, and storage pressure are tracked in the right rail.
- **Relationships:** colonists gain support or tension from shared spaces, routines, directives, and colony priorities.
- **Missions:** field missions have duration, danger, cooldown, rewards, and technology chances.
- **Scenario:** the colony is evaluated around Day 7 for supplies, shelter, technology, and condition.
- **Advisor/objectives:** the left rail surfaces active risks and next priorities.

## Implementation Notes

- Runtime and rendering use `macroquad`.
- `macroquad-toolkit` is used for pathfinding and toolkit colors.
- Colonist movement uses `Grid::find_path`, not direct coordinate stepping.
- Simulation randomness is routed through deterministic `SimulationRng`, keeping domain behavior independent of `macroquad::rand`.
- UI rendering is split across focused modules:
  - `src\ui\top_bar.rs`
  - `src\ui\right_rail.rs`
  - `src\ui\toolbar_panel.rs`
  - `src\ui\toolbar_panel\assign.rs`
  - `src\ui\toolbar_panel\log.rs`
- Gameplay orchestration remains in `src\state\game_state.rs`, with pure helper logic moved to `src\state\game_state_helpers.rs`.

## Verification

Primary checks:

```powershell
cargo fmt --check
cargo test
cargo build
```

Visual smoke captures:

```powershell
.\scripts\capture_ui_smoke.ps1
```

This captures gameplay screenshots at multiple resolutions and key modes into `docs\verification\`.

Headless playthrough matrix:

```powershell
.\scripts\capture_playthrough_report.ps1
```

This writes `docs\verification\playthrough_report.md`. The current reference run reaches a Stable Victory at about 34.2 normal-speed minutes.

Focused manual pass:

- `docs\verification\manual_relationship_playtest.md`

Use it after changes to Assign mode, relationship logic, body-language rendering, Log mode, or restart flow.

## Capture Environment Variables

The native game supports deterministic capture helpers used by scripts:

- `TFL_START_GAMEPLAY=1`: skip the menu and start gameplay.
- `TFL_CAPTURE_PATH=path\to\image.png`: save a screenshot and exit.
- `TFL_CAPTURE_FRAMES=8`: choose how many frames render before capture.
- `TFL_WINDOW_WIDTH` / `TFL_WINDOW_HEIGHT`: set native capture size.
- `TFL_START_TOOLBAR_MODE=assign|log|research|...`: open a specific toolbar mode.
- `TFL_START_SELECTED_BUILDING=habitat|mess_hall|workshop|storage|exploration_gate`: seed a placement tool.
- `TFL_PREVIEW_GRID_X` / `TFL_PREVIEW_GRID_Y`: seed placement preview position.
- `TFL_START_SELECTED_COLONIST=id`: seed selected survivor.
- `TFL_SEED_ASSIGN_SPACES=1`, `TFL_SEED_SOCIAL_HISTORY=1`, `TFL_SEED_ACTIVITY_POSES=1`: seed verification fixtures.
- `TFL_PLAYTHROUGH_REPORT_PATH=path\to\report.md`: write the headless strategy matrix and exit.

## Playtest Checklist

- Start from the main menu without developer guidance.
- Place at least one Habitat, Mess Hall, Workshop, Storage, and Exploration Gate.
- Change priorities after an incident and confirm the advisor/objective state responds.
- Launch at least two mission types from Research mode.
- Use Assign mode to retask one survivor, pin a Habitat and work space, filter by a right-clicked room, and copy pins across the visible roster.
- Confirm tense/supportive survivors show visible map markers and useful inspector details.
- Open Log mode after several day summaries; search, filter, page, select a report, and export the archive.
- Reach victory or failure, then restart and confirm the new run does not leak old selection/log state.
