# ADR-003: Host/Guest Protocol & Physical Custody ABI

**Status:** ACCEPTED  
**Date:** November 2026  
**Deciders:** Principal Systems Architect, Simulation Architect, API Design Lead  
**Consulted:** Game Design Lead, UI Team  
**Informed:** All Engineering Teams  

---

## 1. Context

Player firmware compiled to `wasm32-unknown-unknown` executes inside an isolated Wasmi virtual machine. To fulfill the gameplay loop, the firmware must:
1. Query machine status (temperature, inventory level, sensor reads, fault codes).
2. Issue commands to physical actuators (engage drill, route conveyor slot, transfer crane arm, vent pressure).
3. Respect the physical reality of the industrial world: **resources cannot be cloned, double-spent, or conjured out of thin air.**

We require an ABI and SDK design that:
- Maintains a clean security boundary (guest cannot write to arbitrary host memory).
- Avoids heavy runtime serialization overhead on 20 Hz simulation ticks.
- Exposes an idiomatic, beginner-friendly, yet authentic Rust API to the player.
- Uses Rust's affine type system (ownership and move semantics) to model physical item custody.

---

## 2. Decision

We establish a two-layer protocol:

### Layer 1: Low-Level C-ABI FFI (`sys` layer)
All host-guest communication is handled by four minimalist `extern "C"` imports provided by the host:
```rust
extern "C" {
    // Queries machine telemetry into guest memory buffer; returns written bytes or error code
    fn host_query_telemetry(machine_id: u32, out_ptr: *mut u8, max_len: u32) -> i32;

    // Dispatches a binary-encoded actuator command from guest memory buffer
    fn host_submit_command(machine_id: u32, cmd_ptr: *const u8, cmd_len: u32) -> i32;

    // Relinquishes remaining fuel for the current simulation tick
    fn host_yield_tick();

    // Emits a debug log message to the in-game machine terminal
    fn host_log_telemetry(level: u32, msg_ptr: *const u8, msg_len: u32);
}
```

### Layer 2: High-Level Safe SDK (`rustbound_api`)
The player code interacts exclusively with `rustbound_api`. The raw FFI imports and `unsafe` blocks are completely encapsulated.
The SDK exposes strongly typed machine drivers:
- `QuarryDrill`
- `ConveyorBelt`
- `StorageBay`
- `TransferCrane`
- `ReactorCore`

### Core Metaphor: Physical Custody via Move Semantics
Industrial items are represented by generic linear tokens:
```rust
pub struct ResourceChunk<T: ResourceKind> {
    id: u64,
    grade: OreGrade,
    _marker: core::marker::PhantomData<T>,
}
```
In `rustbound_api`, transferring an item moves ownership:
```rust
impl ConveyorBelt {
    // Consumes item by value; transfers physical custody to the conveyor slot
    pub fn feed<T: ResourceKind>(&self, item: ResourceChunk<T>) -> Result<(), MachineError> {
        // Encodes command packet, calls host_submit_command, drops token
    }
}
```
If the player writes:
```rust
let ore = drill.extract_chunk()?;
conveyor_a.feed(ore)?;
conveyor_b.feed(ore)?; // COMPILE ERROR: use of moved value: `ore`
```
The official Rust compiler catches the duplication bug at compile time, teaching the player that **Rust move semantics enforce the physical conservation of mass**.

---

## 3. Alternatives Considered

### Alternative A: JSON over FFI
- *Concept:* Serialize telemetry queries and commands as JSON strings.
- *Reason for Rejection:* Parsing JSON strings in a `#![no_std]` guest Wasm environment introduces significant code size overhead, string allocations, and CPU cycles on a 20 Hz simulation loop.

### Alternative B: Wasm Component Model & `wit-bindgen`
- *Concept:* Use the emerging WebAssembly Component Model and WIT IDL.
- *Reason for Rejection:* Immature toolchain stability in 2026 for constrained `#![no_std]` game firmware; heavy canonical ABI runtime support crates; unnecessary abstraction layer for simple 4-function C-ABI dispatch.

### Alternative C: Shared Memory Ring Buffers with Raw Pointers
- *Concept:* Map host memory directly into guest Wasm linear memory.
- *Reason for Rejection:* High security risk. Any guest buffer overflow or illegal write could corrupt adjacent host telemetry structures or other machine states. Slice copies across the boundary are safe, verified, and take < 50 nanoseconds for small command packets.

---

## 4. Consequences

### Positive Consequences
- **Zero `unsafe` in Player Code:** Players write 100% safe Rust.
- **Physical Custody Enforcement:** Double-spending or duplication of extracted minerals is mathematically impossible in safe code.
- **Microsecond Latency:** Binary command packets are small POD structs (8–32 bytes), enabling sub-microsecond host dispatch.
- **Zero Allocations:** SDK operations operate over stack-allocated buffers and zero-copy packet references.

### Negative Consequences & Trade-offs
- **ABI Rigidity:** Modifying packet layouts requires recompiling the SDK rlib.
  - *Mitigation:* Packets include a 32-bit version tag (`version: 1`). The host dispatch system supports legacy packet decoders.

---

## 5. Mitigation of Risks

1. **Buffer Overflow Mitigation:** The host validates that `out_ptr + max_len` and `cmd_ptr + cmd_len` reside strictly within the guest's allocated Wasmi linear memory bounds before copying bytes.
2. **Invalid Pointer Mitigation:** If a guest passes an unmapped pointer, the host traps the VM with an `IllegalPointerAccess` fault and marks the machine's in-game status as "Firmware Hardware Fault".

---

## 6. Validation Requirements

- [x] Verified binary packet serialization between `#![no_std]` guest and host ECS structures.
- [x] Verified `rustc` compiler error `E0382` occurs when player attempts double-feed of `ResourceChunk`.
- [x] Verified host memory safety under adversarial tests passing out-of-bounds pointer arguments.
