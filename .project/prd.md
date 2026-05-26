# The Final Landing - Product Requirements Document

> **Document Location:** `.project/prd.md`
>
> This document defines the product requirements, features, and specifications for "The Final Landing".
> Kept as the single source of truth for the MVP.

---

## Overview

### Problem Statement
Standard colony sims often devolve into spreadsheet optimization or direct unit micro-management (e.g., "Right-click to move here"). Players lose connection with the individual colonists' stories.

### Solution
**"The Final Landing"** is a colony simulation focused on **Indirect Control Through Space**. The player influences where people live, work, and relax by placing buildings. Relationships emerge from proximity and shared activities. The game surfaces these stories through AI-generated logs, not cutscenes.

### Target Users
- **Primary:** Fans of RimWorld, Dwarf Fortress, and The Sims who enjoy emergent storytelling and "people watching."
- **Secondary:** Strategy players looking for a more relaxed, narrative-focused experience without combat stress.

### Success Metrics
- Emerging Stories: Players can recount what happened between specific colonists (e.g., "Mara hates Ilya because they share a workshop").
- Interaction: Players rearrange buildings specifically to influence social dynamics.
- Engagement: Players voluntarily read the end-of-day AI logs.

---

## Features

### Core Features (MVP)

#### Feature 1: Emergent Relationship System
**Priority:** P0 (Must Have)

**Description:**
The core pillar. Colonists form opinions (-50 to +50) based on proximity (living near someone), shared activities (working/eating together), and stress levels.

**User Story:**
> As a player, I want to see colonists form likes and dislikes based on who they interact with, so that the colony feels like a living community.

**Acceptance Criteria:**
- [ ] Colonists have hidden relationship values with every other colonist.
- [ ] Proximity (sleeping in adjacent rooms) affects relationships.
- [ ] Shared work (same building) affects relationships.
- [ ] Relationships influence job success chance and mood.

**Technical Notes:**
- "Like / Neutral / Dislike" states derived from numeric values.

---

#### Feature 2: Indirect Control (Buildings & Soft Assignments)
**Priority:** P0 (Must Have)

**Description:**
The player cannot directly control units. Instead, they place buildings and assign "suggested" jobs. Colonists may refuse based on mood or relationships.

**User Story:**
> As a player, I want to place buildings to define where colonists go, without having to micro-manage their movement.

**Acceptance Criteria:**
- [ ] Player can place the 5 MVP buildings: Habitat, Mess Hall, Workshop, Storage, Exploration Gate.
- [ ] Player can suggest jobs (Explorer, Builder, Cook, Hauler).
- [ ] Colonists autonomously decide to accept or refuse jobs.
- [ ] No direct "click-to-move" control.

---

#### Feature 3: AI Story Logs
**Priority:** P0 (Must Have)

**Description:**
The game generates text logs based on simulation events. Meaning is surfaced through writing.

**User Story:**
> As a player, I want to read a daily summary of colony events, so I can understand the social dynamics I missed while watching the screen.

**Acceptance Criteria:**
- [ ] End-of-day summary generated (e.g., "Tension increased between Mara and Ilya").
- [ ] Logs reflect actual game state changes.

---

#### Feature 4: Basic Survival Economy
**Priority:** P0 (Must Have)

**Description:**
A stripped-down economy to support the social play.

**User Story:**
> As a player, I want to manage basic resources so the colony survives, but without getting bogged down in complex production chains.

**Acceptance Criteria:**
- [ ] Track 2 resources: **Supplies** (food/materials) and **Salvage** (building currency).
- [ ] "Exploration Gate" sends colonists off-map to gather resources (abstracted timer).

---

### Secondary Features (Post-MVP)

#### Feature 5: Advanced Exploration
**Priority:** P1 (Should Have - Months 7-8)
**Description:**
Risk vs. reward choices for off-map exploration.

#### Feature 6: UX Polish
**Priority:** P1 (Should Have - Months 9-10)
**Description:**
Tooltips explaining "Why" a colonist refused a job. Speed controls.

---

## User Interface

### Screens/Views
1.  **Main Game View (Grid)**
    *   Isometric/Top-down grid view.
    *   Colonists (Pixel art + Portrait).
    *   Buildings (visible structure).
2.  **Control Panel**
    *   Building Palette (5 items).
    *   Job Board (Soft assignments).
    *   Resource Counters (Supplies, Salvage).
    *   Time Controls (Pause, Speed).
3.  **Log/Story Panel**
    *   Daily summaries.
    *   Event feed.

### Design Guidelines
*   **Style:** Pixel Art with modern UI overlay.
*   **Palette:** Dark theme (Space), Readable text for logs.
*   **Tone:** Melancholic but hopeful. "Firefly" vibes.

---

## Technical Requirements

### Platform
- **Engine:** Macroquad (Rust).
- **Target:** WebGL (WASM) for web play, Native Windows for development.
- **Render:** Pixelated style (image-rendering: pixelated).

### Data
- **State Management:** Strict State Machine (Menu -> Gameplay -> Result).
- **Data-Driven:** All balance values (building costs, relationship modifiers) defined in JSON `assets/`.
- **Persistence:** JSON save files.

### Constraints & Assumptions
- **Constraint:** No Combat.
- **Constraint:** No Children/Generations.
- **Constraint:** No complex Tech Tree.
- **Constraint:** No External Factions (MVP).
- **Assumption:** The fun comes from "People Watching", not "Winning".

---

## Glossary

| Term | Definition |
|------|------------|
| **Soft Job** | A job assignment that a colonist can refuse if their mood is low. |
| **Salvage** | The currency used to construct new buildings. |
| **Supplies** | Consumed daily by colonists to stay alive (abstracted food/water/air). |
| **Grid** | The tile-based world where buildings are placed. |

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-24 | Agent | Initial draft based on tfl_mvp.md |
