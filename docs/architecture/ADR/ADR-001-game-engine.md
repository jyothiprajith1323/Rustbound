# ADR-001: Selection of Primary Game Engine

**Status:** ACCEPTED  
**Date:** November 2026  
**Deciders:** Principal Systems Architect, Game Engine Architect, Lead Technical Producer  
**Consulted:** Engine Team, Simulation Team, UI/UX Lead  
**Informed:** All Engineering Teams  

---

## 1. Context

Rustbound is a systems-engineering simulation game combining deep industrial automation (conveyors, mining drills, power grids, logistics cranes) with real-world Rust programming. The player writes and compiles firmware to control physical machines on Aethelgard.

The game requires:
1. **Headless Execution:** The simulation engine must run independently of rendering for automated testing, CI determinism tests, and headless servers.
2. **High-Performance ECS:** Capable of simulating thousands of active conveyor slots, machine state machines, and power networks at a fixed 20 Hz tick.
3. **Ergonomic Rust Integration:** Written 100% in Rust to avoid FFI marshalling bottlenecks, dual garbage collectors/reference counters, and multi-language toolchain friction.
4. **Dockable Desktop UI:** Support for complex editor docks, diagnostics panes, machine inspectors, and interactive visualizers.
5. **Cross-Platform Portability:** Tier-1 support for Windows (x86_64) and Linux (x86_64), with verified support for Steam Deck (AMD Zen 2 APU) and future macOS (Apple Silicon).

---

## 2. Decision

We select **Bevy (version 0.20+)** as the primary game engine for Rustbound, with the architectural constraint that **the simulation domain (`rustbound_sim`) must be implemented as a completely headless `bevy_ecs` library crate**, isolated from Bevy's rendering, windowing, and audio plugins.

### Architectural Boundaries
- `rustbound_sim` depends only on `bevy_ecs`, `bevy_app`, and shared domain crates. It contains zero dependencies on `winit`, `wgpu`, or `bevy_render`.
- `rustbound_client` acts as the presentation layer. It consumes `rustbound_sim`, attaches Bevy visual/audio plugins, and handles user input.
- User interface is rendered using `bevy_egui` (version 0.43+) and `egui_dock` (version 0.21+) for immediate-mode docking windows, diagnostics panes, and code editor buffers.

---

## 3. Alternatives Considered

### Alternative A: Macroquad
- *Pros:* Minimalist, fast compile times, simple immediate-mode rendering loop.
- *Cons:* Lacks an archetype-based ECS; manual state management; no mature scene or docking UI ecosystem; poor fit for scaling to thousands of industrial automation entities.
- *Reason for Rejection:* Lacks architectural foundations required for complex simulation pipelines.

### Alternative B: Fyrox
- *Pros:* Pure Rust, features an integrated visual 3D scene editor, built-in widget UI (`fyrox-ui`).
- *Cons:* Scene-graph OOP architecture creates friction with pure data-oriented simulation; smaller community; steep learning curve; difficult headless ECS extraction.
- *Reason for Rejection:* Decoupling headless simulation from the scene graph requires significant boilerplate compared to `bevy_ecs`.

### Alternative C: Godot 4.x + GDExtension (`godot-rust`)
- *Pros:* Industry-standard visual editor, mature UI nodes, rich asset pipeline.
- *Cons:* Dual object models (Godot reference counting vs Rust ownership); FFI overhead marshalling telemetry across C++ boundaries; headless simulation requires distributing Godot engine binaries; build pipeline fragility across Godot minor versions.
- *Reason for Rejection:* Unacceptable FFI overhead for 20 Hz simulation ticks updating thousands of machine entities; compromised purity of the Rust educational mission.

---

## 4. Consequences

### Positive Consequences
- **True Headless Simulation:** Unit and integration tests can spin up full world simulations in milliseconds without opening an X11/Wayland/Windows window.
- **Cache-Locality and Multi-threading:** Bevy's archetype ECS automatically schedules independent systems across all available CPU cores.
- **Rich Ecosystem:** Direct access to `bevy_egui`, `bevy_panorbit_camera`, and community spatial indexing crates.
- **Pure Rust Stack:** Unified profiling (`cargo flamegraph`, `tracy`), unified memory tracking, and single-language debugging.

### Negative Consequences & Trade-offs
- **API Churn:** Bevy updates frequently with breaking API changes between minor versions.
- **UI Maturation:** `bevy_ui` is still maturing, requiring `bevy_egui` for the complex IDE docking system.
- **Lack of Native Scene Editor:** Level design and quarry layout must initially be built procedurally or via lightweight custom debug tooling.

---

## 5. Mitigation of Risks

1. **API Churn Mitigation:** The simulation crate depends strictly on `bevy_ecs`, which has a significantly more stable API than Bevy's rendering pipeline. Engine updates are gated behind ADRs and isolated presentation-layer refactors.
2. **UI Stability Mitigation:** `bevy_egui` is pinned to a known compatible version with Bevy (Bevy 0.20 -> `bevy_egui` 0.43). All UI widgets are decoupled from simulation state via immutable snapshot queries.

---

## 6. Validation Requirements

- [x] Verified `bevy_ecs` compiles and runs headless unit tests in CI with `--no-default-features`.
- [x] Verified `bevy_egui` docking windows render at 60+ FPS on Intel HD Graphics 620 integrated GPU.
- [x] Verified memory consumption of basic Bevy app with 1,000 ECS entities remains under 600 MB RSS.
