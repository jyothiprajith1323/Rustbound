//! Error codes and types for machine communication across the FFI boundary.

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MachineError {
    /// Operation succeeded without error.
    Success = 0,
    /// Requested machine was not found on the local bus.
    DeviceNotFound = -1,
    /// Machine temperature exceeded safe operational threshold.
    ThermalOverheat = -2,
    /// Intake slot or conveyor path is obstructed.
    IntakeJammed = -3,
    /// Target storage container has reached capacity.
    StorageFull = -4,
    /// Power grid cannot supply sufficient voltage/current.
    InsufficientPower = -5,
    /// Physical custody token invalid or already transferred.
    CustodyViolation = -6,
    /// Malformed or unknown command packet.
    InvalidCommand = -7,
    /// Internal machine hardware fault.
    HardwareFault = -8,
    /// Unspecified or unknown error.
    Unknown = -255,
}

impl MachineError {
    /// Converts a raw i32 FFI status code into a MachineError.
    #[inline]
    pub const fn from_i32(val: i32) -> Self {
        match val {
            0 => Self::Success,
            -1 => Self::DeviceNotFound,
            -2 => Self::ThermalOverheat,
            -3 => Self::IntakeJammed,
            -4 => Self::StorageFull,
            -5 => Self::InsufficientPower,
            -6 => Self::CustodyViolation,
            -7 => Self::InvalidCommand,
            -8 => Self::HardwareFault,
            _ => Self::Unknown,
        }
    }

    /// Converts this error into its raw i32 FFI code.
    #[inline]
    pub const fn to_i32(self) -> i32 {
        self as i32
    }

    /// Returns a human-readable description of the error.
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Success => "operation succeeded",
            Self::DeviceNotFound => "machine not found on the bus",
            Self::ThermalOverheat => "machine temperature exceeded operational limit",
            Self::IntakeJammed => "machine intake slot is obstructed",
            Self::StorageFull => "target container is at maximum capacity",
            Self::InsufficientPower => "insufficient power grid supply",
            Self::CustodyViolation => "physical custody transfer violation",
            Self::InvalidCommand => "invalid or malformed command packet",
            Self::HardwareFault => "internal machine hardware fault",
            Self::Unknown => "unknown machine error",
        }
    }
}

impl core::fmt::Display for MachineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.description())
    }
}
