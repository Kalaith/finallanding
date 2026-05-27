# The Final Landing UI + Gameplay Rebuild Plan

## Target Reference

Source screenshot: `tfl_guide_mvp.png`

Extracted reference files:

- Visual samples: `docs/reference/tfl_guide_mvp/visual_extraction.json`
- Style samples: `docs/reference/tfl_guide_mvp/visual_style_samples.json`
- Character contact sheet: `docs/reference/tfl_guide_mvp/character_asset_contact_sheet.png`
- Character crops: `docs/reference/tfl_guide_mvp/portrait_*.png`, `docs/reference/tfl_guide_mvp/rel_*.png`, `docs/reference/tfl_guide_mvp/sprite_*.png`

Important asset note: the extracted character crops come from a flat screenshot. Treat them as reference-only unless the screenshot art is owned/licensed for direct use. Production art should be original, licensed, or regenerated from approved references.

## Extracted Visual Direction

The screenshot reads as a grim isometric colony management UI: illustrated crash-site map, dark glass panels, warm practical lighting, compact objective/resource/status panels, and portrait-driven colonist management.

### Font

The font cannot be literally extracted from the PNG. Closest practical matches:

- Primary UI/headings: `Rajdhani SemiBold` or `Rajdhani Bold`
- Alternate: `Exo 2 SemiBold`
- Body UI fallback: `Roboto Condensed`
- Treatment: all-caps headings, compact line height, light tracking, no decorative serif styling.

Recommended implementation:

- Add a licensed `.ttf`/`.otf` to `assets/fonts/`.
- Use one family for all UI text.
- Use weight, color, and case to distinguish hierarchy instead of multiple fonts.

### Color Tokens

Measured samples from the screenshot:

- Panel background: `#191C1C`, `#191D1E`, deep panel `#101315`
- Primary title text: `#EDECE8`
- Body text: `#A1A3A0`
- Muted text: `#8C8D8A`
- Section heading blue: `#92B0B6`
- Gold/day accent: `#AD873C`
- Alert red: `#AA4538`
- Research blue: `#4E718E`
- Mood green: `#598F42`
- Hunger red: `#9C362E`
- Health cyan: `#4089AB`

### UI Shape Language

- Panels are translucent charcoal rectangles with subtle borders and 4-8px radius.
- Buttons are icon-first, compact, and rectangular.
- The central map owns most of the screen; UI panels frame it without hiding the colony.
- The bottom toolbar is the main action hub.
- Colonist management is portrait-led, with mood bars and relationship chips.

## Rebuild Goals

1. Replace the current abstract grid presentation with an isometric illustrated colony surface.
2. Rebuild the UI around the screenshot layout: top time bar, left objectives/alerts/inspector, right minimap/resources/colonists, bottom action bar.
3. Keep the existing verified systems where possible: resources, priorities, relationships, missions, incidents, daily summaries, playtest verifier.
4. Make gameplay readable at a glance: what needs attention, who is struggling, what building/action matters next.
5. Preserve keyboard/mouse ergonomics while adding screenshot-style interaction affordances.

## Proposed Layout

### Top Bar

- Left: game title.
- Center-left: pause/play/speed controls.
- Center: day and clock.
- Right of center: weather/condition chip.

### Left Rail

- Objectives panel with checkbox/progress objectives.
- Alert stack for urgent colony pressure.
- Selected colonist card:
  - portrait
  - name
  - job
  - current activity
  - mood/energy/hunger/health bars
  - relationship portrait row

### Right Rail

- Minimap preview.
- Resources panel:
  - food
  - salvage
  - metal
  - plastic
  - fabric
  - fuel
- Colonist list with portraits, names, and mood icons.

### Bottom Toolbar

- Build
- Rooms
- Objects
- Colony
- Research
- Assign
- Log

The toolbar should become the primary mode switcher. Current building shortcuts can remain as accelerators.

## Gameplay Rebuild Scope

### Objectives

Replace the single objective line with active objective cards:

- Stabilize the colony.
- Build specific room/building milestones.
- Increase food supply.
- Explore the crash site.
- Gather salvage or mission materials.

Objectives should be generated from current game state and scenario requirements, not hardcoded UI text.

### Resources

Current supplies/salvage can map to the screenshot-style resources:

- Supplies becomes Food.
- Salvage remains Salvage.
- Add derived/simple stockpiles for Metal, Plastic, Fabric, Fuel only if they immediately support gameplay.

If new resources are added, keep the first pass shallow: building costs and mission rewards only.

### Rooms + Buildings

Reframe buildings as room-like colony structures:

- Habitat
- Mess Hall
- Workshop
- Storage
- Exploration Gate

The visual rebuild should show building interiors, labels above structures, and occupied/active colonists.

### Colonists

Move from circles to:

- isometric standing sprites
- portrait cards
- list-row portraits
- role/activity labels
- mood face indicators

Use extracted crops only as reference for proportions and rendering style.

### Relationships

Keep the existing relationship model but expose it like the screenshot:

- relationship portraits under selected colonist
- numeric relationship deltas
- color-coded positive/negative values
- stronger daily story logs

### Alerts

Use existing incidents/advisor lines to feed the left alert stack:

- exhausted colonist
- low food
- research available
- habitat tension
- mission crew regrouping

Alerts should be clickable later, but first pass can be visual-only.

## Technical Plan

### Sprint A: Visual Foundation

- Add a UI style module with color tokens, typography constants, panel styles, and icon sizing.
- Load a chosen font from `assets/fonts/`.
- Replace current flat panel styling with screenshot-matched panel rendering.
- Keep existing gameplay unchanged.

Done when: top bar, panels, and buttons visibly match the screenshot palette and typography.

### Sprint B: Screenshot Layout Shell

- Implement the full screen layout:
  - top bar
  - left rail
  - right rail
  - bottom toolbar
  - central playfield
- Move existing side-panel content into the new regions.
- Add hit-zone tests for bottom toolbar and restart/menu controls.

Done when: the game screen composition matches the screenshot before any isometric art rewrite.

### Sprint C: Isometric Colony Presentation

- Add an isometric camera/projection layer.
- Render terrain tiles with muted dirt/grass/wreckage colors.
- Render buildings as placed isometric structures or high-fidelity placeholders.
- Render labels above buildings and selected colonists.

Done when: placement and colonist positions render in an isometric colony scene without breaking existing placement logic.

### Sprint D: Colonist Portrait + Sprite System

- Add portrait asset slots per colonist.
- Add sprite asset slots per job/activity.
- Replace circle rendering with small figure rendering.
- Update inspector and colonist list to use portraits.

Done when: every colonist has a portrait/list visual/world sprite and can still be selected reliably.

### Sprint E: Objectives, Alerts, and Minimap

- Replace objective line with objective cards.
- Convert advisor/incidents into alert rows.
- Add minimap panel with a simplified colony footprint.
- Keep debug overlay separate.

Done when: a new player can read current goals, risks, and colony state from the screenshot-style UI.

### Sprint F: Gameplay Reframe

- Add room/action mode switching from the bottom toolbar.
- Rework building placement to use toolbar modes.
- Expand resource categories only where they create real decisions.
- Keep missions, incidents, relationships, and daily summaries integrated.

Done when: gameplay feels like colony management through the new interface rather than the old side panel moved around.

### Sprint G: Polish + Verification

- Add tooltip text for toolbar icons.
- Add hover/selection states.
- Add visual tests/screenshots for standard 1280x720 and 1920x1080.
- Update README with new controls and playtest checklist.
- Run full verifier and native launch smoke.

Done when: a tester can start, read the UI, play a complete run, and restart without developer guidance.

## Asset Work Needed

### Immediate Reference Assets Already Extracted

- `portrait_mara_kovac.png`
- `rel_portrait_male_beard.png`
- `rel_portrait_mara.png`
- `rel_portrait_dark_hair.png`
- `rel_portrait_blond.png`
- `rel_portrait_pale.png`
- `sprite_charlie_world.png`
- `sprite_ilya_world.png`
- `sprite_eva_world.png`
- `sprite_gate_worker_world.png`
- `sprite_habitat_eva_world.png`
- `sprite_mess_eli_world.png`
- `sprite_workshop_worker_world.png`

### Production Asset Requirements

- 6 colonist portraits, 128x128 or 256x256.
- 6 isometric standing sprites, idle/work/eat/sleep variants optional.
- 5 building sprites with interior or roof-cutaway views.
- terrain tile set for dirt, grass, wreckage, crops, water/edge if needed.
- toolbar icons matching screenshot blue line-art style.
- mood/status icons.

## Risks

- Direct screenshot crops may not be legally usable in production.
- Isometric rendering could become a large rewrite if tied too deeply to grid logic.
- New resources can dilute the relationship-manager goal if added before UI readability is stable.
- A visually richer UI can hide state if panels become decorative instead of functional.

## Recommended First Implementation Sprint

Start with **Sprint A: Visual Foundation**.

Reason: it gives immediate visual alignment with the screenshot while keeping the current proven gameplay loop intact. After the style system and layout shell are stable, the isometric map and character art can be swapped in without destabilizing simulation code.

## Current Implementation Status

Completed in the active rebuild:

- Reference extraction exists under `docs/reference/tfl_guide_mvp/`, including visual/color samples and screenshot-cropped placeholder portraits/sprites.
- Screenshot-style color tokens, panel treatment, left rail, right rail, top bar, and bottom toolbar are implemented.
- The central playfield now uses isometric projection for terrain, placement previews, buildings, and colonist positions.
- Placeholder portrait and world-sprite crops are wired into the inspector, relationship row, right rail colonist list, and colony view.
- Objective cards are generated from current game state: survival day, shelter capacity, food buffer, core rooms, and field technology.
- The right rail now reports real gameplay tracks: food days, salvage, prepared meals, survey, repair, and hauling progress.
- Bottom toolbar modes now affect gameplay:
  - `Build` shows all construction plans.
  - `Rooms` filters to Habitat, Mess Hall, and Storage.
  - `Objects` filters to Workshop and Exploration Gate.
  - `Colony` changes the active priority.
  - `Research` launches mission types.
  - `Assign` cycles colonist work roles, which changes future work targets and mission eligibility.
  - `Log` shows recent colony events.
- Buildings render as raised isometric shell placeholders instead of flat colored cells.

Validation currently passing:

- `cargo fmt --check`
- `cargo test`
- `cargo build`
- `git diff --check`
- Native launch smoke test

## Remaining Rebuild Backlog

Highest-value remaining items:

1. Add licensed UI font loading.
   - Preferred: Rajdhani SemiBold/Bold.
   - Keep default font fallback if the font asset fails to load.
   - Apply it through shared text helpers instead of one-off calls.

2. Replace screenshot-cropped placeholder art with production-safe art.
   - Generate or commission six portraits.
   - Generate or commission six standing sprites.
   - Replace screenshot crops once approved art exists.

3. Add richer isometric terrain and building assets.
   - Dirt, wreckage, scrap, paths, and work-zone tile variation.
   - Building interiors or roof-cutaway sprites for each room/object.
   - Stronger selected/hover outlines.

4. Add tooltip and hover polish.
   - Toolbar icon tooltips.
   - Context card hover states with compact consequences.
   - Colonist/world hover cards that do not overlap core panels.

5. Expand relationship-manager gameplay.
   - Assigning roles should expose social consequences, not just production targets.
   - Add relationship-aware work grouping and recovery decisions.
   - Promote daily social stories into player-readable log summaries.

6. Add screenshot-based visual verification.
   - Capture 1280x720 and 1920x1080 smoke screenshots.
   - Check that panels do not overlap, the map is nonblank, and text stays inside key containers.
   - Add a manual playtest checklist to `README.md`.
