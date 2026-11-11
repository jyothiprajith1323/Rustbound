//! Industrial conveyor belt simulation component and topological advancement system.

use bevy_ecs::component::Component;
use rustbound_protocol::MachineError;

/// Number of discrete sequential slots along a single conveyor segment.
pub const CONVEYOR_SLOT_COUNT: usize = 4;

/// ECS component modeling a conveyor segment transporting items along discrete slots.
#[derive(Clone, Debug, Component)]
pub struct ConveyorBeltComponent {
    /// Slots from index 0 (intake) to index 3 (exit)
    pub slots: [Option<u64>; CONVEYOR_SLOT_COUNT],
    /// Progress speed in fixed-point units per tick
    pub speed_units_per_tick: u32,
    pub is_jammed: bool,
    pub total_transported: u32,
}

impl Default for ConveyorBeltComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ConveyorBeltComponent {
    pub fn new() -> Self {
        Self {
            slots: [None; CONVEYOR_SLOT_COUNT],
            speed_units_per_tick: 16_384, // Traverses 1 tile every 4 ticks (0.2 seconds at 20 Hz)
            is_jammed: false,
            total_transported: 0,
        }
    }

    /// Returns `true` if the intake slot (slot 0) is empty and ready to accept an item.
    #[inline]
    pub fn is_intake_clear(&self) -> bool {
        self.slots[0].is_none() && !self.is_jammed
    }

    /// Feeds an item onto intake slot 0.
    pub fn feed(&mut self, item_id: u64) -> Result<(), MachineError> {
        if self.slots[0].is_some() {
            self.is_jammed = true;
            return Err(MachineError::IntakeJammed);
        }

        self.slots[0] = Some(item_id);
        self.is_jammed = false;
        Ok(())
    }

    /// Pops an item ready for collection at the exit slot (slot 3).
    #[inline]
    pub fn pop_exit(&mut self) -> Option<u64> {
        let item = self.slots[CONVEYOR_SLOT_COUNT - 1].take();
        if item.is_some() {
            self.total_transported = self.total_transported.saturating_add(1);
        }
        item
    }
}

/// Simulation system that advances conveyor slots topologically from exit to intake.
///
/// Evaluating downstream slots first ensures items move cleanly forward without clobbering
/// or double-stepping in a single tick.
pub fn conveyor_advancement_system(mut query: bevy_ecs::system::Query<&mut ConveyorBeltComponent>) {
    for mut conveyor in &mut query {
        // Step backward from exit towards intake: 2 -> 3, 1 -> 2, 0 -> 1
        for i in (0..(CONVEYOR_SLOT_COUNT - 1)).rev() {
            if conveyor.slots[i + 1].is_none() && conveyor.slots[i].is_some() {
                conveyor.slots[i + 1] = conveyor.slots[i].take();
            }
        }
    }
}
