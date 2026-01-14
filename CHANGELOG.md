# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0] - 2026-01-15 "The Architect Update"

> **Summary**: This release marks a complete architectural overhaul of the EyeMotion project, transitioning from a prototype to a production-grade application. The entire codebase has been refactored to separate concerns, improve stability, and enable cross-platform scalability.

### 🏗️ Architecture & Core (Major Refactor)
- **Rust Core Extraction**: Extracted all business logic, physics engine, and state management into a dedicated `eyemotion-core` crate. This ensures that game logic is completely decoupled from the presentation layer.
- **State Management**: Implemented thread-safe `Mutex<GameState>` within the Rust backend, serving as the Single Source of Truth (SSOT). Frontend now strictly renders state snapshots, eliminating desync issues.
- **Type-Safe Bridge**: Introduced strict TypeScript interfaces (`bridge.ts`) that mirror Rust structs 1:1, ensuring type safety across the FFI boundary.
- **Modular Design**: Codebase split into `core` (logic), `src-tauri` (app shell), and `ui` (presentation), following Clean Architecture principles.

### 🎨 Rendering Engine 2.0
- **TypeScript Migration**: Rewrote the entire rendering layer from vanilla JS to strict **TypeScript** (`renderer.ts`).
- **Pixel-Perfect Rendering**: Implemented sub-pixel rounding logic to solve "fuzzy edge" issues on high-DPI displays.
- **Advanced Anti-Aliasing**:
    - **Inside Stroke**: Custom implementation of "Inside" stroke alignment using Canvas `clip()` for sharper UI elements.
    - **Dual-Pass Rendering**: Added a secondary 1px low-opacity pass to smooth out jagged edges introduced by clipping.
- **Logic-Render Decoupling**: The rendering loop (`requestAnimationFrame`) is now independent of the physics tick, preparing the ground for high-refresh-rate interpolation.

### ⚡ Performance & Engineering
- **Binary Diet**: Reduced executable size by **~60%** (11MB → 4.6MB) through:
    - Enabling LTO (Link Time Optimization).
    - Setting `codegen-units = 1` for better optimization.
    - Stripping debug symbols.
    - Using `panic = "abort"` to remove unwinding code.
- **High-FPS Support**: Engine verified and optimized for **240Hz - 540Hz** eSports monitors.

### 🎮 Gameplay Balancing
- **Mouse-Only Control**: Completely removed keyboard dependencies. All interactions (Start, Pause, Restart, Quit) are now handled via intuitive mouse clicks.
- **Stage 2 Evolution**: Replaced "Pure Vertical" motion with "Near Vertical" (random 5-15° deviation) to prevent user adaptation and improve training efficacy.
- **Stage 3 Tuning**: Vertical speed is now strictly locked to **0.7x** of horizontal speed.
- **Global Pacing**: Adjusted global speed multipliers for Stage 2 and Stage 4 to optimize the difficulty curve.

### 📝 Documentation & Compliance
- **Open Source Ready**: Added `LICENSE` (MIT), `README_ZH.md` (Chinese Manual), and updated `.gitignore` for a clean repository.
- **Release Automation**: Added GitHub Actions workflow (`release.yml`) for automated Windows builds.

---

## [1.0.0] - 2026-01-11
- Initial prototype release.
- Basic linear and circular tracking modes.
- Simple JS-based logic.
