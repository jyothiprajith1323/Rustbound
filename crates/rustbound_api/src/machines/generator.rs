//! Safe driver interface for base power auxiliary generators.

use crate::sys;
use rustbound_protocol::MachineError;
use rustbound_protocol::packets::{GeneratorIgniteCommand, TelemetryPacket};
use zerocopy::{FromBytes, IntoBytes};

/// Driver for a base power auxiliary generator.
pub struct Generator {
    id: u32,
}

impl Generator {
    /// Binds to an auxiliary generator by ID.
    pub fn bind(id: u32) -> Result<Self, MachineError> {
        let mut buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: Stack buffer is valid for write.
        let res = unsafe { sys::host_query_telemetry(id, buf.as_mut_ptr(), buf.len() as u32) };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        Ok(Self { id })
    }

    /// Queries live telemetry status from the generator.
    pub fn telemetry(&self) -> Result<TelemetryPacket, MachineError> {
        let mut buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: Stack buffer is valid for write.
        let res = unsafe { sys::host_query_telemetry(self.id, buf.as_mut_ptr(), buf.len() as u32) };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        TelemetryPacket::read_from_bytes(&buf).map_err(|_| MachineError::HardwareFault)
    }

    /// Ignites the generator ignition sequence at nominal power (level 100).
    pub fn ignite(&self) -> Result<(), MachineError> {
        let cmd = GeneratorIgniteCommand::new(100);
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

    /// Returns `true` if the generator is actively running and supplying power.
    pub fn is_ignited(&self) -> bool {
        self.telemetry()
            .map(|t| (t.status_flags & 0x01) != 0) // bit 0 = running
            .unwrap_or(false)
    }
}
