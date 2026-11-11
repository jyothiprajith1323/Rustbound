//! Safe driver interface for industrial conveyor belts.

use crate::resource::{ResourceChunk, ResourceKind};
use crate::sys;
use rustbound_protocol::MachineError;
use rustbound_protocol::packets::{ConveyorFeedCommand, TelemetryPacket};
use zerocopy::{FromBytes, IntoBytes};

/// Driver for an automated conveyor belt transporting materials between machines.
pub struct ConveyorBelt {
    id: u32,
}

impl ConveyorBelt {
    /// Binds to a conveyor belt on the local bus by ID.
    pub fn bind(id: u32) -> Result<Self, MachineError> {
        let mut buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: Stack buffer is valid for write of TelemetryPacket bytes.
        let res = unsafe { sys::host_query_telemetry(id, buf.as_mut_ptr(), buf.len() as u32) };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        Ok(Self { id })
    }

    /// Queries live telemetry status from the conveyor belt.
    pub fn telemetry(&self) -> Result<TelemetryPacket, MachineError> {
        let mut buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: Stack buffer is valid for write of TelemetryPacket bytes.
        let res = unsafe { sys::host_query_telemetry(self.id, buf.as_mut_ptr(), buf.len() as u32) };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        TelemetryPacket::read_from_bytes(&buf).map_err(|_| MachineError::HardwareFault)
    }

    /// Returns `true` if the conveyor's intake slot is clear and ready to receive an item.
    pub fn is_intake_clear(&self) -> bool {
        self.telemetry()
            .map(|t| (t.status_flags & 0x02) == 0) // bit 1 = jammed / occupied
            .unwrap_or(false)
    }

    /// Feeds an extracted resource chunk onto the conveyor intake slot.
    ///
    /// # Physical Custody & Ownership
    ///
    /// Notice that this method takes `item` **by value** (`ResourceChunk<T>`).
    /// This moves ownership of the physical item into the conveyor belt.
    /// Attempting to use or feed the same item again triggers compile error `E0382`.
    pub fn feed<T: ResourceKind>(&self, item: ResourceChunk<T>) -> Result<(), MachineError> {
        let cmd = ConveyorFeedCommand::new(item.id(), 0);
        let cmd_bytes = cmd.as_bytes();

        // SAFETY: cmd_bytes is a valid immutable slice.
        let res = unsafe {
            sys::host_submit_command(self.id, cmd_bytes.as_ptr(), cmd_bytes.len() as u32)
        };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        // `item` is dropped here; physical custody has been transferred to the conveyor's ECS slot.
        Ok(())
    }
}
