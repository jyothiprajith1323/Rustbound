# Rustbound Feature Status — Milestone 1

## Core Infrastructure
- [x] `rustbound_protocol` — `#![no_std]` POD binary packets (`zerocopy 0.8`), C-ABI FFI error codes
- [x] `rustbound_api` — `#![no_std]` linear `ResourceChunk<T>` custody token, machine drivers (`QuarryDrill`, `ConveyorBelt`, `StorageBay`, `Generator`)
- [x] `rustbound_runtime` — Wasmi 2.0 VM with fuel metering (50,000 instructions/tick) and resumable execution
- [x] `rustbound_compiler` — Subprocess `rustc` compiler targeting `wasm32-unknown-unknown`, JSON diagnostics parser, watchdog timeout
- [x] `rustbound_sim` — Headless Bevy ECS 20 Hz simulation, `TileCoord(i32,i32,i32)`, discrete sub-tile progress, 60-tick ring buffer
- [x] `rustbound_client` (Library) — `egui_dock` 4-pane IDE layout, Ropey text editor, borrow checker visualizer, mission runner

## Milestone 1 Implementation Checklist
- [x] Application entry point (`rustbound` binary)
- [x] Bevy `DefaultPlugins` window setup (audio disabled for headless/ALSA-less hosts)
- [x] Ferric Brutalism UI theme & main menu screen
- [x] 3D industrial Dropship Bay environment (PBR materials, platform, bulkheads, accent lighting)
- [x] CAD/Isometric Orbit Camera (orbit, pan, zoom, keyboard navigation)
- [x] 3D Machine Meshes (Generator, Quarry Drill, Conveyor Belt, Storage Bay)
- [x] Raycast machine picking (left click machine -> opens inspector)
- [x] In-game IDE overlay with docking panes (Source Editor, Machine Inspector, Borrow Visualizer, Diagnostics, Terminal)
- [x] Top bar toolbar with tick counter, F5 Deploy button, menu return
- [x] Real `rustc` compilation trigger and firmware deployment into running simulation
- [x] 20 Hz fixed-step simulation execution in Bevy loop
- [x] Mission Briefing modal dialog
- [x] Mission Complete celebration modal dialog
- [x] Mission 01: Cold Boot (Generator ignition)
- [x] Mission 02: Subterranean Extraction (Drill thermal regulation)
- [x] Mission 03: Single Owner Custody (Drill -> Conveyor -> Storage linear types pipeline)
- [x] Mission 04: Immutable Interlock (`let` vs `let mut`, depth registers, `E0384` resolution)
- [x] Mission 05: Thermal Governor (Expression-based returns, functions, cooling cycles)
- [x] Mission 06: Automated Gantry Sorting (Multi-machine logistics, `TransferCrane`, coordinates)
- [x] Expanded compiler watchdog timeout (15s ceiling for constrained environments)
- [x] End-of-milestone verification report
