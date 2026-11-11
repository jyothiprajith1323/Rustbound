//! Wasmi 2.0 WebAssembly execution instance and store management.

use thiserror::Error;
use wasmi::{Config, Engine, Instance, Linker, Module, Store};

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
    /// Firmware completed execution or called `host_yield_tick()`.
    Completed { fuel_consumed: u64 },
    /// Firmware exhausted its tick fuel budget (e.g. while in a loop).
    FuelExhausted,
    /// Firmware trapped (panic, divide-by-zero, illegal memory access).
    Trapped,
}

/// Host state shared with the guest VM during FFI callbacks.
#[derive(Default)]
pub struct HostState {
    pub machine_id: u32,
    pub yield_requested: bool,
    pub last_submitted_command: Option<u8>,
}

/// A running Wasmi virtual machine instance bound to a specific physical machine.
pub struct VmInstance {
    engine: Engine,
    store: Store<HostState>,
    instance: Instance,
    config: VmConfig,
}

impl VmInstance {
    /// Returns a reference to the execution engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Compiles and instantiates a new Wasm firmware module.
    pub fn instantiate(
        machine_id: u32,
        wasm_bytes: &[u8],
        config: VmConfig,
    ) -> Result<Self, VmError> {
        let mut wasmi_config = Config::default();
        wasmi_config.consume_fuel(true);

        let engine = Engine::new(&wasmi_config);
        let module = Module::new(&engine, wasm_bytes)?;

        let host_state = HostState {
            machine_id,
            yield_requested: false,
            last_submitted_command: None,
        };

        let mut store = Store::new(&engine, host_state);
        let linker = Linker::new(&engine);

        // Instantiate module with linker
        let instance = linker.instantiate_and_start(&mut store, &module)?;

        Ok(Self {
            engine,
            store,
            instance,
            config,
        })
    }

    /// Executes the firmware for one 20 Hz simulation tick slice with fuel metering.
    pub fn execute_tick(&mut self) -> Result<TickOutcome, VmError> {
        // Reset tick yield flag
        self.store.data_mut().yield_requested = false;

        // Allocate fuel for this tick
        self.store.set_fuel(self.config.fuel_per_tick)?;

        // Find firmware_main entrypoint
        let main_func = self
            .instance
            .get_typed_func::<(), ()>(&self.store, "firmware_main")
            .map_err(|_| VmError::MissingEntryPoint)?;

        // Invoke firmware_main
        match main_func.call(&mut self.store, ()) {
            Ok(()) => {
                let remaining_fuel = self.store.get_fuel().unwrap_or(0);
                let consumed = self.config.fuel_per_tick.saturating_sub(remaining_fuel);
                Ok(TickOutcome::Completed {
                    fuel_consumed: consumed,
                })
            }
            Err(err) => {
                // Check if trap was due to fuel exhaustion
                let err_str = alloc::format!("{err}");
                if err_str.contains("fuel") || err_str.contains("Fuel") {
                    Ok(TickOutcome::FuelExhausted)
                } else {
                    Ok(TickOutcome::Trapped)
                }
            }
        }
    }
}
