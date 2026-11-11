//! Safe, zero-cost, type-safe API for programming Aethelgard industrial automation.
//!
//! # Physical Custody
//!
//! Rustbound models all physical materials as owned linear tokens ([`ResourceChunk`]).
//! Moving an item transfers custody to the receiving machine. Attempting to duplicate
//! or reuse an item after transfer triggers real Rust compiler errors (`E0382`).

#![no_std]

pub mod machines;
pub mod prelude;
pub mod resource;
pub mod sys;

/// Voluntarily yields remaining execution fuel for the current 20 Hz simulation tick.
///
/// In a long-running loop (`loop { ... }`), call this function once per iteration
/// to allow machines to cool, conveyors to advance, and the game loop to progress.
#[inline]
pub fn yield_tick() {
    // SAFETY: Foreign FFI call to host dispatcher with zero arguments.
    unsafe { sys::host_yield_tick() };
}

/// Emits an informational log message to the machine's in-game terminal.
#[inline]
pub fn log(message: &str) {
    // SAFETY: message slice is valid immutable memory.
    unsafe {
        sys::host_log(1, message.as_ptr(), message.len() as u32);
    }
}
