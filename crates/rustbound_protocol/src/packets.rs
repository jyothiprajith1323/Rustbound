//! Binary Plain-Old-Data (POD) packets exchanged across the host-guest boundary.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Standard 8-byte packet header present at the beginning of all command packets.
///
/// 8-byte alignment ensures subsequent 64-bit fields (`u64`) are naturally aligned
/// with zero implicit padding bytes.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PacketHeader {
    /// Magic signature bytes: b"RB"
    pub magic: [u8; 2],
    /// Protocol version (currently 1)
    pub version: u8,
    /// Command identifier
    pub command_id: u8,
    /// Explicit 4-byte padding ensuring 8-byte header size
    pub _reserved: [u8; 4],
}

impl PacketHeader {
    pub const MAGIC: [u8; 2] = *b"RB";
    pub const CURRENT_VERSION: u8 = 1;

    #[inline]
    pub const fn new(command_id: u8) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::CURRENT_VERSION,
            command_id,
            _reserved: [0; 4],
        }
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.magic[0] == Self::MAGIC[0]
            && self.magic[1] == Self::MAGIC[1]
            && self.version == Self::CURRENT_VERSION
    }
}

pub mod commands {
    pub const DRILL_EXTRACT: u8 = 10;
    pub const CONVEYOR_FEED: u8 = 20;
    pub const STORAGE_DEPOSIT: u8 = 30;
    pub const CRANE_TRANSFER: u8 = 40;
    pub const GENERATOR_IGNITE: u8 = 50;
}

/// Command packet sent to initiate a drilling extraction cycle.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DrillExtractCommand {
    pub header: PacketHeader,
    pub target_vein_id: u32,
    pub power_setting: u8,
    pub _reserved: [u8; 3],
}

impl DrillExtractCommand {
    pub const COMMAND_ID: u8 = commands::DRILL_EXTRACT;

    #[inline]
    pub const fn new(target_vein_id: u32, power_setting: u8) -> Self {
        Self {
            header: PacketHeader::new(Self::COMMAND_ID),
            target_vein_id,
            power_setting,
            _reserved: [0; 3],
        }
    }
}

/// Command packet to feed an item onto a conveyor belt intake slot.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConveyorFeedCommand {
    pub header: PacketHeader,
    pub item_id: u64,
    pub target_slot: u32,
    pub _reserved: [u8; 4],
}

impl ConveyorFeedCommand {
    pub const COMMAND_ID: u8 = commands::CONVEYOR_FEED;

    #[inline]
    pub const fn new(item_id: u64, target_slot: u32) -> Self {
        Self {
            header: PacketHeader::new(Self::COMMAND_ID),
            item_id,
            target_slot,
            _reserved: [0; 4],
        }
    }
}

/// Command packet to deposit an item into a storage bay.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageDepositCommand {
    pub header: PacketHeader,
    pub item_id: u64,
    pub bay_index: u32,
    pub _reserved: [u8; 4],
}

impl StorageDepositCommand {
    pub const COMMAND_ID: u8 = commands::STORAGE_DEPOSIT;

    #[inline]
    pub const fn new(item_id: u64, bay_index: u32) -> Self {
        Self {
            header: PacketHeader::new(Self::COMMAND_ID),
            item_id,
            bay_index,
            _reserved: [0; 4],
        }
    }
}

/// Command packet to instruct a gantry crane to move and transfer items.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CraneTransferCommand {
    pub header: PacketHeader,
    pub target_x: i32,
    pub target_y: i32,
    pub grab: u8,
    pub _reserved: [u8; 7],
}

impl CraneTransferCommand {
    pub const COMMAND_ID: u8 = commands::CRANE_TRANSFER;

    #[inline]
    pub const fn new(target_x: i32, target_y: i32, grab: bool) -> Self {
        Self {
            header: PacketHeader::new(Self::COMMAND_ID),
            target_x,
            target_y,
            grab: grab as u8,
            _reserved: [0; 7],
        }
    }
}

/// Command packet to ignite an auxiliary generator.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeneratorIgniteCommand {
    pub header: PacketHeader,
    pub power_level: u32,
    pub _reserved: [u8; 4],
}

impl GeneratorIgniteCommand {
    pub const COMMAND_ID: u8 = commands::GENERATOR_IGNITE;

    #[inline]
    pub const fn new(power_level: u32) -> Self {
        Self {
            header: PacketHeader::new(Self::COMMAND_ID),
            power_level,
            _reserved: [0; 4],
        }
    }
}

/// Standard machine telemetry packet read by firmware from the host.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TelemetryPacket {
    /// Unique machine identifier
    pub machine_id: u32,
    /// Machine type classifier (1 = Drill, 2 = Conveyor, 3 = Storage, 4 = Crane, 5 = Generator)
    pub machine_type: u32,
    /// Operational status flags (bit 0 = running, bit 1 = jammed, bit 2 = overheat, bit 3 = no_power)
    pub status_flags: u32,
    /// Temperature reading in whole degrees Celsius
    pub temperature_celsius: i32,
    /// Current power consumption in Watts
    pub power_draw_watts: u32,
    /// Number of items stored or in intake
    pub item_count: u32,
    /// Next available item ID for extraction/pickup (0 if none)
    pub next_item_id: u64,
    /// Tick count when this telemetry was captured
    pub captured_at_tick: u64,
}
