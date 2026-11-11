//! Physical token custody tracking and move validation across machines.

use alloc::collections::BTreeMap;
use rustbound_protocol::MachineError;

/// Central authority verifying that physical item custody is strictly conserved.
///
/// Prevents token cloning, forged item IDs, or double-transfer across machines.
#[derive(Default, Clone, Debug)]
pub struct CustodyTracker {
    /// Maps each existing physical `item_id` to the `machine_id` currently holding it.
    holders: BTreeMap<u64, u32>,
}

impl CustodyTracker {
    pub fn new() -> Self {
        Self {
            holders: BTreeMap::new(),
        }
    }

    /// Registers a newly extracted resource chunk under custody of `machine_id`.
    pub fn register_extracted(
        &mut self,
        item_id: u64,
        machine_id: u32,
    ) -> Result<(), MachineError> {
        if self.holders.contains_key(&item_id) {
            return Err(MachineError::CustodyViolation);
        }
        self.holders.insert(item_id, machine_id);
        Ok(())
    }

    /// Validates that `from_machine` currently holds custody of `item_id`, and transfers
    /// custody to `to_machine`.
    pub fn verify_and_transfer(
        &mut self,
        item_id: u64,
        from_machine: u32,
        to_machine: u32,
    ) -> Result<(), MachineError> {
        match self.holders.get(&item_id) {
            Some(&current_holder) if current_holder == from_machine => {
                self.holders.insert(item_id, to_machine);
                Ok(())
            }
            _ => Err(MachineError::CustodyViolation),
        }
    }

    /// Consumes an item permanently (e.g. smelted, processed, or spent as fuel).
    pub fn consume(&mut self, item_id: u64, machine_id: u32) -> Result<(), MachineError> {
        match self.holders.get(&item_id) {
            Some(&current_holder) if current_holder == machine_id => {
                self.holders.remove(&item_id);
                Ok(())
            }
            _ => Err(MachineError::CustodyViolation),
        }
    }

    /// Returns the machine currently holding physical custody of `item_id`, if any.
    pub fn current_holder(&self, item_id: u64) -> Option<u32> {
        self.holders.get(&item_id).copied()
    }

    /// Returns the total count of active physical items in circulation.
    pub fn total_active_items(&self) -> usize {
        self.holders.len()
    }
}
