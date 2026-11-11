use rustbound_protocol::MachineError;
use rustbound_protocol::packets::*;
use zerocopy::{FromBytes, IntoBytes};

#[test]
fn test_packet_header_validation() {
    let header = PacketHeader::new(commands::DRILL_EXTRACT);
    assert!(header.is_valid());
    assert_eq!(header.magic, [b'R', b'B']);
    assert_eq!(header.version, 1);
    assert_eq!(header.command_id, 10);
    assert_eq!(size_of::<PacketHeader>(), 8);
}

#[test]
fn test_drill_command_serialization() {
    let cmd = DrillExtractCommand::new(1, 2);
    let bytes = cmd.as_bytes();
    assert_eq!(bytes.len(), 16);

    let parsed = DrillExtractCommand::read_from_bytes(bytes).unwrap();
    assert_eq!(parsed.header.command_id, commands::DRILL_EXTRACT);
    assert_eq!(parsed.target_vein_id, 1);
    assert_eq!(parsed.power_setting, 2);
}

#[test]
fn test_conveyor_command_serialization() {
    let cmd = ConveyorFeedCommand::new(9999, 1);
    let bytes = cmd.as_bytes();
    assert_eq!(bytes.len(), 24);

    let parsed = ConveyorFeedCommand::read_from_bytes(bytes).unwrap();
    assert_eq!(parsed.item_id, 9999);
    assert_eq!(parsed.target_slot, 1);
}

#[test]
fn test_storage_command_serialization() {
    let cmd = StorageDepositCommand::new(12345, 0);
    let bytes = cmd.as_bytes();
    assert_eq!(bytes.len(), 24);

    let parsed = StorageDepositCommand::read_from_bytes(bytes).unwrap();
    assert_eq!(parsed.item_id, 12345);
    assert_eq!(parsed.bay_index, 0);
}

#[test]
fn test_crane_command_serialization() {
    let cmd = CraneTransferCommand::new(-10, 25, true);
    let bytes = cmd.as_bytes();
    assert_eq!(bytes.len(), 24);

    let parsed = CraneTransferCommand::read_from_bytes(bytes).unwrap();
    assert_eq!(parsed.target_x, -10);
    assert_eq!(parsed.target_y, 25);
    assert_eq!(parsed.grab, 1);
}

#[test]
fn test_telemetry_packet_serialization() {
    let telem = TelemetryPacket {
        machine_id: 1,
        machine_type: 2,
        status_flags: 0x01,
        temperature_celsius: 45,
        power_draw_watts: 1500,
        item_count: 3,
        next_item_id: 42,
        captured_at_tick: 100,
    };
    let bytes = telem.as_bytes();
    assert_eq!(bytes.len(), 40);

    let parsed = TelemetryPacket::read_from_bytes(bytes).unwrap();
    assert_eq!(parsed.machine_id, 1);
    assert_eq!(parsed.temperature_celsius, 45);
    assert_eq!(parsed.next_item_id, 42);
    assert_eq!(parsed.captured_at_tick, 100);
}

#[test]
fn test_machine_error_codes() {
    assert_eq!(MachineError::from_i32(0), MachineError::Success);
    assert_eq!(MachineError::from_i32(-1), MachineError::DeviceNotFound);
    assert_eq!(MachineError::from_i32(-2), MachineError::ThermalOverheat);
    assert_eq!(MachineError::from_i32(-3), MachineError::IntakeJammed);
    assert_eq!(MachineError::from_i32(-4), MachineError::StorageFull);
    assert_eq!(MachineError::from_i32(-5), MachineError::InsufficientPower);
    assert_eq!(MachineError::from_i32(-6), MachineError::CustodyViolation);
    assert_eq!(MachineError::from_i32(-999), MachineError::Unknown);
}
