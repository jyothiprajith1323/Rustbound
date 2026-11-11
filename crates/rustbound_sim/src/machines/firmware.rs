//! Machine firmware execution component and ECS dispatch system.

use crate::machines::conveyor::ConveyorBeltComponent;
use crate::machines::drill::{MachineId, QuarryDrillComponent};
use crate::machines::generator::GeneratorComponent;
use crate::machines::storage::StorageBayComponent;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use bevy_ecs::component::Component;
use bevy_ecs::system::Query;
use rustbound_protocol::packets::TelemetryPacket;
use rustbound_runtime::{ActuatorCommand, SubmittedCommand, VmInstance};

/// ECS component holding the sandboxed Wasmi virtual machine running player firmware.
#[derive(Component)]
pub struct MachineFirmware {
    pub vm: Option<VmInstance>,
}

impl MachineFirmware {
    pub fn new(vm: VmInstance) -> Self {
        Self { vm: Some(vm) }
    }

    pub fn empty() -> Self {
        Self { vm: None }
    }
}

/// In-game terminal log buffer attached to a machine.
#[derive(Clone, Debug, Default, Component)]
pub struct MachineTerminalLog {
    pub entries: Vec<(u32, String)>,
}

impl MachineTerminalLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

/// Simulation system executing active firmware inside Wasmi instances at 20 Hz.
///
/// Dispatches telemetry to guest VMs, executes a metered fuel slice, and applies
/// all submitted actuator commands to the target ECS machine components.
pub fn firmware_execution_system(
    mut firmware_query: Query<(
        &MachineId,
        &mut MachineFirmware,
        Option<&mut MachineTerminalLog>,
    )>,
    mut drills: Query<(&MachineId, &mut QuarryDrillComponent)>,
    mut conveyors: Query<(&MachineId, &mut ConveyorBeltComponent)>,
    mut storages: Query<(&MachineId, &mut StorageBayComponent)>,
    mut generators: Query<(&MachineId, &mut GeneratorComponent)>,
) {
    // Fast path: avoid all telemetry collection if no active virtual machines exist
    let has_active_vm = firmware_query.iter().any(|(_, fw, _)| fw.vm.is_some());
    if !has_active_vm {
        return;
    }

    // 1. Collect telemetry bus snapshot from all machine components
    let mut bus_telemetry: BTreeMap<u32, TelemetryPacket> = BTreeMap::new();

    for (id, drill) in &drills {
        bus_telemetry.insert(
            id.0,
            TelemetryPacket {
                machine_id: id.0,
                machine_type: 1, // Drill
                status_flags: if drill.is_overheated { 4 } else { 1 },
                temperature_celsius: drill.temperature_celsius,
                power_draw_watts: 1500,
                item_count: drill.total_extracted,
                next_item_id: drill.next_item_counter,
                captured_at_tick: 0,
            },
        );
    }

    for (id, conveyor) in &conveyors {
        let occupied_count = conveyor.slots.iter().filter(|s| s.is_some()).count() as u32;
        let mut flags = 1u32;
        if conveyor.slots[0].is_some() || conveyor.is_jammed {
            flags |= 2; // Jammed / slot occupied
        }

        bus_telemetry.insert(
            id.0,
            TelemetryPacket {
                machine_id: id.0,
                machine_type: 2, // Conveyor
                status_flags: flags,
                temperature_celsius: 20,
                power_draw_watts: 300,
                item_count: occupied_count,
                next_item_id: conveyor.slots[crate::machines::conveyor::CONVEYOR_SLOT_COUNT - 1]
                    .unwrap_or(0),
                captured_at_tick: 0,
            },
        );
    }

    for (id, storage) in &storages {
        bus_telemetry.insert(
            id.0,
            TelemetryPacket {
                machine_id: id.0,
                machine_type: 3, // Storage
                status_flags: 1,
                temperature_celsius: 20,
                power_draw_watts: 50,
                item_count: storage.stored_count(),
                next_item_id: 0,
                captured_at_tick: 0,
            },
        );
    }

    for (id, generator) in &generators {
        bus_telemetry.insert(
            id.0,
            TelemetryPacket {
                machine_id: id.0,
                machine_type: 5, // Generator
                status_flags: if generator.is_ignited { 1 } else { 0 },
                temperature_celsius: if generator.is_ignited { 75 } else { 20 },
                power_draw_watts: generator.power_output_watts,
                item_count: 0,
                next_item_id: 0,
                captured_at_tick: 0,
            },
        );
    }

    // 2. Execute firmware for machines with an active VmInstance
    let mut pending_commands: Vec<SubmittedCommand> = Vec::new();

    for (_machine_id, mut firmware, mut opt_log) in &mut firmware_query {
        let Some(ref mut vm) = firmware.vm else {
            continue;
        };

        // Sync telemetry bus state
        for packet in bus_telemetry.values() {
            vm.update_telemetry(*packet);
        }

        // Execute 20 Hz slice
        let _outcome = vm.execute_tick();

        // Drain logs
        let logs = vm.drain_log_messages();
        if let Some(ref mut terminal_log) = opt_log {
            terminal_log.entries.extend(logs);
        }

        // Collect submitted commands
        let commands = vm.drain_submitted_commands();
        pending_commands.extend(commands);
    }

    // 3. Apply submitted actuator commands to ECS machine components
    for sub_cmd in pending_commands {
        match sub_cmd.command {
            ActuatorCommand::DrillExtract(_) => {
                for (id, mut drill) in &mut drills {
                    if id.0 == sub_cmd.machine_id {
                        let _ = drill.trigger_extraction();
                        break;
                    }
                }
            }
            ActuatorCommand::ConveyorFeed(feed_cmd) => {
                for (id, mut conveyor) in &mut conveyors {
                    if id.0 == sub_cmd.machine_id {
                        let _ = conveyor.feed(feed_cmd.item_id);
                        break;
                    }
                }
            }
            ActuatorCommand::StorageDeposit(dep_cmd) => {
                for (id, mut storage) in &mut storages {
                    if id.0 == sub_cmd.machine_id {
                        let _ = storage.deposit(dep_cmd.item_id);
                        break;
                    }
                }
            }
            ActuatorCommand::GeneratorIgnite(_) => {
                for (id, mut generator) in &mut generators {
                    if id.0 == sub_cmd.machine_id {
                        let _ = generator.ignite();
                        break;
                    }
                }
            }
            ActuatorCommand::CraneTransfer(_) => {
                // Crane transfer is handled in crane simulation
            }
        }
    }
}
