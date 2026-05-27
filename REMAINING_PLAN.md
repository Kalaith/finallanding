# The Final Landing Remaining Plan

## Current Baseline

The game now has a verified 30-40 minute playable slice:

- A Day 1 to Day 7 survival objective.
- Building placement, undo, time controls, and priority controls.
- Resource pressure through supplies, salvage, meals, storage, daily needs, and colony condition.
- Abstract missions that return resources, injuries, and technology items.
- Relationship and mood pressure through shared work, meals, habitats, proximity, and refusals.
- A player-facing advisor, relationship summary, colony log, objective line, and debug overlay.
- Automated proof that the live systems can reach a Day 7 victory inside the 30-40 minute normal-speed window.
- UI hit-zone tests for the visible menu, speed, priority, building, undo, and mission controls.

## Remaining Goal

Move from a verified playable slice to a stronger colony relationship manager where players care about people, not only the resource checklist.

The main remaining risk is not run length. It is whether players understand and emotionally read the colony during the run.

## Sprint 1: Relationship Readability

Purpose: make social consequences clear enough that players notice and respond.

- Add a compact colonist inspection panel on hover or click.
- Show each colonist's mood, job preference, current activity, injury status, and strongest relationship.
- Add clearer relationship event language for work, meals, habitat sharing, and strained proximity.
- Add tests for relationship summary edge cases: all neutral, multiple hostile pairs, and one strong positive pair.

Done when:

- A player can identify who is tense or connected without opening debug.
- The colony log explains why at least one relationship changed.

## Sprint 2: Planning Feedback

Purpose: make building placement feel like space planning instead of dropping colored rectangles.

- Show building footprint, cost, and purpose in the placement preview.
- Add invalid-placement reasons near the cursor.
- Show whether a selected building helps recovery, food, salvage, storage, or exploration.
- Add a small post-placement log line that tells the player what changed mechanically.

Done when:

- A new player can place the first five buildings without guessing what each one is for.
- Failed placement attempts explain the failure.

## Sprint 3: Mission Depth

Purpose: make missions a recurring decision across the 30-40 minute run.

- Add two more mission types with different duration, danger, and reward profiles.
- Let priority mode visibly affect mission recommendation and risk.
- Add mission cooldown or crew fatigue if rapid scanning becomes too optimal.
- Expand mission logs so injuries, discoveries, and tech unlocks feel less repetitive.

Done when:

- The player has a real reason to choose between quick supplies, safer scouting, and riskier tech pushes.

## Sprint 4: Mid-Run Pressure Events

Purpose: keep minutes 15-30 from becoming autopilot.

- Add timed colony incidents: ration spoilage, habitat conflict, tool breakage, morale dip, or storm warning.
- Route incidents through existing systems instead of adding new resource types.
- Let incidents create short-term advisor priorities.
- Add verifier coverage that the reference run can survive at least one incident.

Done when:

- A normal run requires at least one meaningful adjustment after the opening setup.

## Sprint 5: End-of-Day Story

Purpose: make the relationship-manager identity stronger.

- Rewrite daily summaries into more specific human-readable reports.
- Include one best relationship, one worst relationship, and one pressure recommendation.
- Add more varied social log text so repeated co-work/co-meal events do not read mechanically.
- Preserve concise UI text; keep full story detail in the log or summary.

Done when:

- Players can recall a small story about who helped, clashed, recovered, or carried the colony.

## Sprint 6: Balance Pass

Purpose: make the Day 7 victory neither automatic nor opaque.

- Tune starting salvage, supplies, building costs, work thresholds, mission rewards, injury duration, and daily supply pressure.
- Add reference playthrough variants for conservative, survey-heavy, and recovery-heavy strategies.
- Track whether each strategy wins, fails, or limps into Day 7.
- Decide the target failure rate for poor planning.

Done when:

- A reasonable strategy can win.
- Ignoring food, habitats, or missions can lose.
- The advisor points toward recovery before the colony collapses.

## Sprint 7: Presentation Polish

Purpose: reduce friction and make the game easier to show or playtest.

- Improve menu copy with a short one-screen premise and controls.
- Add a restart button on victory/failure.
- Improve colonist visuals beyond simple circles if time allows.
- Add a short playtest checklist to the README.

Done when:

- A tester can start, play, finish, and restart without developer guidance.

## Verification To Keep Green

Run these after each sprint:

```powershell
cargo fmt
cargo test
cargo build
git diff --check
```

Current key gates:

- The reference playthrough must still prove Day 7 victory in the 30-40 minute window.
- UI hit-zone tests must still match visible controls.
- Native launch smoke should pass before calling a milestone complete.

## Non-Goals For Now

- No combat.
- No external factions.
- No children or generations.
- No large tech tree expansion.
- No map exploration screen.
- No complex save/load until the core relationship loop is more compelling.

## Recommended Next Sprint

Start with **Sprint 1: Relationship Readability**.

That sprint is the highest leverage because the game already survives for the requested length. The next improvement should make the relationship-manager part legible and emotionally actionable during that run.
