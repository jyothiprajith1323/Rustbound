//! Safe driver interface for secure planetary storage bays.

use crate::resource::{ResourceChunk, ResourceKind};
use crate::sys;
use rustbound_protocol::MachineError;
use rustbound_protocol::packets::{StorageDepositCommand, TelemetryPacket};
use zerocopy::{FromBytes, IntoBytes};

/// Driver for a high-capacity resource storage silo or containment bay.
pub struct StorageBay {
    id: u32,
}

impl StorageBay {
    /// Binds to a storage bay on the local bus by ID.
    pub fn bind(id: u32) -> Result<Self, MachineError> {
        let mut buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: Stack buffer is valid for write.
        let res = unsafe { sys::host_query_telemetry(id, buf.as_mut_ptr(), buf.len() as u32) };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        Ok(Self { id })
    }

    /// Queries live telemetry status from the storage bay.
    pub fn telemetry(&self) -> Result<TelemetryPacket, MachineError> {
        let mut buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: Stack buffer is valid for write.
        let res = unsafe { sys::host_query_telemetry(self.id, buf.as_mut_ptr(), buf.len() as u32) };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        TelemetryPacket::read_from_bytes(&buf).map_err(|_| MachineError::HardwareFault)
    }

    /// Returns the total number of items securely stored inside this bay.
    pub fn stored_count(&self) -> u32 {
        self.telemetry().map(|t| t.item_count).unwrap_or(0)
    }

    /// Deposits a resource chunk into this storage bay.
    ///
    /// Consumes the [`ResourceChunk`] by value, permanently transferring ownership.
    pub fn deposit<T: ResourceKind>(&self, item: ResourceChunk<T>) -> Result<(), MachineError> {
        let cmd = StorageDepositCommand::new(item.id(), 0);
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
}
