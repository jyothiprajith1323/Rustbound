//! Wasmi 2.0 WebAssembly execution instance, fuel metering, and host state.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use rustbound_protocol::packets::{
    ConveyorFeedCommand, CraneTransferCommand, DrillExtractCommand, GeneratorIgniteCommand,
    StorageDepositCommand, TelemetryPacket,
};
use thiserror::Error;
use wasmi::{
    Config, Engine, Instance, Linker, Module, Store, TypedResumableCall,
    TypedResumableCallOutOfFuel,
};

#[derive(Error, Debug)]
pub enum VmError {
    #[error("failed to compile or parse wasm bytecode: {0}")]
    BytecodeValidation(#[from] wasmi::Error),
    #[error("firmware trapped during execution: {0}")]
    Trap(String),
    #[error("firmware entry point 'firmware_main' not exported")]
    MissingEntryPoint,
    #[error("host memory access violation")]
    MemoryAccessViolation,
}

/// Strongly typed actuator commands dispatched by guest firmware.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActuatorCommand {
    DrillExtract(DrillExtractCommand),
    ConveyorFeed(ConveyorFeedCommand),
    StorageDeposit(StorageDepositCommand),
    CraneTransfer(CraneTransferCommand),
    GeneratorIgnite(GeneratorIgniteCommand),
}

/// A validated command packet submitted by a specific machine entity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmittedCommand {
    pub machine_id: u32,
    pub command: ActuatorCommand,
}

/// Execution configuration parameters for a machine Wasmi virtual machine.
#[derive(Clone, Debug)]
pub struct VmConfig {
    /// Instruction fuel allocated per 20 Hz simulation tick.
    pub fuel_per_tick: u64,
    /// Maximum initial linear memory pages (1 page = 64 KB). 16 pages = 1 MB.
    pub initial_memory_pages: u32,
    /// Maximum linear memory pages allowed. 64 pages = 4 MB.
    pub max_memory_pages: u32,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            fuel_per_tick: 50_000,
            initial_memory_pages: 16,
            max_memory_pages: 64,
        }
    }
}

/// Result of executing a single 20 Hz tick slice in the virtual machine.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TickOutcome {
    /// Firmware completed execution or voluntarily called `yield_tick()`.
    Completed { fuel_consumed: u64 },
    /// Firmware exhausted its tick fuel budget (e.g. while in an unyielding loop).
    FuelExhausted,
    /// Firmware trapped (panic, divide-by-zero, illegal memory access).
    Trapped,
}

/// Host state shared with the guest VM during FFI callbacks.
#[derive(Default)]
pub struct HostState {
    pub machine_id: u32,
    pub yield_requested: bool,
    pub telemetry: BTreeMap<u32, TelemetryPacket>,
    pub submitted_commands: Vec<SubmittedCommand>,
    pub log_messages: Vec<(u32, String)>,
}

impl HostState {
    pub fn new(machine_id: u32) -> Self {
        Self {
            machine_id,
            yield_requested: false,
            telemetry: BTreeMap::new(),
            submitted_commands: Vec::new(),
            log_messages: Vec::new(),
        }
    }

    /// Sets or updates available telemetry for a machine on the bus.
    pub fn set_telemetry(&mut self, packet: TelemetryPacket) {
        self.telemetry.insert(packet.machine_id, packet);
    }
}

/// A running Wasmi virtual machine instance bound to a specific physical machine.
pub struct VmInstance {
    engine: Engine,
    store: Store<HostState>,
    instance: Instance,
    config: VmConfig,
    resumable_invocation: Option<TypedResumableCallOutOfFuel<()>>,
}

impl VmInstance {
    /// Returns a reference to the execution engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Compiles and instantiates a new Wasm firmware module with registered FFI imports.
    pub fn instantiate(
        machine_id: u32,
        wasm_bytes: &[u8],
        config: VmConfig,
    ) -> Result<Self, VmError> {
        let mut wasmi_config = Config::default();
        wasmi_config.consume_fuel(true);

        let engine = Engine::new(&wasmi_config);
        let module = Module::new(&engine, wasm_bytes)?;

        let host_state = HostState::new(machine_id);
        let mut store = Store::new(&engine, host_state);
        let mut linker = Linker::new(&engine);

        // Register the 4 standard host FFI imports
        crate::ffi::register_host_ffi(&mut linker)?;

        // Instantiate module with linker
        let instance = linker.instantiate_and_start(&mut store, &module)?;

        Ok(Self {
            engine,
            store,
            instance,
            config,
            resumable_invocation: None,
        })
    }

    /// Updates or inserts a machine's telemetry packet into the VM's readable bus cache.
    pub fn update_telemetry(&mut self, packet: TelemetryPacket) {
        self.store.data_mut().set_telemetry(packet);
    }

    /// Drains all commands submitted by the guest during the tick.
    pub fn drain_submitted_commands(&mut self) -> Vec<SubmittedCommand> {
        core::mem::take(&mut self.store.data_mut().submitted_commands)
    }

    /// Drains all terminal log messages emitted by the guest.
    pub fn drain_log_messages(&mut self) -> Vec<(u32, String)> {
        core::mem::take(&mut self.store.data_mut().log_messages)
    }

    /// Returns a reference to the internal host state.
    pub fn host_state(&self) -> &HostState {
        self.store.data()
    }

    /// Returns a mutable reference to the internal host state.
    pub fn host_state_mut(&mut self) -> &mut HostState {
        self.store.data_mut()
    }

    /// Executes the firmware for one 20 Hz simulation tick slice with fuel metering.
    pub fn execute_tick(&mut self) -> Result<TickOutcome, VmError> {
        // Reset tick yield flag
        self.store.data_mut().yield_requested = false;

        // Allocate fuel for this tick
        self.store.set_fuel(self.config.fuel_per_tick)?;

        let outcome = if let Some(invocation) = self.resumable_invocation.take() {
            invocation.resume(&mut self.store)
        } else {
            let main_func = self
                .instance
                .get_typed_func::<(), ()>(&self.store, "firmware_main")
                .map_err(|_| VmError::MissingEntryPoint)?;
            main_func.call_resumable(&mut self.store, ())
        };

        match outcome {
            Ok(TypedResumableCall::Finished(())) => {
                let remaining = self.store.get_fuel().unwrap_or(0);
                let consumed = self.config.fuel_per_tick.saturating_sub(remaining);
                Ok(TickOutcome::Completed {
                    fuel_consumed: consumed,
                })
            }
            Ok(TypedResumableCall::OutOfFuel(inv)) => {
                let remaining = self.store.get_fuel().unwrap_or(0);
                let consumed = self.config.fuel_per_tick.saturating_sub(remaining);
                if self.store.data().yield_requested {
                    self.resumable_invocation = Some(inv);
                    Ok(TickOutcome::Completed {
                        fuel_consumed: consumed,
                    })
                } else {
                    self.resumable_invocation = Some(inv);
                    Ok(TickOutcome::FuelExhausted)
                }
            }
            Ok(TypedResumableCall::HostTrap(_)) => {
                self.resumable_invocation = None;
                Ok(TickOutcome::Trapped)
            }
            Err(_) => {
                self.resumable_invocation = None;
                Ok(TickOutcome::Trapped)
            }
        }
    }
}
