//! Central interactive game session managing simulation, editor, compiler, and missions.

use crate::dock::{RustboundTab, create_default_dock_layout};
use crate::editor::CodeEditorState;
use crate::inspector::InspectorState;
use crate::missions::MissionDefinition;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use egui_dock::DockState;
use rustbound_compiler::worker::CompilerWorker;
use rustbound_protocol::packets::TelemetryPacket;
use rustbound_runtime::{VmConfig, VmInstance};
use rustbound_sim::{
    CONVEYOR_SLOT_COUNT, ConveyorBeltComponent, GeneratorComponent, MachineFirmware, MachineId,
    MachineTerminalLog, QuarryDrillComponent, SimulationRunner, StorageBayComponent,
};

/// Complete game session state tying presentation, simulation, and compiler.
pub struct RustboundSession {
    pub sim_runner: SimulationRunner,
    pub editor: CodeEditorState,
    pub inspector: InspectorState,
    pub dock_state: DockState<RustboundTab>,
    pub mission: MissionDefinition,
    pub compiler: CompilerWorker,
    pub terminal_logs: Vec<(u32, String)>,
}

impl Default for RustboundSession {
    fn default() -> Self {
        Self::new(1)
    }
}

impl RustboundSession {
    /// Creates a new session initialized to the specified mission.
    pub fn new(mission_id: u32) -> Self {
        let mut session = Self {
            sim_runner: SimulationRunner::new(),
            editor: CodeEditorState::new(),
            inspector: InspectorState::new(),
            dock_state: create_default_dock_layout(),
            mission: MissionDefinition::load(mission_id)
                .unwrap_or_else(MissionDefinition::mission_1_cold_boot),
            compiler: CompilerWorker::default(),
            terminal_logs: Vec::new(),
        };

        session.setup_mission(mission_id);
        session
    }

    /// Sets up the ECS world and starter code for a specific mission.
    pub fn setup_mission(&mut self, mission_id: u32) {
        if let Some(def) = MissionDefinition::load(mission_id) {
            self.mission = def;
        }

        self.editor = CodeEditorState::from_str(&self.mission.starter_code);
        self.sim_runner = SimulationRunner::new();
        self.terminal_logs.clear();

        match mission_id {
            1 => {
                // Mission 1: Auxiliary Generator (ID: 0)
                self.sim_runner.world_mut().spawn((
                    MachineId(0),
                    GeneratorComponent::new(),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.inspector.select_machine(0);
            }
            2 => {
                // Mission 2: Quarry Excavator Drill (ID: 0)
                self.sim_runner.world_mut().spawn((
                    MachineId(0),
                    QuarryDrillComponent::new(100),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.inspector.select_machine(0);
            }
            3 => {
                // Mission 3: Drill (ID: 0), Conveyor (ID: 1), Storage Bay (ID: 2)
                self.sim_runner.world_mut().spawn((
                    MachineId(0),
                    QuarryDrillComponent::new(500),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.sim_runner.world_mut().spawn((
                    MachineId(1),
                    ConveyorBeltComponent::new(),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.sim_runner.world_mut().spawn((
                    MachineId(2),
                    StorageBayComponent::new(100),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.inspector.select_machine(0);
            }
            4 => {
                // Mission 4: Excavator Drill (ID: 0)
                self.sim_runner.world_mut().spawn((
                    MachineId(0),
                    QuarryDrillComponent::new(500),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.inspector.select_machine(0);
            }
            5 => {
                // Mission 5: Thermal Governor (ID: 0)
                self.sim_runner.world_mut().spawn((
                    MachineId(0),
                    QuarryDrillComponent::new(500),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.inspector.select_machine(0);
            }
            6 => {
                // Mission 6: Drill (ID: 0), Conveyor (ID: 1), Storage Bay (ID: 2)
                self.sim_runner.world_mut().spawn((
                    MachineId(0),
                    QuarryDrillComponent::new(500),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.sim_runner.world_mut().spawn((
                    MachineId(1),
                    ConveyorBeltComponent::new(),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.sim_runner.world_mut().spawn((
                    MachineId(2),
                    StorageBayComponent::new(100),
                    MachineFirmware::empty(),
                    MachineTerminalLog::new(),
                ));
                self.inspector.select_machine(0);
            }
            _ => {}
        }
    }

    /// Advances the simulation by 1 tick (50ms at 20 Hz) and updates mission objectives.
    pub fn step_simulation(&mut self) {
        self.sim_runner.step_tick();

        // Collect new terminal logs from machines
        let mut query = self
            .sim_runner
            .world_mut()
            .query::<(&MachineId, &mut MachineTerminalLog)>();
        for (_id, mut log) in query.iter_mut(self.sim_runner.world_mut()) {
            if !log.entries.is_empty() {
                self.terminal_logs.extend(core::mem::take(&mut log.entries));
            }
        }

        self.evaluate_objectives();
    }

    /// Evaluates mission completion conditions.
    pub fn evaluate_objectives(&mut self) {
        match self.mission.id {
            1 => {
                // Objective: Generator ignited
                let mut query = self
                    .sim_runner
                    .world_mut()
                    .query::<(&MachineId, &GeneratorComponent)>();
                for (id, generator_comp) in query.iter(self.sim_runner.world()) {
                    if id.0 == 0 && generator_comp.is_ignited {
                        self.mission.objectives[0].completed = true;
                        self.mission.objectives[1].completed = true;
                    }
                }
            }
            2 => {
                // Objective: 20 extractions without exceeding 80°C
                let mut query = self
                    .sim_runner
                    .world_mut()
                    .query::<(&MachineId, &QuarryDrillComponent)>();
                for (id, drill) in query.iter(self.sim_runner.world()) {
                    if id.0 == 0 {
                        if drill.total_extracted >= 20 {
                            self.mission.objectives[0].completed = true;
                        }
                        if drill.temperature_celsius < 80 && !drill.is_overheated {
                            self.mission.objectives[1].completed = true;
                        }
                    }
                }
            }
            3 => {
                // Objective: 10 items stored
                let mut query = self
                    .sim_runner
                    .world_mut()
                    .query::<(&MachineId, &StorageBayComponent)>();
                for (id, storage) in query.iter(self.sim_runner.world()) {
                    if id.0 == 2 && storage.stored_count() >= 10 {
                        self.mission.objectives[0].completed = true;
                    }
                }
                if self
                    .editor
                    .diagnostics()
                    .iter()
                    .all(|d| !d.is_borrow_error())
                {
                    self.mission.objectives[1].completed = true;
                }
            }
            4 => {
                // Mission 4: 25 extractions across deepening stratum
                let mut query = self
                    .sim_runner
                    .world_mut()
                    .query::<(&MachineId, &QuarryDrillComponent)>();
                for (id, drill) in query.iter(self.sim_runner.world()) {
                    if id.0 == 0 {
                        self.mission.objectives[0].completed = true;
                        if drill.total_extracted >= 25 {
                            self.mission.objectives[1].completed = true;
                        }
                    }
                }
            }
            5 => {
                // Mission 5: 30 extractions while maintaining temperature < 80°C
                let mut query = self
                    .sim_runner
                    .world_mut()
                    .query::<(&MachineId, &QuarryDrillComponent)>();
                for (id, drill) in query.iter(self.sim_runner.world()) {
                    if id.0 == 0 {
                        if drill.temperature_celsius < 80 && !drill.is_overheated {
                            self.mission.objectives[0].completed = true;
                        }
                        if drill.total_extracted >= 30 {
                            self.mission.objectives[1].completed = true;
                        }
                    }
                }
            }
            6 => {
                // Mission 6: 15 items routed into storage
                let mut query = self
                    .sim_runner
                    .world_mut()
                    .query::<(&MachineId, &StorageBayComponent)>();
                for (id, storage) in query.iter(self.sim_runner.world()) {
                    if id.0 == 2 {
                        self.mission.objectives[0].completed = true;
                        if storage.stored_count() >= 15 {
                            self.mission.objectives[1].completed = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Returns `true` if all objectives of the current mission have been fulfilled.
    pub fn is_mission_completed(&self) -> bool {
        self.mission.objectives.iter().all(|obj| obj.completed)
    }

    /// Deploys compiled WebAssembly firmware bytes to the targeted machine.
    pub fn deploy_firmware_bytes(
        &mut self,
        target_machine_id: u32,
        wasm_bytes: &[u8],
    ) -> Result<(), String> {
        let vm = VmInstance::instantiate(target_machine_id, wasm_bytes, VmConfig::default())
            .map_err(|e| alloc::format!("failed to instantiate VM: {e}"))?;

        let mut query = self
            .sim_runner
            .world_mut()
            .query::<(&MachineId, &mut MachineFirmware)>();

        for (id, mut fw) in query.iter_mut(self.sim_runner.world_mut()) {
            if id.0 == target_machine_id {
                fw.vm = Some(vm);
                return Ok(());
            }
        }

        Err(alloc::format!(
            "target machine #{target_machine_id} not found"
        ))
    }

    /// Collects a current telemetry snapshot of all active machines in the world.
    pub fn collect_telemetry_snapshot(&mut self) -> BTreeMap<u32, TelemetryPacket> {
        let mut map = BTreeMap::new();
        let tick = self.sim_runner.current_tick();

        let mut drills = self
            .sim_runner
            .world_mut()
            .query::<(&MachineId, &QuarryDrillComponent)>();
        for (id, drill) in drills.iter(self.sim_runner.world()) {
            map.insert(
                id.0,
                TelemetryPacket {
                    machine_id: id.0,
                    machine_type: 1,
                    status_flags: if drill.is_overheated { 4 } else { 1 },
                    temperature_celsius: drill.temperature_celsius,
                    power_draw_watts: 1500,
                    item_count: drill.total_extracted,
                    next_item_id: drill.next_item_counter,
                    captured_at_tick: tick,
                },
            );
        }

        let mut conveyors = self
            .sim_runner
            .world_mut()
            .query::<(&MachineId, &ConveyorBeltComponent)>();
        for (id, conveyor) in conveyors.iter(self.sim_runner.world()) {
            let count = conveyor.slots.iter().filter(|s| s.is_some()).count() as u32;
            map.insert(
                id.0,
                TelemetryPacket {
                    machine_id: id.0,
                    machine_type: 2,
                    status_flags: if conveyor.is_jammed || conveyor.slots[0].is_some() {
                        2
                    } else {
                        1
                    },
                    temperature_celsius: 20,
                    power_draw_watts: 300,
                    item_count: count,
                    next_item_id: conveyor.slots[CONVEYOR_SLOT_COUNT - 1].unwrap_or(0),
                    captured_at_tick: tick,
                },
            );
        }

        let mut storages = self
            .sim_runner
            .world_mut()
            .query::<(&MachineId, &StorageBayComponent)>();
        for (id, storage) in storages.iter(self.sim_runner.world()) {
            map.insert(
                id.0,
                TelemetryPacket {
                    machine_id: id.0,
                    machine_type: 3,
                    status_flags: 1,
                    temperature_celsius: 20,
                    power_draw_watts: 50,
                    item_count: storage.stored_count(),
                    next_item_id: 0,
                    captured_at_tick: tick,
                },
            );
        }

        let mut generators = self
            .sim_runner
            .world_mut()
            .query::<(&MachineId, &GeneratorComponent)>();
        for (id, generator_comp) in generators.iter(self.sim_runner.world()) {
            map.insert(
                id.0,
                TelemetryPacket {
                    machine_id: id.0,
                    machine_type: 5,
                    status_flags: if generator_comp.is_ignited { 1 } else { 0 },
                    temperature_celsius: if generator_comp.is_ignited { 75 } else { 20 },
                    power_draw_watts: generator_comp.power_output_watts,
                    item_count: 0,
                    next_item_id: 0,
                    captured_at_tick: tick,
                },
            );
        }

        map
    }

    /// Compiles the current editor code using real rustc targeting wasm32-unknown-unknown.
    pub fn compile_current_firmware(&mut self) -> Result<Vec<u8>, String> {
        let temp_dir = std::env::temp_dir().join("rustbound_build");
        let _ = std::fs::create_dir_all(&temp_dir);
        let src_path = temp_dir.join("player_firmware.rs");
        let out_wasm = temp_dir.join("player_firmware.wasm");

        let code = self.editor.text();
        if let Err(e) = std::fs::write(&src_path, &code) {
            return Err(alloc::format!("failed to write firmware scratch file: {e}"));
        }

        let root = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("/home/prajith/Rustbound"));
        let rlib = root.join("target/wasm32-unknown-unknown/release/librustbound_api.rlib");
        if rlib.exists() {
            self.compiler.sdk_rlib_path = rlib;
        }
        self.compiler.search_paths.clear();
        self.compiler
            .add_search_path(root.join("target/wasm32-unknown-unknown/release/deps"));
        self.compiler
            .add_search_path(root.join("target/release/deps"));

        let res = self
            .compiler
            .compile(&src_path, &out_wasm)
            .map_err(|e| alloc::format!("compiler execution error: {e}"))?;

        self.editor.set_diagnostics(res.diagnostics);

        if res.success {
            res.wasm_bytes
                .ok_or_else(|| String::from("compiler succeeded but emitted no wasm binary"))
        } else {
            Err(String::from(
                "compilation failed with errors (inspect Diagnostics tab)",
            ))
        }
    }
}
