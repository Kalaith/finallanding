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
- Q, W, E, R, T: select building tools.
- 1-3: set Recovery, Stockpile, or Survey priority.
- Space: pause or resume time.
- Top bar buttons: adjust time speed.
- Z: undo placement.
- Esc: cancel current tool.

## Current Scope

Playable colony-planning loop with building placement, priorities, time control, and settlement management.

## Verification

- `cargo test`: runs unit coverage plus a headless reference playthrough.
- The reference playthrough starts at Day 1 07:00, advances through the live mission, work, resource, relationship, technology, and scenario systems, and asserts a Day 7 victory inside the 30-40 minute normal-speed window.
