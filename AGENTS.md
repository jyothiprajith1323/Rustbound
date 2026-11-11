# AGENTS.md — Development Constitution & Engineering Protocol
**Project:** Rustbound: The Systems Frontier  
**Version:** 1.0.0  
**Authority:** MANDATORY FOR ALL HUMAN DEVELOPERS AND AI CODING AGENTS  
**Parent Document:** [01-TECHNICAL-ARCHITECTURE-LOCK.md](file:///home/prajith/Rustbound/docs/architecture/01-TECHNICAL-ARCHITECTURE-LOCK.md)

---

## 1. Project Identity & Design Philosophy

**Rustbound: The Systems Frontier** is an industrial automation simulation and systems-programming game set on the hostile world of Aethelgard. The player programs physical machines using real, standard Rust code.

### Core Philosophy:
1. **Authenticity Over Toys:** We do not build a toy scripting language or a pseudo-Rust interpreter. The player writes real Rust code compiled by the official Rust compiler (`rustc`).
2. **Physical Custody as Linear Types:** Rust's ownership system, affine types, and borrow checker directly model the physical conservation of mass. Moving an item token transfers physical custody; duplicating it triggers real borrow-checker errors (`E0382`).
3. **Rock-Solid Determinism:** Factory automation requires exact repeatability. The simulation runs on a headless, fixed-timestep 20 Hz schedule with discrete integer coordinates. Given identical inputs, the world evolves with 100% bit-for-bit identity across Windows, Linux, and Steam Deck.
4. **Air-Tight Security Boundaries:** We execute untrusted player code. The compiler subprocess and the WebAssembly runtime are treated as hostile boundaries isolated by OS resource limits and fuel metering.

---

## 2. Fundamental Engineering Principles

All contributors (AI agents and human engineers) must uphold:
- **Idiomatic Rust:** Follow the official Rust API Guidelines, Rust 2024 idioms, and high-performance practices.
- **Strict Decoupling:** The simulation core (`rustbound_sim`) must remain completely headless, having zero knowledge of rendering, windows, cameras, or audio.
- **Explicit Error Handling:** Panics and unwrap in production code are strictly forbidden. Use `thiserror` for library crates and explicit domain errors.
- **Zero IEEE-754 Floats in Simulation:** All spatial, inventory, thermal, and electrical simulation math must use integer or fixed-point arithmetic.
- **Minimal Dependencies:** Every crate in the workspace must justify its existence. Bloated dependencies or unmaintained crates are rejected.

---

## 3. Rust Coding Standards

Adhere to the 265 rules specified in the project's Rust engineering guidelines:

### 3.1 Ownership, Memory & Borrowing (`own-`, `mem-`)
- **Borrow over Clone (`own-borrow-over-clone`):** Prefer passing `&T` or `&mut T` rather than cloning data structures. Never call `.clone()` merely to satisfy the borrow checker without a profiling justification.
- **Slice over Concrete Collection (`own-slice-over-vec`):** Function parameters must accept `&[T]` instead of `&Vec<T>`, and `&str` instead of `&String`.
- **Move Semantics for Custody:** Transfer of physical in-game resources must use by-value move semantics (`fn feed(&self, item: ResourceChunk<T>)`).
- **Zero Allocations on 20 Hz Hot Paths (`mem-`):** Simulation tick systems must avoid dynamic heap allocations (`Vec::new()`, `String::new()`, `Box::new()`) inside the tick loop. Pre-allocate collections during entity initialization.

### 3.2 Error Handling (`err-`)
- **Library Crates (`err-thiserror-lib`):** Use `thiserror::Error` for all domain errors in `rustbound_protocol`, `rustbound_sim`, `rustbound_runtime`, and `rustbound_compiler`.
- **Application Crates (`err-anyhow-app`):** Use `anyhow::Result` only in the top-level client executable (`rustbound_client`).
- **No Unwraps in Production (`err-no-unwrap-prod`):** `unwrap()` is banned in production crates. Use `?`, `let-else`, or `.expect("detailed invariant explanation")` only where invariant violations represent catastrophic programmer bugs.
- **Lowercase Error Messages (`err-lowercase-msg`):** Error messages must begin with a lowercase letter and contain no trailing punctuation (e.g., `write!(f, "machine {id} intake is jammed")`).

### 3.3 Unsafe Rust Restrictions (`unsafe-`)
- **Player Code:** ZERO unsafe code permitted in `rustbound_api`.
- **Engine & Subsystems:** `unsafe` is strictly forbidden across the codebase unless interfacing directly with OS C-APIs (`setrlimit`, Windows Job Objects) or raw Wasmi foreign function imports.
- **Mandatory Safety Comments:** Every `unsafe` block must be preceded by a `// SAFETY:` comment proving that all invariants and memory layouts are sound.

### 3.4 Naming & Conventions (`name-`)
- Types, structs, enums: `UpperCamelCase` (`QuarryDrill`, `TelemetryPacket`).
- Functions, methods, modules: `snake_case` (`extract_chunk`, `query_telemetry`).
- Constants, static globals: `SCREAMING_SNAKE_CASE` (`MAX_FUEL_PER_TICK`).
- Acronyms: Consistent casing (`WasmRuntime` or `WasmVm`, not `WASMRuntime`).

---

## 4. Bevy Engine & ECS Development Rules

When writing code for `rustbound_sim` or `rustbound_client`:

### 4.1 ECS Component & Resource Separation
- **Components:** Compact Plain-Old-Data (POD) types representing entity state (`Position(GridCoord)`, `ThermalStatus`, `InventorySlot`). Components must NOT contain channels, threads, or engine pointers.
- **Resources:** Global singleton state (`Time<Virtual>`, `SimulationConfig`, `WasmiRuntimePool`).
- **Systems:** Small, focused functions operating over queries. Keep queries as narrow as possible (e.g., `Query<(&mut ThermalStatus, &MachineId), With<ActiveDrill>>`).

### 4.2 System Scheduling & FixedUpdate
- **Simulation Schedule:** All simulation systems belong in the dedicated `SimulationTick` schedule advancing at 20 Hz (50 ms).
- **Presentation Schedule:** Visual interpolation, mesh updates, camera controllers, and egui UI belong in the `Update` schedule running at screen refresh rates.
- **Never Mix:** Never put rendering or input polling systems inside `SimulationTick`. Never put gameplay custody logic inside `Update`.

### 4.3 Change Detection Discipline
- Leverage Bevy's built-in change detection (`Changed<Component>`, `Added<Component>`) to prevent redundant computations in conveyor and electrical systems.

---

## 5. Non-Negotiable Architecture Rules

AI agents and human developers are **STRICTLY FORBIDDEN** from:

1. **Embedding Compiler Logic into the Simulation:** Never invoke `rustc`, parse source code, or manage file scratchpads inside `rustbound_sim`. That belongs exclusively in `rustbound_compiler`.
2. **Executing Untrusted Player Code in the Game Process:** Never use `libloading` or native execution for player code. All player firmware MUST execute inside the sandboxed `wasmi` interpreter with instruction fuel limits.
3. **Coupling Simulation to UI Widgets:** Never pass `egui::Ui` references or UI state into `rustbound_sim`. The UI interacts with the simulation purely via immutable snapshot queries or command channels.
4. **Bypassing the Host/Guest Protocol:** Never invent ad-hoc side channels for guest firmware to read host memory. All data flows through the 4 foreign C-ABI functions in `rustbound_protocol`.
5. **Introducing Floating-Point Math into Simulation:** Never use `f32` or `f64` in `rustbound_sim`. Use `TileCoord(i32, i32, i32)` and `SubTileProgress(u32)`.
6. **Violating Module Boundaries:** Never add dependencies from lower-tier crates to higher-tier crates (e.g., `rustbound_sim` must never depend on `rustbound_client`).
7. **Modifying Locked Architecture Without an ADR:** Never alter a locked architectural decision without preparing an Architectural Decision Record (ADR) and obtaining user approval.

---

## 6. Mandatory Quality & Testing Gates

Before committing or claiming a task is done, the following checks MUST be executed and pass with zero errors:

```bash
# 1. Code formatting check
cargo fmt --check

# 2. Strict Clippy check with zero warnings allowed
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Full test suite execution
cargo test --workspace

# 4. Headless simulation determinism verification
cargo test --package rustbound_sim --test determinism

# 5. Security sandbox benchmark test
cargo test --package rustbound_compiler --test security
```

---

## 7. Dependency Governance

Before adding any new dependency to `Cargo.toml`, verify:
1. **License Compatibility:** Must be MIT, Apache-2.0, or BSD. GPL/LGPL or proprietary licenses are forbidden.
2. **Maintenance Status:** Must have active commits within the last 12 months and no outstanding unpatched RustSec security advisories.
3. **Transitive Bloat:** Inspect dependency tree (`cargo tree`). Avoid crates that pull hundreds of transitive dependencies or massive C libraries.
4. **`#![no_std]` Compatibility:** Any dependency added to `rustbound_api` or `rustbound_protocol` must support `#![no_std]`.

---

## 8. Git Workflow & Commit Discipline

### 8.1 Branch Strategy
- `main`: Production-ready, locked releases.
- `feat/<subsystem>-<description>`: New functional features (e.g., `feat/sim-conveyor-slots`).
- `fix/<subsystem>-<description>`: Bug fixes (e.g., `fix/compiler-span-mapping`).

### 8.2 Commit Message Format
Follow Conventional Commits:
```text
<type>(<scope>): <short description in present tense>

[optional body explaining motivation and context]

[optional footer with issue/ADR references]
```
Allowed types: `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `chore`.

Examples:
- `feat(sim): implement 20hz fixed tick schedule with bevy_ecs`
- `fix(compiler): correct byte-to-char span mapping in json diagnostics`
- `docs(adr): add ADR-002 for wasmi fuel metering runtime`

---

## 9. AI Agent Execution Protocol (11-Step Protocol)

Every AI coding agent operating on the Rustbound repository must execute tasks using this exact sequence:

```
┌────────────────────────────────────────────────────────────────────────┐
│  STEP 1: INSPECT REPOSITORY STATE                                      │
│  Run `git status`, inspect directories, check Cargo.toml & Cargo.lock. │
├────────────────────────────────────────────────────────────────────────┤
│  STEP 2: READ AGENTS.MD & RELEVANT ADRs                                │
│  Understand module boundaries, coding rules, and subsystem ownership.  │
├────────────────────────────────────────────────────────────────────────┤
│  STEP 3: IDENTIFY MINIMAL IMPLEMENTATION PATH                          │
│  Determine the smallest, cleanest change that fulfills the request.    │
├────────────────────────────────────────────────────────────────────────┤
│  STEP 4: WRITE TESTS FIRST (TDD)                                       │
│  Author unit or integration tests exposing the feature or bug.         │
├────────────────────────────────────────────────────────────────────────┤
│  STEP 5: IMPLEMENT CODE CHANGES                                        │
│  Write idiomatic, clean Rust code respecting all coding standards.     │
├────────────────────────────────────────────────────────────────────────┤
│  STEP 6: RUN QUALITY & TESTING CHECKS                                  │
│  Execute `cargo fmt --check`, `cargo clippy`, and `cargo test`.         │
├────────────────────────────────────────────────────────────────────────┤
│  STEP 7: NEVER FABRICATE TEST RESULTS                                  │
│  Never claim a test passed without running the shell tool and seeing 0.│
├────────────────────────────────────────────────────────────────────────┤
│  STEP 8: REVIEW DIFF RIGOROUSLY                                        │
│  Inspect `git diff`. Verify zero unrelated files or debug code leaked. │
├────────────────────────────────────────────────────────────────────────┤
│  STEP 9: UPDATE RELEVANT DOCUMENTATION                                 │
│  Update architecture docs, specs, or risk registers if behavior changed│
├────────────────────────────────────────────────────────────────────────┤
│  STEP 10: PREPARE ADR FOR ARCHITECTURAL CHANGES                        │
│  If a locked decision is challenged, draft an ADR before code changes. │
├────────────────────────────────────────────────────────────────────────┤
│  STEP 11: DELIVER HONEST COMPLETION REPORT                             │
│  List files created/modified, tests executed, and known limitations.   │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 10. Definition of Done (DoD)

A task is officially complete **only when all seven criteria are met**:

1. **Functionality Verified:** The requested feature or bug fix works as specified in [02-MVP-ENGINEERING-SPECIFICATION.md](file:///home/prajith/Rustbound/docs/architecture/02-MVP-ENGINEERING-SPECIFICATION.md).
2. **All Tests Pass:** `cargo test --workspace` passes with zero failures.
3. **Zero Clippy Warnings:** `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits with code 0.
4. **Code Formatted:** `cargo fmt --check` exits with code 0.
5. **No Unrelated Modifications:** `git diff` shows only changes strictly necessary for the assigned task.
6. **Documentation Updated:** Public API docs (`///`), architecture specifications, and risk registers reflect the current state.
7. **Report Delivered:** A concise report detailing what was implemented, what was verified, and what remains is provided to the user.
