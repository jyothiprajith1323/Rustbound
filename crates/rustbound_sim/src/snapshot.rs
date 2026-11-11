//! Rolling snapshot ring buffer and state capture for deterministic rewind.

use std::vec::Vec;

/// Standard ring buffer capacity: 60 ticks = 3.0 seconds at 20 Hz.
pub const SNAPSHOT_RING_CAPACITY: usize = 60;

/// State snapshot of an individual machine entity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineSnapshot {
    pub machine_id: u32,
    pub temperature_celsius: i32,
    pub is_jammed: bool,
    pub is_overheated: bool,
    pub item_count: u32,
}

/// Snapshot of the complete world state at a specific simulation tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub machines: Vec<MachineSnapshot>,
}

/// Rolling in-memory ring buffer storing the last 60 simulation snapshots.
pub struct SnapshotRingBuffer {
    buffer: [Option<WorldSnapshot>; SNAPSHOT_RING_CAPACITY],
    head: usize,
    count: usize,
}

impl Default for SnapshotRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotRingBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [const { None }; SNAPSHOT_RING_CAPACITY],
            head: 0,
            count: 0,
        }
    }

    /// Stores a new snapshot at the current head, overwriting oldest entry if full.
    pub fn push(&mut self, snapshot: WorldSnapshot) {
        self.buffer[self.head] = Some(snapshot);
        self.head = (self.head + 1) % SNAPSHOT_RING_CAPACITY;
        if self.count < SNAPSHOT_RING_CAPACITY {
            self.count += 1;
        }
    }

    /// Returns the number of snapshots currently stored in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Pops the most recent snapshot from the ring buffer for a 1-tick rewind.
    pub fn pop_latest(&mut self) -> Option<WorldSnapshot> {
        if self.count == 0 {
            return None;
        }

        self.head = (self.head + SNAPSHOT_RING_CAPACITY - 1) % SNAPSHOT_RING_CAPACITY;
        let snapshot = self.buffer[self.head].take();
        self.count -= 1;
        snapshot
    }
}
