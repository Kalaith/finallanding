# Manual Relationship Playtest Checklist

Use this checklist after relationship, Assign-mode, Log-mode, or character-visual changes. Automated tests and smoke captures cover regressions; this pass checks whether the relationship manager is readable and playable.

## Setup

- Run `cargo test`.
- Run `.\scripts\capture_ui_smoke.ps1`.
- Start a fresh game from the main menu at 1280x720 or larger.
- Keep time paused while checking UI affordances, then run at normal speed for daily-routine checks.

## First Frame Readability

- Confirm the left rail shows current objectives and at least one actionable alert.
- Confirm the right rail shows every survivor with a portrait and Friendly/Tense relationship chip where relevant.
- Select each starting survivor and confirm the inspector shows portrait, job, activity, bars, and relationship line.
- Confirm support/tension markers and body-language frames are visible on the map before opening Log mode.

## Assign Mode Decisions

- Open Assign mode and confirm the selected survivor stays pinned at the start of the roster.
- Cycle filters through ALL, RISK, SUP, PIN, and at least one role filter; confirm visible rows match the selected filter.
- Cycle sorting through ORD, MOOD, BOND, and R-ALL; confirm low-mood or high-pressure survivors move as expected.
- Hover survivor cards and confirm relationship-preview text explains the expected impact before retasking.
- Click survivor cards to change work role, then confirm future work-space compatibility changes.
- Click a compatible Habitat and work room on the map for the selected survivor; confirm HOME/WORK map labels and warnings update.
- Right-click a room or work space and confirm the roster filters to survivors pinned to that specific building instance.
- Use `P-H`, `P-W`, `ALL-H`, and `ALL-W`; confirm capacity and compatibility warnings prevent bad silent assignments.

## Relationship Pressure Loop

- Put a tense pair in the same Habitat or same compatible work room and confirm warning text appears.
- Put a supportive pair together and confirm the assignment reads as beneficial or neutral.
- Advance time through work/eating/recovery periods and confirm daily routine contact changes relationship pressure.
- Confirm strong support/tension creates pulsing social markers and alternates support/tension body-language frames.
- Change colony priority to Recovery, Stockpile, and Survey; confirm the relationship loop still remains readable.

## Log And Archive

- Advance through at least three daily summaries.
- Open Log mode and confirm the social brief names mood, close/tense pair counts, and strongest signal.
- Filter the social archive by ALL, TENSE, and SUP; confirm only matching reports remain.
- Search by survivor name, relationship wording, recommendation text, and day number.
- Open a report drilldown and confirm detail plus recommendation are visible without layout overflow.
- Export the archive and confirm `docs\exports\social_archive.md` contains newest reports first.

## End-To-End Outcome

- Launch at least two mission types from Research mode.
- Reach Day 7 victory or trigger a failure case.
- Confirm the final outcome is understandable from objectives, alerts, resources, and social history.
- Restart and confirm Assign/Log state does not leak from the previous run.
