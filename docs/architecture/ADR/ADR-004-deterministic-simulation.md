# ADR-004: Deterministic Fixed-Timestep Simulation Architecture

**Status:** ACCEPTED  
**Date:** November 2026  
**Deciders:** Principal Systems Architect, Simulation Architect, Lead Engine Programmer  
**Consulted:** Platform Team, Testing Lead  
**Informed:** All Engineering Teams  

---

## 1. Context

Rustbound's industrial automation world on Aethelgard consists of interconnected physical machines: mining drills, conveyor networks, sorting sorters, robotic transfer cranes, and chemical refineries.

The simulation has four mandatory technical requirements:
1. **Bit-for-Bit Determinism:** Given the same initial world seed, mission configuration, and player firmware, the simulation must evolve identically across Windows, Linux, and Steam Deck.
2. **Instant Rewind & Replay:** Players must be able to pause, step backward by ticks, or replay historical execution to debug automated factory bottlenecks.
3. **Decoupled Presentation:** Visual rendering (60–144 Hz) must be completely decoupled from simulation tick rates (20 Hz) without stutter or temporal aliasing.
4. **Headless Execution:** The simulation must be able to run at uncapped speeds (> 2,000 ticks/sec) in CI test suites to verify mission solutions without a display.

---

## 2. Decision

We establish the simulation architecture on four pillars:

### Pillar 1: Fixed 20 Hz Timestep Schedule
- The simulation advances in discrete ticks at **20 Hz** (50 ms per tick).
- All simulation systems run inside a dedicated Bevy schedule (`SimulationTick`).
- Client rendering interpolates visual transforms (`Transform2D` / `Transform3D`) between the previous tick state and the current tick state based on the fractional accumulator remainder.

### Pillar 2: Strict Integer & Fixed-Point Spatial Model
- **Grid Coordinates:** Machines, tiles, and storage inventories use integer coordinate triples:
  ```rust
  #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Component)]
  pub struct GridCoord {
      pub x: i32,
      pub y: i32,
      pub z: i32,
  }
  ```
- **Conveyor Item Positions:** Sub-tile movement is tracked using fixed-point integer progress:
  ```rust
  #[derive(Copy, Clone, Debug, PartialEq, Eq, Component)]
  pub struct SubTileProgress(pub u32); // 65,536 units = 1.0 full tile traversal
  ```
- **Floating-point Rule:** Zero IEEE-754 floating-point operations (`f32`, `f64`) are permitted in `rustbound_sim`. Floats are strictly confined to presentation shaders, camera controllers, and visual transforms in `rustbound_client`.

### Pillar 3: Deterministic Machine Update Ordering
- Machines are simulated in a deterministic, topological execution sequence:
  1. Sensor and quarry extraction systems.
  2. Conveyor belt slot progression (upstream to downstream to prevent item collisions).
  3. Transfer crane kinematics.
  4. Wasmi firmware execution slices (sorted strictly by stable `MachineId`).
  5. Actuator command application.
  6. Power grid balance calculation.

### Pillar 4: Snapshot Ring Buffer & Delta Compression
- The simulation maintains a 60-tick (3-second) rolling in-memory ring buffer of complete world snapshots.
- Keyframe snapshots are captured every 200 ticks (10 seconds) and saved to disk alongside the player's firmware binary.
- Rewinding restores the nearest keyframe and deterministically fast-forwards the remaining ticks.

---

## 3. Alternatives Considered

### Alternative A: Variable Timestep (`delta_time`) Simulation
- *Concept:* Advance physics and conveyor progress based on the rendering frame's elapsed time.
- *Reason for Rejection:* Non-deterministic by design. Frame rate drops or stutter alter machine outputs, causing factory throughput to fluctuate based on GPU load. Save replays and automated mission grading become impossible.

### Alternative B: Floating-Point Physics Engines (Rapier / PhysX)
- *Concept:* Use continuous 3D rigid-body physics for conveyor items.
- *Reason for Rejection:* Unnecessary CPU overhead and notorious cross-platform divergence. Industrial automation items (ore, ingots, fuel cells) move along discrete slots, not chaotic ragdoll physics.

### Alternative C: Multi-threaded Asynchronous Machine Simulation
- *Concept:* Run each machine's firmware on a separate thread in parallel.
- *Reason for Rejection:* Asynchronous thread scheduling introduces race conditions in resource custody and breaks deterministic replay. Bevy's ECS provides deterministic multi-threading by parallelizing independent archetypes within well-defined system barriers.

---

## 4. Consequences

### Positive Consequences
- **Absolute Reproducibility:** A bug reported by a player can be reproduced with 100% accuracy simply by loading their save file and running the tick journal.
- **Ultra-Fast Automated Testing:** Unit and mission tests can run 1,000 ticks in under 200 ms in CI.
- **Smooth High-Refresh Rendering:** Players with 144 Hz monitors enjoy silky visual motion via transform interpolation while the underlying simulation maintains rock-solid 20 Hz logic.

### Negative Consequences & Trade-offs
- **Interpolation Delay:** Visual presentation lags real simulation state by exactly one tick (50 ms).
  - *Mitigation:* Completely imperceptible in an industrial automation game (Factorio and Satisfactory utilize similar models).

---

## 5. Mitigation of Risks

1. **Floating-Point Contamination:** A strict Clippy lint (`#![deny(clippy::float_arithmetic)]`) is enforced on the `rustbound_sim` crate to prevent accidental introduction of float operations.
2. **Snapshot Memory Bloat:** Snapshots store only mutable component deltas (inventory slot states, machine registers), requiring < 10 KB per tick for the MVP quarry scene.

---

## 6. Validation Requirements

- [x] Verified `clippy::float_arithmetic` prevents float usage across `rustbound_sim`.
- [x] Verified 1,000-tick automated test run produces identical SHA-256 state hashes on Linux and Windows.
- [x] Verified transform interpolation produces smooth 60 FPS visual rendering from a 20 Hz fixed tick.
