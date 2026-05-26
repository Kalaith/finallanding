# The Final Landing - Build Plan

> **CRITICAL INSTRUCTIONS FOR ENGINEERS**
>
> ## Project Structure
> All project documentation lives in the `.project/` directory at the repository root:
> ```
> .project/
> ├── prd.md           # Product Requirements Document
> ├── tech-stack.md    # Technology choices and rationale
> ├── build-plan.md    # This file - task tracking
> └── changelog.md     # Version history and updates
> ```
>
> ## Build Discipline
> 1. **Keep this document up to date** - Mark tasks as completed immediately after finishing them
> 2. **Build after every task** - Run the build command after completing each task
> 3. **Zero tolerance for warnings/errors** - Fix any warnings or errors before moving to the next task
> 4. **Update changelog.md** - Log significant changes, fixes, and milestones
>
> ```bash
> # Build command (run after each task)
> cargo build --release --target wasm32-unknown-unknown
> ```
>
> If warnings or errors appear, fix them immediately. Do not proceed until the build is clean.

---

## Status Legend

| Icon | Status | Description |
|------|--------|-------------|
| ⬜ | Not Started | Task has not begun |
| 🔄 | In Progress | Currently being worked on |
| ✅ | Completed | Task finished |
| ⛔ | Blocked | Cannot proceed due to external dependency |
| ⚠️ | Has Blockers | Waiting on another task |
| 🔍 | In Review | Pending review/approval |
| 🚫 | Skipped | Intentionally not doing |
| ⏸️ | Deferred | Postponed to later phase/sprint |

---

## Project Progress Summary

```
Phase 1: Setup & Skeleton    [███████████████████] 100%  ✅
Phase 2: Relationships       [░░░░░░░░░░░░░░░░░░░]   0%  ⬜
Phase 3: Logs & Feedback     [░░░░░░░░░░░░░░░░░░░]   0%  ⬜
Phase 4: Exploration         [░░░░░░░░░░░░░░░░░░░]   0%  ⬜
Phase 5: UX & Polish         [░░░░░░░░░░░░░░░░░░░]   0%  ⬜
─────────────────────────────────────────────────────────
Overall Progress             [██░░░░░░░░░░░░░░░░░]  10%
```

---

## Phase 1: Setup & Skeleton (Months 1-2)

### 1.1 Project Foundation
| Status | Task | Description |
|--------|------|-------------|
| ✅ | 1.1.1 | Initialize repository and structure |
| ✅ | 1.1.2 | Define PRD (`.project/prd.md`) |
| ✅ | 1.1.3 | Define Tech Stack (`.project/tech-stack.md`) |
| ✅ | 1.1.4 | Setup Macroquad window and entry point |
| ✅ | 1.1.5 | Implement strict State Machine (Menu/Game) |

### 1.2 Core Systems (Skeleton)
| Status | Task | Description |
|--------|------|-------------|
| ✅ | 1.2.1 | Implement Grid Map Data Structure |
| ✅ | 1.2.2 | Render isometric/top-down grid |
| ✅ | 1.2.3 | Implement Colonist entities (Walking) |
| ✅ | 1.2.4 | Implement Building Placement System |
| ✅ | 1.2.5 | Implement Time System (Day/Night cycle) |

---

## Phase 2: Core Relationships (Months 3-4)

### 2.1 Relationship Engine
| Status | Task | Description |
|--------|------|-------------|
| ⬜ | 2.1.1 | Implement Relationship Data (-50 to +50 matrix) |
| ⬜ | 2.1.2 | Implement Proximity Tracking (who sleeps near whom) |
| ⬜ | 2.1.3 | Implement Shared Activity Tracking (Work/Eat) |

### 2.2 Behaviors
| Status | Task | Description |
|--------|------|-------------|
| ⬜ | 2.2.1 | Implement Mood System (Stress/Happiness) |
| ⬜ | 2.2.2 | Implement Job Soft-Assignment System |
| ⬜ | 2.2.3 | Implement Refusal Logic (Colonist refuses work) |

---

## Phase 3: Logs & Feedback (Months 5-6)

### 3.1 AI Storyteller
| Status | Task | Description |
|--------|------|-------------|
| ⬜ | 3.1.1 | Implement Event Logger system |
| ⬜ | 3.1.2 | Generate Daily Summary text |
| ⬜ | 3.1.3 | Create UI Panel for Reading Logs |

---

## Phase 4: Exploration (Months 7-8)

### 4.1 Off-Map Exploration
| Status | Task | Description |
|--------|------|-------------|
| ⬜ | 4.1.1 | Implement Exploration Gate logic |
| ⬜ | 4.1.2 | Implement Resource Returns (Supplies/Salvage) |
| ⬜ | 4.1.3 | Implement Risk/Reward RNG |

---

## Phase 5: UX & Polish (Months 9-10)

### 5.1 User Experience
| Status | Task | Description |
|--------|------|-------------|
| ⬜ | 5.1.1 | Implement Speed Controls (Pause/1x/2x) |
| ⬜ | 5.1.2 | Add "Why" Tooltips to refusals |
| ⬜ | 5.1.3 | Polish Pixel Art Assets |

---

## Changelog Reference

See `.project/changelog.md` for detailed version history.

---

*Last updated: 2026-01-24*
*Current Phase: Phase 1 - Setup & Skeleton*
