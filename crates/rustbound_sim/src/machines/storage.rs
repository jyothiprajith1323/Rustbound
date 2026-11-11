//! Planetary storage silo and containment bay simulation component.

use bevy_ecs::component::Component;
use rustbound_protocol::MachineError;
use std::vec::Vec;

/// ECS component modeling a secure planetary storage silo.
#[derive(Clone, Debug, Component)]
pub struct StorageBayComponent {
    pub capacity: u32,
    pub items: Vec<u64>,
}

impl StorageBayComponent {
    pub fn new(capacity: u32) -> Self {
        Self {
            capacity,
            items: Vec::with_capacity(capacity.min(1024) as usize),
        }
    }

    /// Returns the number of items currently stored in the silo.
    #[inline]
    pub fn stored_count(&self) -> u32 {
        self.items.len() as u32
    }

    /// Deposits an item into the storage silo.
    pub fn deposit(&mut self, item_id: u64) -> Result<(), MachineError> {
        if self.items.len() as u32 >= self.capacity {
            return Err(MachineError::StorageFull);
        }

        self.items.push(item_id);
        Ok(())
    }
}
