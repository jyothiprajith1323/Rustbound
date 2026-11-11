//! Safe driver interface for robotic gantry cranes and transfer arms.

use crate::resource::{IronOre, OreGrade, ResourceChunk};
use crate::sys;
use rustbound_protocol::MachineError;
use rustbound_protocol::packets::{CraneTransferCommand, TelemetryPacket};
use zerocopy::{FromBytes, IntoBytes};

/// Driver for an automated overhead gantry crane.
pub struct TransferCrane {
    id: u32,
}

impl TransferCrane {
    /// Binds to a transfer crane on the local bus by ID.
    pub fn bind(id: u32) -> Result<Self, MachineError> {
        let mut buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: Stack buffer is valid for write.
        let res = unsafe { sys::host_query_telemetry(id, buf.as_mut_ptr(), buf.len() as u32) };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        Ok(Self { id })
    }

    /// Queries live telemetry status from the crane.
    pub fn telemetry(&self) -> Result<TelemetryPacket, MachineError> {
        let mut buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: Stack buffer is valid for write.
        let res = unsafe { sys::host_query_telemetry(self.id, buf.as_mut_ptr(), buf.len() as u32) };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        TelemetryPacket::read_from_bytes(&buf).map_err(|_| MachineError::HardwareFault)
    }

    /// Instructs the crane trolley to move to target 2D grid coordinates.
    pub fn move_to(&self, x: i32, y: i32) -> Result<(), MachineError> {
        let cmd = CraneTransferCommand::new(x, y, false);
        let cmd_bytes = cmd.as_bytes();

        // SAFETY: cmd_bytes is a valid slice.
        let res = unsafe {
            sys::host_submit_command(self.id, cmd_bytes.as_ptr(), cmd_bytes.len() as u32)
        };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        Ok(())
    }

    /// Grabs an item at the current coordinates, acquiring physical custody.
    pub fn grab(&self) -> Result<ResourceChunk<IronOre>, MachineError> {
        let cmd = CraneTransferCommand::new(0, 0, true);
        let cmd_bytes = cmd.as_bytes();

        // SAFETY: cmd_bytes is a valid slice.
        let res = unsafe {
            sys::host_submit_command(self.id, cmd_bytes.as_ptr(), cmd_bytes.len() as u32)
        };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        let telem = self.telemetry()?;
        Ok(ResourceChunk::new(telem.next_item_id, OreGrade::Standard))
    }

    /// Releases an item into a container or conveyor below the crane.
    pub fn release(&self, item: ResourceChunk<IronOre>) -> Result<(), MachineError> {
        let cmd = CraneTransferCommand::new(0, 0, false);
        let cmd_bytes = cmd.as_bytes();

        // SAFETY: cmd_bytes is a valid slice.
        let res = unsafe {
            sys::host_submit_command(self.id, cmd_bytes.as_ptr(), cmd_bytes.len() as u32)
        };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        // `item` dropped, custody transferred to destination below crane
        let _ = item;
        Ok(())
    }
}
