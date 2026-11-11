//! Low-level host import function implementations registered with the Wasmi linker.

use crate::vm::{ActuatorCommand, HostState, SubmittedCommand};
use alloc::string::ToString;
use rustbound_protocol::MachineError;
use rustbound_protocol::packets::{
    ConveyorFeedCommand, CraneTransferCommand, DrillExtractCommand, GeneratorIgniteCommand,
    PacketHeader, StorageDepositCommand, commands,
};
use wasmi::{AsContextMut, Caller, Extern, Linker};
use zerocopy::{FromBytes, IntoBytes};

/// Registers all four standard host foreign function imports into the `"env"` module namespace.
pub fn register_host_ffi(linker: &mut Linker<HostState>) -> Result<(), wasmi::Error> {
    linker.func_wrap(
        "env",
        "host_query_telemetry",
        |mut caller: Caller<'_, HostState>, machine_id: u32, out_ptr: u32, max_len: u32| -> i32 {
            host_query_telemetry_impl(&mut caller, machine_id, out_ptr, max_len)
        },
    )?;

    linker.func_wrap(
        "env",
        "host_submit_command",
        |mut caller: Caller<'_, HostState>, machine_id: u32, cmd_ptr: u32, cmd_len: u32| -> i32 {
            host_submit_command_impl(&mut caller, machine_id, cmd_ptr, cmd_len)
        },
    )?;

    linker.func_wrap(
        "env",
        "host_yield_tick",
        |mut caller: Caller<'_, HostState>| {
            host_yield_tick_impl(&mut caller);
        },
    )?;

    linker.func_wrap(
        "env",
        "host_log",
        |mut caller: Caller<'_, HostState>, level: u32, msg_ptr: u32, msg_len: u32| {
            host_log_impl(&mut caller, level, msg_ptr, msg_len);
        },
    )?;

    Ok(())
}

fn host_query_telemetry_impl(
    caller: &mut Caller<'_, HostState>,
    machine_id: u32,
    out_ptr: u32,
    max_len: u32,
) -> i32 {
    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
        return MachineError::HardwareFault.to_i32();
    };

    let start = out_ptr as usize;
    let end = match start.checked_add(max_len as usize) {
        Some(e) => e,
        None => return MachineError::HardwareFault.to_i32(),
    };

    let (mem_slice, host_state) = memory.data_and_store_mut(caller);
    if end > mem_slice.len() {
        return MachineError::HardwareFault.to_i32();
    }

    let Some(packet) = host_state.telemetry.get(&machine_id) else {
        return MachineError::DeviceNotFound.to_i32();
    };

    let packet_bytes = packet.as_bytes();
    let copy_len = packet_bytes.len().min(max_len as usize);
    mem_slice[start..start + copy_len].copy_from_slice(&packet_bytes[..copy_len]);

    copy_len as i32
}

fn host_submit_command_impl(
    caller: &mut Caller<'_, HostState>,
    machine_id: u32,
    cmd_ptr: u32,
    cmd_len: u32,
) -> i32 {
    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
        return MachineError::HardwareFault.to_i32();
    };

    let start = cmd_ptr as usize;
    let end = match start.checked_add(cmd_len as usize) {
        Some(e) => e,
        None => return MachineError::InvalidCommand.to_i32(),
    };

    let (mem_slice, host_state) = memory.data_and_store_mut(caller);
    if end > mem_slice.len() {
        return MachineError::InvalidCommand.to_i32();
    }

    let cmd_slice = &mem_slice[start..end];
    if cmd_slice.len() < size_of::<PacketHeader>() {
        return MachineError::InvalidCommand.to_i32();
    }

    let Ok(header) = PacketHeader::read_from_bytes(&cmd_slice[..size_of::<PacketHeader>()]) else {
        return MachineError::InvalidCommand.to_i32();
    };

    if !header.is_valid() {
        return MachineError::InvalidCommand.to_i32();
    }

    match header.command_id {
        commands::DRILL_EXTRACT => {
            let Ok(cmd) = DrillExtractCommand::read_from_bytes(cmd_slice) else {
                return MachineError::InvalidCommand.to_i32();
            };

            // Update telemetry anticipating extraction
            if let Some(telem) = host_state.telemetry.get_mut(&machine_id) {
                if telem.temperature_celsius >= 80 {
                    return MachineError::ThermalOverheat.to_i32();
                }
                telem.next_item_id = telem.next_item_id.saturating_add(1);
                telem.item_count = telem.item_count.saturating_add(1);
            }

            host_state.submitted_commands.push(SubmittedCommand {
                machine_id,
                command: ActuatorCommand::DrillExtract(cmd),
            });

            MachineError::Success.to_i32()
        }
        commands::CONVEYOR_FEED => {
            let Ok(cmd) = ConveyorFeedCommand::read_from_bytes(cmd_slice) else {
                return MachineError::InvalidCommand.to_i32();
            };

            if let Some(telem) = host_state.telemetry.get_mut(&machine_id) {
                if (telem.status_flags & 0x02) != 0 {
                    return MachineError::IntakeJammed.to_i32();
                }
                telem.status_flags |= 0x02; // Mark intake slot occupied
                telem.item_count = telem.item_count.saturating_add(1);
            }

            host_state.submitted_commands.push(SubmittedCommand {
                machine_id,
                command: ActuatorCommand::ConveyorFeed(cmd),
            });

            MachineError::Success.to_i32()
        }
        commands::STORAGE_DEPOSIT => {
            let Ok(cmd) = StorageDepositCommand::read_from_bytes(cmd_slice) else {
                return MachineError::InvalidCommand.to_i32();
            };

            if let Some(telem) = host_state.telemetry.get_mut(&machine_id) {
                telem.item_count = telem.item_count.saturating_add(1);
            }

            host_state.submitted_commands.push(SubmittedCommand {
                machine_id,
                command: ActuatorCommand::StorageDeposit(cmd),
            });

            MachineError::Success.to_i32()
        }
        commands::CRANE_TRANSFER => {
            let Ok(cmd) = CraneTransferCommand::read_from_bytes(cmd_slice) else {
                return MachineError::InvalidCommand.to_i32();
            };

            host_state.submitted_commands.push(SubmittedCommand {
                machine_id,
                command: ActuatorCommand::CraneTransfer(cmd),
            });

            MachineError::Success.to_i32()
        }
        commands::GENERATOR_IGNITE => {
            let Ok(cmd) = GeneratorIgniteCommand::read_from_bytes(cmd_slice) else {
                return MachineError::InvalidCommand.to_i32();
            };

            if let Some(telem) = host_state.telemetry.get_mut(&machine_id) {
                telem.status_flags |= 0x01; // running
                telem.power_draw_watts = 50_000;
            }

            host_state.submitted_commands.push(SubmittedCommand {
                machine_id,
                command: ActuatorCommand::GeneratorIgnite(cmd),
            });

            MachineError::Success.to_i32()
        }
        _ => MachineError::InvalidCommand.to_i32(),
    }
}

fn host_yield_tick_impl(caller: &mut Caller<'_, HostState>) {
    caller.data_mut().yield_requested = true;
    let _ = caller.as_context_mut().set_fuel(0);
}

fn host_log_impl(caller: &mut Caller<'_, HostState>, level: u32, msg_ptr: u32, msg_len: u32) {
    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
        return;
    };

    let start = msg_ptr as usize;
    let end = match start.checked_add(msg_len as usize) {
        Some(e) => e,
        None => return,
    };

    let (mem_slice, host_state) = memory.data_and_store_mut(caller);
    if end <= mem_slice.len() {
        let msg = match core::str::from_utf8(&mem_slice[start..end]) {
            Ok(s) => s.to_string(),
            Err(_) => alloc::string::String::from("<invalid utf-8>"),
        };
        host_state.log_messages.push((level, msg));
    }
}
