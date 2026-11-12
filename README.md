# Rustbound: The Systems Frontier

> **"Think in Rust. Build beyond limits."**

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/engine-bevy_0.20-blue.svg)](https://bevyengine.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green.svg)](#license)
[![GitHub Pages](https://img.shields.io/badge/showcase-live_demo-brightgreen.svg)](https://jyothiprajith1323.github.io/Rustbound/)

**Rustbound: The Systems Frontier** is an industrial sci-fi automation and systems engineering simulation game set on the abandoned industrial planet **Aethelgard**. 

Unlike traditional programming games that use simplified pseudo-code or toy scripting languages, Rustbound requires players to write and compile **genuine Rust programs** compiled with the actual `rustc` compiler to `wasm32-unknown-unknown`, executed inside a deterministic WebAssembly virtual machine to restore planetary infrastructure.

---

## 🌌 The World: Aethelgard & The Great Desynchronization

Centuries ago, Aethelgard hosted a vast machine civilization spanning geothermal quarries, automated smelting networks, chemical refineries, and orbital tethers. All infrastructure operated autonomously under an unsafe control system known as **Kernel Zero**.

When Kernel Zero suffered catastrophic memory corruption and concurrent race conditions, the entire planet suffered **The Great Desynchronization**:
* Cargo transport networks collided.
* Chemical refineries experienced unconstrained pressure surges.
* Foundries underwent thermal runaway.

The surviving planetary operating system, **The Ferric Arbiter**, permanently locked down all automation facilities under hardware quarantine. The Arbiter refuses to ignite machinery until its firmware passes strict compile-time safety and runtime verification.

As a Systems Reclamation Engineer, you deploy to Aethelgard's Dropship Bay to rebuild planetary infrastructure by writing real Rust firmware.

---

## ⚡ Core Engineering Pillars

```
+-------------------------------------------------------------------------------+
|                             RUSTBOUND ARCHITECTURE                            |
+-------------------------------------------------------------------------------+
|                                                                               |
|   +---------------------+                       +-------------------------+   |
|   |   In-Game Editor    |                       |    Bevy 0.20 Client     |   |
|   | (Ropey + egui_dock) |                       | (3D Dropship Bay + CAD) |   |
|   +----------+----------+                       +------------+------------+   |
|              |                                               |                |
|              v [F5 Compile]                                  |                |
|   +---------------------+                                    |                |
|   | Subprocess rustc    |                                    |                |
|   | target: wasm32      |                                    | (Render Sync)  |
|   +----------+----------+                                    |                |
|              |                                               |                |
|              v (Wasm Bytecode)                               v                |
|   +---------------------+   POD Packets (zero-copy)  +--------------------+   |
|   | Wasmi 2.0 Engine    |<==========================>| rustbound_sim      |   |
|   | (50k fuel / tick)   |    Guest <-> Host FFI      | Headless 20 Hz ECS |   |
|   +---------------------+                            +--------------------+   |
|                                                                               |
+-------------------------------------------------------------------------------+
```

1. **Genuine Rust Toolchain Integration:**
   Player code in the in-game IDE is piped directly through the host `rustc` compiler targeting `wasm32-unknown-unknown`, linking against the `#![no_std]` `rustbound_api.rlib`. Compiler diagnostics are captured in JSON and rendered in the in-game terminal with precise line spans.

2. **Deterministic WebAssembly Virtual Machine:**
   Firmware runs inside **Wasmi 2.0** with strict fuel metering (50,000 instructions per 50 ms tick) and isolated 16 MiB linear memory pages. Unyielding loops or out-of-bounds pointer accesses trigger deterministic runtime traps without crashing the game host.

3. **Zero-Copy FFI Protocol:**
   Communication between firmware and machine hardware occurs across a 4-function C-ABI FFI protocol using `#![no_std]` Plain Old Data (POD) structures validated via `zerocopy 0.8`.

4. **Linear Types & Resource Custody:**
   Physical resources in transit are modeled as linear Rust tokens: `ResourceChunk<T>`. Dropping or cloning a resource token is forbidden by the compiler. Resources must be explicitly moved into machinery intakes, enforcing memory safety as physical safety.

5. **Headless Deterministic ECS Simulation:**
   The simulation core (`rustbound_sim`) runs at a fixed 20 Hz (50 ms timestep) completely headless with zero floating-point arithmetic (`#![deny(clippy::float_arithmetic)]`) and integer grid coordinates (`TileCoord(i32, i32, i32)`).

6. **Bevy 0.20 CAD & 3D Visualization:**
   A complete 3D industrial scene rendered in Bevy 0.20 with PBR graphite plating, accent lighting, CAD-style orbit camera (orbit, pan, zoom, keyboard navigation), raycast machine picking, and dynamic telemetry gauges.

---

## 🎮 Playable Missions (Milestone 1)

* **Mission 01: Cold Boot**
  - **Objective:** Read telemetry from the Dropship Aux Generator, poll status codes, and issue the ignition cycle command.
  - **Rust Concepts:** Basic program structure, functions, telemetry structs, state checking.
* **Mission 02: Subterranean Extraction**
  - **Objective:** Command the Quarry Drill to extract ore while regulating thermal output through intermittent cooling intervals.
  - **Rust Concepts:** Mutable state, while/match loops, branching logic, threshold monitoring.
* **Mission 03: Single Owner Custody**
  - **Objective:** Transfer ore chunks from Quarry Drill through Conveyor Belts into the Storage Bay without leaking or duplicating resources.
  - **Rust Concepts:** Linear types, ownership, move semantics, zero-leak validation.

---

## 🛠️ Quickstart Guide

### Prerequisites
* **Rust 1.85+** (`rustup toolchain install stable`)
* **Wasm32 Target:**
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
* **System Packages (Linux):**
  ```bash
  sudo apt install clang lld libx11-dev
  ```

### Build & Run
```bash
# Clone the repository
git clone https://github.com/jyothiprajith1323/Rustbound.git
cd Rustbound

# Precompile the wasm32 SDK library
cargo build --release --target wasm32-unknown-unknown -p rustbound_api

# Run the full workspace test suite (33 tests)
cargo test --workspace --jobs 1

# Launch the game!
cargo run --bin rustbound
```

---

## ⌨️ Controls

* **Right Mouse Button + Drag:** Orbit CAD camera
* **Middle Mouse Button + Drag:** Pan camera
* **Mouse Scroll:** Zoom in / out
* **W / A / S / D:** Pan camera horizontally / vertically
* **Left Click (3D Viewport):** Raycast select machine into Inspector
* **F5 Key:** Compile current editor code and deploy firmware to selected machine
* **Menu Button:** Return to Main Menu

---

## 📂 Workspace Structure

```
Rustbound/
├── Cargo.toml                                 # Workspace manifest (Bevy modular features)
├── Rustbound Game Development Blueprint.pdf   # 63-page master design & architecture document
├── crates/
│   ├── rustbound_protocol/                    # #![no_std] POD packets & C-ABI FFI definitions
│   ├── rustbound_api/                         # Player-facing SDK with linear ResourceChunk<T>
│   ├── rustbound_runtime/                     # Wasmi 2.0 VM host runner & fuel metering
│   ├── rustbound_compiler/                    # Subprocess rustc compiler & diagnostics parser
│   ├── rustbound_sim/                         # Headless 20 Hz deterministic ECS simulation
│   └── rustbound_client/                      # Bevy 0.20 3D renderer & egui_dock IDE overlay
└── docs/
    ├── architecture/                          # Architecture lock, specs, & ADRs (001-004)
    ├── development/                           # Feature status checklist & engineering logs
    └── research/                              # Technology validation & risk registers
```

---

## 📄 Documentation & Blueprint

The full 63-page technical blueprint document is included in the repository root:
* [Rustbound Game Development Blueprint.pdf](file:///home/prajith/Rustbound/Rustbound%20Game%20Development%20Blueprint.pdf)
* [Technical Architecture Lock](file:///home/prajith/Rustbound/docs/architecture/01-TECHNICAL-ARCHITECTURE-LOCK.md)
* [MVP Engineering Specification](file:///home/prajith/Rustbound/docs/architecture/02-MVP-ENGINEERING-SPECIFICATION.md)

---

## 🌐 Live Showcase & GitHub Page

Explore the interactive web showcase, animated telemetry simulator, and blueprint preview:  
👉 **[https://jyothiprajith1323.github.io/Rustbound/](https://jyothiprajith1323.github.io/Rustbound/)**

---

## 📜 License

Licensed under either of:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
