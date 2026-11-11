# ADR-002: Player Code Compilation and Execution Architecture

**Status:** ACCEPTED  
**Date:** November 2026  
**Deciders:** Principal Systems Architect, Security Engineer, Runtime Engineer  
**Consulted:** Toolchain Team, Game Design Lead  
**Informed:** All Engineering Teams  

---

## 1. Context

Rustbound's primary gameplay loop requires players to write real Rust code, compile it with the official Rust compiler, receive actionable diagnostics, and deploy the compiled artifact to control automated industrial machines.

This introduces two distinct security and architectural boundaries:
1. **The Compilation Boundary:** The player's raw Rust source code is untrusted input processed by the host Rust compiler (`rustc`).
2. **The Execution Boundary:** The compiled bytecode executes alongside or within the game simulation loop at 20 Hz without freezing, corrupting, or compromising the host game process.

We must select:
- A compilation pipeline (direct `rustc` invocation vs `cargo build`).
- A compilation target (`wasm32-unknown-unknown` vs native dynamic libraries).
- An execution runtime (Wasmi interpreter vs Wasmtime JIT vs native execution).

---

## 2. Decision

We lock the compilation and execution architecture as follows:

1. **Compilation Pipeline:**
   - The game compiles player code by directly invoking **`rustc`** as an isolated subprocess.
   - Target triple is locked to **`wasm32-unknown-unknown`** with `#![no_std]` and `-C panic=abort`.
   - The game links against a precompiled **`rustbound_api`** `.rlib` and `.rmeta` SDK library.
   - Player compilation explicitly disables external crates, procedural macros, and build scripts.
   - Diagnostics are captured via `--error-format=json --json=diagnostic-rendered-ansi`.

2. **Subprocess Isolation:**
   - Compiler subprocesses are launched under strict OS-level resource limits:
     - **Linux:** `setrlimit` enforcing `RLIMIT_CPU = 5s`, `RLIMIT_AS = 1GB`, `RLIMIT_FSIZE = 10MB`, `RLIMIT_NPROC = 0`.
     - **Windows:** Assigned to a Windows Job Object configured with `JOB_OBJECT_LIMIT_PROCESS_MEMORY (1GB)` and `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
   - Hard execution timeout: 5,000 ms watchdog in the host process.

3. **Execution Runtime:**
   - Player firmware is executed inside **`wasmi` (version 2.0)** with the **`deterministic`** cargo feature enabled.
   - Execution is throttled using **instruction fuel metering**. Every simulation tick assigns 50,000 fuel units per VM instance.
   - Memory is strictly bound to 16 initial pages (1 MB) and 64 maximum pages (4 MB).
   - Zero OS system calls (WASI) are imported. All external interactions occur via explicit host machine function imports.

---

## 3. Alternatives Considered

### Alternative A: Native Dynamic Libraries (`.dll` / `.so` / `dylib`)
- *Concept:* Player compiles native machine code and the host loads it via `libloading`.
- *Reason for Rejection:* **Catastrophic Security & Stability Risk**. A single null pointer dereference, segfault, or infinite loop in player code crashes the entire game process. Arbitrary code execution on the player's computer cannot be sandboxed without root virtualization (Docker/KVM). Rejected immediately.

### Alternative B: Wasmtime JIT Runtime
- *Concept:* Execute Wasm using Cranelift JIT compilation for maximum execution speed.
- *Reason for Rejection:*
  1. **Deployment Latency:** JIT compilation of a Wasm module takes 20ms – 150ms on low-end CPUs, causing noticeable hitching when the player clicks "Deploy Firmware".
  2. **Memory Footprint:** Wasmtime runtime caches and Cranelift structures consume 15MB – 40MB per active engine, limiting concurrency on 8GB RAM laptops.
  3. **Non-JIT Platforms:** Consoles and hardened OS environments restrict runtime generation of executable memory pages (`W^X` enforcement).
  4. **Determinism Nuances:** Bit-for-bit determinism across x86 and ARM requires strict float handling flags and eliminates JIT SIMD optimizations.

### Alternative C: Custom AST / Bytecode Interpreter
- *Concept:* Write an in-engine Rust AST interpreter (like `rune` or `rhai`).
- *Reason for Rejection:* Defeats the foundational premise of Rustbound. The player must learn and write **real standard Rust**, experiencing the real borrow checker, official error codes, type inference, and compiler diagnostics.

---

## 4. Consequences

### Positive Consequences
- **Absolute Host Safety:** A buggy or malicious player program can neither crash the game nor execute host syscalls or network requests.
- **Deterministic 20 Hz Simulation:** `wasmi` executes bit-for-bit identical instructions across Windows, Linux, macOS, and Steam Deck.
- **Instant Deployment:** `wasmi` parses Wasm bytecode in < 1 ms; zero JIT compile lag upon clicking "Deploy".
- **Real Rust Compiler Feedback:** Players receive the exact compiler diagnostics (`E0382`, `E0502`, `E0308`) produced by the official Rust compiler.
- **Microscopic Memory Overhead:** Each `wasmi` instance consumes < 64 KB overhead above its linear memory allocation.

### Negative Consequences & Trade-offs
- **Interpreter Performance Overhead:** `wasmi` bytecode interpretation is ~10x – 20x slower than native JIT execution.
  - *Mitigation:* Firmware logic in Rustbound consists of state machines, telemetry reading, and actuator command dispatch, typically consuming < 2,000 instructions per tick. At 20 Hz, this takes < 10 microseconds per VM, leaving over 95% of the frame budget for the Bevy client.
- **No Direct Cargo Dependency Ecosystem:** Players cannot write `rand = "0.8"` in a `Cargo.toml`.
  - *Mitigation:* Educational focus is reinforced. All necessary math, collections, and machine drivers are supplied cleanly through the `rustbound_api` SDK.

---

## 5. Mitigation of Risks

1. **Compiler Bomb Mitigation:** A player writing `#![recursion_limit="100000000"]` or giant macros is terminated by the 5-second OS resource limit and watchdog thread, generating a polite "Compilation Timed Out: Compiler resource budget exceeded" UI warning.
2. **Infinite Loop Mitigation:** A player writing `loop {}` consumes their 50,000 fuel allocation, pauses cleanly, and does not block the 20 Hz simulation thread.

---

## 6. Validation Requirements

- [x] Verified `wasmi` 2.0 builds with `--features deterministic` and executes test modules with instruction fuel bounds.
- [x] Verified subprocess execution of `rustc` on Linux with `setrlimit` kills hung processes at 5.0 seconds.
- [x] Verified `--error-format=json` parses into structured diagnostic spans without dropping borrow-checker notes.
