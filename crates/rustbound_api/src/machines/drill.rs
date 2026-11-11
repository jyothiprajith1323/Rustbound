//! Safe driver interface for subterranean quarry excavator drills.

use crate::resource::{IronOre, OreGrade, ResourceChunk};
use crate::sys;
use rustbound_protocol::MachineError;
use rustbound_protocol::packets::{DrillExtractCommand, TelemetryPacket};
use zerocopy::{FromBytes, IntoBytes};

/// Driver for a heavy quarry excavator drill.
pub struct QuarryDrill {
    id: u32,
}

impl QuarryDrill {
    /// Binds to a quarry drill on the local machine bus by ID.
    pub fn bind(id: u32) -> Result<Self, MachineError> {
        let mut telemetry_buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: telemetry_buf is an exclusive valid stack buffer of correct size.
        let res = unsafe {
            sys::host_query_telemetry(id, telemetry_buf.as_mut_ptr(), telemetry_buf.len() as u32)
        };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        Ok(Self { id })
    }

    /// Queries live telemetry status from the drill.
    pub fn telemetry(&self) -> Result<TelemetryPacket, MachineError> {
        let mut buf = [0u8; size_of::<TelemetryPacket>()];
        // SAFETY: Stack buffer is valid for write of TelemetryPacket bytes.
        let res = unsafe { sys::host_query_telemetry(self.id, buf.as_mut_ptr(), buf.len() as u32) };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        TelemetryPacket::read_from_bytes(&buf).map_err(|_| MachineError::HardwareFault)
    }

    /// Returns the current temperature of the drill core in Celsius.
    pub fn temperature(&self) -> i32 {
        self.telemetry().map(|t| t.temperature_celsius).unwrap_or(0)
    }

    /// Attempts to extract an iron ore chunk from the subsurface vein.
    ///
    /// Returns an owned [`ResourceChunk`] representing exclusive physical custody.
    pub fn extract(&self) -> Result<ResourceChunk<IronOre>, MachineError> {
        let cmd = DrillExtractCommand::new(0, 2);
        let cmd_bytes = cmd.as_bytes();

        // SAFETY: cmd_bytes is a valid slice of initialized memory.
        let res = unsafe {
            sys::host_submit_command(self.id, cmd_bytes.as_ptr(), cmd_bytes.len() as u32)
        };

        if res < 0 {
            return Err(MachineError::from_i32(res));
        }

        // Query the newly extracted token ID from updated telemetry
        let telem = self.telemetry()?;
        let item_id = telem.next_item_id;

        Ok(ResourceChunk::new(item_id, OreGrade::Standard))
    }
}
