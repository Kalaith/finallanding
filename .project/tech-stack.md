# The Final Landing - Tech Stack

> **Document Location:** `.project/tech-stack.md`
>
> This document outlines the technology choices and rationale for the project.
> All technology decisions should be documented here with reasoning.

---

## Stack Overview

```
┌─────────────────────────────────────────────────┐
│                   Frontend                       │
│  Macroquad (Canvas) + macroquad-toolkit (UI)    │
│  Immediate Mode GUI                             │
├─────────────────────────────────────────────────┤
│                    Backend                       │
│  Rust 2021 (Internal Game Logic)                 │
│  State Machine Pattern                          │
├─────────────────────────────────────────────────┤
│                   Data Layer                     │
│  JSON Files (Assets/Config/Save)                │
│  Serde + Serde JSON                             │
├─────────────────────────────────────────────────┤
│                Infrastructure                    │
│  Cargo (Build) + GitHub Actions (CI)            │
│  Target: wasm32-unknown-unknown + Windows       │
└─────────────────────────────────────────────────┘
```

---

## Core Technologies

### Language & Runtime

| Technology | Version | Purpose |
|------------|---------|---------|
| Rust | 2021 | Primary language for safety and performance |
| WASM | MVP | WebGL target for browser play |

**Rationale:**
- **Performance:** Rust provides near-native performance for simulation logic.
- **Safety:** Ownership model prevents common game-crashing bugs (null pointers, race conditions).
- **Portability:** Seamless compilation to both Windows and WebAssembly.

---

### Framework

| Technology | Version | Purpose |
|------------|---------|---------|
| Macroquad | 0.4 | Rendering, Input, Audio, Main Loop |
| macroquad-toolkit | * | UI Widgets, Input Helpers |

**Rationale:**
- **Simplicity:** Macroquad is a "thin" layer, avoiding the bloat of engines like Bevy or Unity for 2D games.
- **Immediate Mode:** Fits well with the "UI is a function of State" philosophy mandated in `GAME_DEVELOPMENT_GUIDE.md`.
- **Toolkit:** Provides ready-made UI components (Buttons, Panels) to speed up development.

---

### Database / Data Persistence

| Technology | Version | Purpose |
|------------|---------|---------|
| JSON Files | Standard | Static Assets (Balance, Prompts) & Save Files |
| Serde | 1.0 | Serialization/Deserialization |

**Rationale:**
- **Human Readable:** JSON allows for easy debugging and balancing without tools.
- **Simplicity:** No need for a complex SQL database for an offline, single-player session game.
- **Schema:** Defined by Rust structs (Strongly typed).

**Location:** `assets/` (Static Data) and `save.json` (User Data).

---

## Dependencies

### Production Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `macroquad` | 0.4 | Core Engine |
| `macroquad-toolkit` | Path | UI Library |
| `serde` | 1.0 | Serialization Traits |
| `serde_json` | 1.0 | JSON Parsing |
| `rand` | 0.8 | RNG (Simulation/Events) |

---

## Build & Tooling

### Build System

| Tool | Version | Purpose |
|------|---------|---------|
| Cargo | Latest | Build, Test, Package |
| rustfmt | Latest | Code Formatting |
| clippy | Latest | Linter / Static Analysis |

### Build Commands

```bash
# Development (Windows)
cargo run

# Production Build (Windows)
cargo build --release

# Production Build (Web/WASM)
cargo build --release --target wasm32-unknown-unknown

# Linting
cargo clippy
```

---

## Architecture Patterns

### Code Organization

```
/
├── src/
│   ├── main.rs             # Entry Point & Loop
│   ├── game.rs             # High-level Game Struct
│   ├── state/              # Game States (Menu, Gameplay)
│   ├── engine/             # Stateless Logic Service
│   ├── data/               # Struct Definitions & Loaders
│   └── ui/                 # View Components
├── assets/                 # JSON Data & Images
└── .project/               # Documentation
```

### Design Patterns Used

| Pattern | Where Used | Purpose |
|---------|------------|---------|
| **State Machine** | `state/mod.rs` | Enforce explicit transitions (Menu -> Game -> Pause). |
| **Immediate Mode UI** | `ui/` | Draw UI based on current frame state; no retained DOM. |
| **Data-Driven** | `data/loader.rs` | Load balance values from JSON to allow tuning without recompile. |

---

## Environment Configuration

### Required Environment Variables
*None required for local build.*

### Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Rust Dependencies |
| `assets/constants.json` | Game Balance Configuration |

---

## Decision Log

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|------------------------|
| 2026-01-24 | Use Macroquad | "Simplicity is a feature." Matches Guide. | Bevy (Too complex for MVP), Godot (Too heavy). |
| 2026-01-24 | JSON for Save Data | Human readable, easy to implement with Serde. | SQLite (Overkill for MVP scope). |

---

*Last updated: 2026-01-24*
