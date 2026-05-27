# The Final Landing

The Final Landing is a colony simulation game about guiding a small crashed settlement by shaping where people live, work, recover, and connect.

You influence the colony through buildings, priorities, and space planning while relationships and pressure emerge from your choices.

## Gameplay

- Place buildings that define colony life and production.
- Set priorities for work, recovery, and settlement needs.
- Arrange space so people can function under pressure.
- Pause, adjust, and undo plans as the colony changes.
- Watch the settlement develop through daily routines.

## Goal

Keep the colony alive after the crash and turn a fragile landing site into a stable community.

## Controls

- Mouse: place buildings and use the interface.
- Bottom toolbar: switch between Build, Rooms, Objects, Colony, Research, Assign, and Log.
- Q, W, E, R, T: select building tools.
- 1-3: set Recovery, Stockpile, or Survey priority.
- Space: pause or resume time.
- Top bar buttons: adjust time speed.
- Z: undo placement.
- Esc: cancel current tool.
- Assign mode: page through survivor cards, hover to preview social pressure, click a survivor card to cycle their work role, then click a compatible map building to pin or clear their recovery/work space; warnings flag over-capacity or tense pins.
- Log mode: review the live social brief and page through daily relationship reports.
- Research mode: click a mission card to launch a field mission when the colony has an Exploration Gate.

## Current Scope

Playable screenshot-style colony-planning loop with isometric placement, room/action toolbar modes, priority control, survivor role assignment, field missions, daily routines, and relationship pressure.
The left rail surfaces current objectives and alerts while the right rail tracks food days, salvage, work progress, active colonists, and local map state.
Strong social ties can now surface directly on the colony map through survivor pose changes and pulsing support/tension markers.

## Verification

- `cargo test`: runs unit coverage plus a headless reference playthrough.
- The reference playthrough starts at Day 1 07:00, advances through the live mission, work, resource, relationship, technology, and scenario systems, and asserts a Day 7 victory inside the 30-40 minute normal-speed window.
- UI hit-zone tests verify the visible menu, speed, priority, building, undo, and mission controls resolve to the intended gameplay actions.
- `.\scripts\capture_ui_smoke.ps1`: builds the native game, captures gameplay screenshots at 1280x720, 1920x1080, Assign mode, Log mode, and pose setup, then verifies important visible regions.

## Playtest Checklist

- Start from the main menu without developer guidance.
- Use Build, Rooms, and Objects modes to place at least one Habitat, Mess Hall, Workshop, Storage, and Exploration Gate.
- Hover toolbar buttons and context cards; confirm tooltips stay inside the playable view.
- Change priorities from Colony mode after an incident and confirm the advisor responds.
- Use Assign mode to page through the roster, retask one survivor, then pin a compatible Habitat or work space from the map and confirm the HOME/WORK marker and any conflict warning appears.
- Confirm tense/supportive survivors show visible body-language markers on the colony map before opening the inspector.
- Open Log mode after several day summaries and confirm the social archive pages through older relationship recommendations.
- Launch at least two different mission types from Research mode and compare risk/reward.
- Hover and inspect one colonist, then identify their strongest or weakest relationship.
- Reach victory or failure, then use the restart button to begin another run.
