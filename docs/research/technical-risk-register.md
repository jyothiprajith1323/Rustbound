# Rustbound: Technical Risk Register

**Document ID:** `RISK-001`  
**Status:** ACTIVE  
**Last Updated:** November 2026  
**Subsystem Scope:** Compiler, Runtime, Engine, Simulation, IDE, Platform

---

## 1. Risk Assessment Methodology

Each technical risk is evaluated across three dimensions on a 1–5 scale:
- **Severity (S):** Impact on project viability, security, or core gameplay (1 = Negligible, 5 = Critical/Fatal).
- **Probability (P):** Likelihood of occurrence without architectural mitigation (1 = Improbable, 5 = Highly Probable).
- **Impact Score (IS):** Computed as $S \times P$, with scores ranging from 1 to 25:
  - **Critical Risk (IS 16–25):** Mandatory architectural block; must be resolved before MVP coding.
  - **High Risk (IS 10–15):** Must have verified mitigations and automated test gates.
  - **Moderate Risk (IS 5–9):** Requires monitoring and contingency planning.
  - **Low Risk (IS 1–4):** Manageable with standard engineering hygiene.

---

## 2. Technical Risk Matrix Summary

```
Severity (S)
  5 │ [RISK-01] [RISK-02] [RISK-05]
  4 │ [RISK-04] [RISK-03] [RISK-07] [RISK-08]
  3 │           [RISK-06] [RISK-09] [RISK-10]
  2 │                     [RISK-11] [RISK-12]
  1 │
    └────────────────────────────────────────
        1          2          3          4          5
                    Probability (P)
```

---

## 3. Prioritized Technical Risk Register

| Risk ID | Category | Risk Description | S | P | IS | Mitigation Strategy | Validation Test | Owner |
| :--- | :--- | :--- | :---: | :---: | :---: | :--- | :--- | :--- |
| **RISK-01** | Security / Host | **Compiler Subprocess Resource Exhaustion**<br>Player code exploits deep type recursion (`#![recursion_limit]`), giant array allocation in `const`, or infinite macro expansion to hang/crash host OS. | 5 | 4 | **20** | Execute `rustc` inside an isolated subprocess with strict OS limits (`setrlimit` on Linux, Job Objects on Windows): 5s CPU limit, 1GB RAM cap, zero subprocess creation (`NPROC=0`). Hard timeout kill in host watcher. | `test_compiler_dos_bomb`: Run benchmark with infinite macro recursion; assert compiler process terminated within 5,000 ms with structured timeout diagnostic. | Compiler Subsystem |
| **RISK-02** | Security / VM | **Infinite Loop & Fuel Starvation in Guest VM**<br>Player firmware executes `loop {}` or heavy computation, starving the simulation tick and freezing the game loop. | 5 | 4 | **20** | Enforce Wasmi instruction fuel metering. Every tick allocates a fixed fuel budget (50,000 instructions) per VM. Execution suspends deterministically upon budget depletion; host resumes next tick. | `test_wasm_infinite_loop`: Deploy module with unbounded loop; verify simulation advances, host frame time < 2ms, VM traps or yields cleanly. | Runtime Subsystem |
| **RISK-03** | Simulation | **Non-Deterministic Simulation Drift Across Platforms**<br>IEEE-754 floating-point operations diverge between x86_64, ARM64 (Steam Deck), or Windows/Linux, breaking save replays. | 4 | 4 | **16** | Strict integer/fixed-point arithmetic for all simulation state (tile grid indices, fixed-point conveyor progress). Floating-point restricted strictly to visual interpolation. Compile Wasmi with `deterministic` feature. | `test_cross_platform_replay`: Run a 2,000-tick automated quarry simulation on Linux and Windows; compare SHA-256 hashes of snapshot state at tick 2,000. | Simulation Subsystem |
| **RISK-04** | Security / Host | **Arbitrary Code Execution via Player Dependencies**<br>Player includes external crates or `build.rs` scripts that execute arbitrary malicious code on player's host system. | 5 | 3 | **15** | Eliminate `Cargo` build pipeline in player compilation. Invoke `rustc` directly targeting a single `.rs` file. Disallow external crates; provide only precompiled `rustbound_api` rlib via `--extern`. Disallow `#![link_args]` and `#![no_core]`. | `test_disallowed_extern_crate`: Attempt to compile player code with `extern crate evil;` or build scripts; assert compilation rejected with security diagnostic. | Compiler Subsystem |
| **RISK-05** | Security / VM | **Guest Memory Out-of-Bounds & Host Corruption**<br>Malicious or buggy Wasm pointer access corrupts host memory or crashes game process. | 5 | 3 | **15** | Wasmi enforces WebAssembly linear memory bounds in software. Host imports validate every pointer offset and length (`ptr + len <= memory.len()`) before copying slices. Zero raw host pointers exposed to guest. | `test_wasm_oob_memory_access`: Execute guest module that writes to address `0xFFFFFFFF`; verify Wasmi raises caught memory trap without host panic. | Runtime Subsystem |
| **RISK-06** | Engine / UI | **Bevy API Churn & Breaking Ecosystem Changes**<br>Rapid Bevy releases (0.14 -> 0.15 -> 0.20) introduce breaking ECS and rendering API changes, stalling development. | 3 | 4 | **12** | Decouple simulation engine (`rustbound_sim`) from Bevy rendering. Pin Bevy version in `Cargo.lock`. Gate all engine upgrades behind an Architectural Decision Record (ADR) and integration test suite. | `test_ecs_headless_isolation`: Ensure `rustbound_sim` compiles and tests without any rendering or windowing crates. | Engine Lead |
| **RISK-07** | IDE / UX | **IDE Memory Bloat on Low-End Hardware (8GB RAM)**<br>Attempting to embed full language servers (`ra_ap_ide`) pushes RAM usage above 2GB, causing paging on 8GB laptops. | 4 | 3 | **12** | Reject embedded `ra_ap_ide`. Implement lightweight in-game editor using `ropey`, `tree-sitter-rust`, and parsed `rustc` JSON diagnostics. Provide external IDE launch hook for power users. | `bench_editor_memory_footprint`: Measure total game RSS while editing and compiling; assert total process memory remains under 1,500 MB. | UI / IDE Subsystem |
| **RISK-08** | Toolchain | **Host System Missing Rust Toolchain**<br>Player launches game on a clean system without `rustup` or `rustc` installed, preventing code compilation. | 4 | 3 | **12** | Package a self-contained, minimal bundled toolchain (`rustc`, `rust-std-wasm32-unknown-unknown`, `rust-lld`) inside game distribution directory, or provide an automatic first-boot installer with clear diagnostic UI. | `test_isolated_toolchain_run`: Run compiler worker in an environment with `PATH=""` and `RUSTUP_HOME` unset; verify successful compilation using bundled toolchain. | Build / Infra |
| **RISK-09** | Simulation | **Snapshot Memory Bandwidth & Rewind Lag**<br>Saving full simulation snapshots every tick for 100+ running machines causes memory bloat and frame spikes. | 3 | 3 | **9** | Implement sparse keyframing (every 200 ticks = 10s) combined with a rolling ring buffer of delta-compressed snapshots for the last 60 ticks (3s). Wasmi VM state snapshots only track dirtied memory pages. | `bench_snapshot_tick_duration`: Measure time taken to capture snapshot across 100 active machines; assert snapshot time < 1.5ms. | Simulation Subsystem |
| **RISK-10** | Game Design | **Player Frustration with Borrow Checker Complexity**<br>New programmers hit borrow checker errors (`E0382`, `E0502`) without understanding why physical custody failed, leading to early churn. | 3 | 3 | **9** | Build a dedicated Borrow Visualizer panel in the UI. Map borrow errors to physical world animations (e.g., ghost crane or conveyor jam hologram showing where custody was transferred). Interactive educational tooltips explain the fix. | `test_borrow_visualizer_telemetry`: Verify that `E0382` compiler JSON triggers correct highlight spans and physical custody hint in the UI. | Game Design / UI |
| **RISK-11** | Platform | **Steam Deck / Proton Input and Performance Bottlenecks**<br>Text editing in egui is awkward on Steam Deck virtual keyboard; performance drops on battery. | 2 | 3 | **6** | Implement responsive UI scaling, full gamepad radial menus for simulation controls, external keyboard detection, and optimize Wasmi interpreter loops for AMD Zen 2 CPU architecture. | `test_steam_deck_preset`: Run simulation on Linux with 15W TDP limit; verify 60 FPS viewport rendering and 20 Hz fixed tick consistency. | Platform Engineer |
| **RISK-12** | Protocol | **Host/Guest ABI Drift & Save Incompatibility**<br>Updating the `rustbound_api` SDK invalidates existing player firmware or breaks historic world saves. | 2 | 2 | **4** | Version every protocol packet with a 32-bit magic header (`RSTB`) and protocol version (`v1`). Maintain backward-compatible dispatch shims in host runtime. | `test_abi_version_handshake`: Load and execute a simulated legacy v1 compiled bytecode module on a v2 host runtime; verify graceful degradation or compatibility. | Architecture Lead |

---

## 4. Acceptance Gates for High/Critical Risks

Before the MVP release is signed off, the following gates must be passed:

1. **Gate G-SEC (Security & Sandbox):**
   - Passes 100 consecutive fuzzing tests with adversarial Rust snippets (macro bombs, infinite loops, huge type recursions, illegal syscall attempts).
   - Zero crashes, zero memory leaks, zero host filesystem leaks.
2. **Gate G-DET (Determinism):**
   - 10,000-tick automated test run on Windows x86_64 and Linux x86_64 producing identical state hashes at every 500-tick checkpoint.
3. **Gate G-MEM (Memory Budget):**
   - Peak RSS under 1,500 MB during simultaneous 100-machine simulation, active code editing, and background compilation.
