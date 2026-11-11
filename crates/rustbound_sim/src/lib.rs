//! Deterministic, headless simulation engine for Rustbound: The Systems Frontier.
//!
//! # Determinism Guarantee
//!
//! This crate strictly enforces integer and fixed-point arithmetic.
//! Floating-point operations (`f32`, `f64`) are forbidden in the simulation core.

#![deny(clippy::float_arithmetic)]

pub mod grid;
pub mod machines;
pub mod schedule;
pub mod snapshot;

pub use grid::{Direction, SubTileProgress, TileCoord};
pub use machines::{
    CONVEYOR_SLOT_COUNT, ConveyorBeltComponent, GeneratorComponent, MachineId,
    QuarryDrillComponent, StorageBayComponent, conveyor_advancement_system, drill_cooling_system,
};
pub use schedule::SimulationRunner;
pub use snapshot::{MachineSnapshot, SNAPSHOT_RING_CAPACITY, SnapshotRingBuffer, WorldSnapshot};
