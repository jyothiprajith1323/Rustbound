//! Integration tests for Wasmi virtual machine execution, FFI imports, and custody tracking.

use rustbound_protocol::MachineError;
use rustbound_protocol::packets::{PacketHeader, TelemetryPacket};
use rustbound_runtime::{ActuatorCommand, CustodyTracker, TickOutcome, VmConfig, VmInstance};

#[test]
fn test_custody_tracker_transfers() {
    let mut tracker = CustodyTracker::new();
    let item_id = 1001;
    let drill_id = 1;
    let conveyor_id = 2;
    let storage_id = 3;

    // Register extracted item
    assert!(tracker.register_extracted(item_id, drill_id).is_ok());
    assert_eq!(tracker.current_holder(item_id), Some(drill_id));
    assert_eq!(tracker.total_active_items(), 1);

    // Cannot double-register the same token
    assert_eq!(
        tracker.register_extracted(item_id, drill_id),
        Err(MachineError::CustodyViolation)
    );

    // Transfer from drill to conveyor
    assert!(
        tracker
            .verify_and_transfer(item_id, drill_id, conveyor_id)
            .is_ok()
    );
    assert_eq!(tracker.current_holder(item_id), Some(conveyor_id));

    // Drill can no longer transfer it (custody moved!)
    assert_eq!(
        tracker.verify_and_transfer(item_id, drill_id, storage_id),
        Err(MachineError::CustodyViolation)
    );

    // Transfer from conveyor to storage
    assert!(
        tracker
            .verify_and_transfer(item_id, conveyor_id, storage_id)
            .is_ok()
    );
    assert_eq!(tracker.current_holder(item_id), Some(storage_id));

    // Consume item
    assert!(tracker.consume(item_id, storage_id).is_ok());
    assert_eq!(tracker.current_holder(item_id), None);
    assert_eq!(tracker.total_active_items(), 0);
}

#[test]
fn test_vm_telemetry_query_and_command_submit() {
    let wat = r#"
    (module
        (import "env" "host_query_telemetry" (func $query_telem (param i32 i32 i32) (result i32)))
        (import "env" "host_submit_command" (func $submit_cmd (param i32 i32 i32) (result i32)))
        (import "env" "host_yield_tick" (func $yield_tick))
        (memory (export "memory") 1 2)
        (data (i32.const 100) "\52\42\01\32\00\00\00\00\64\00\00\00\00\00\00\00") ;; GeneratorIgniteCommand (magic=RB, v=1, cmd=50, power=100)
        (func (export "firmware_main")
            ;; Query telemetry into address 0
            (drop (call $query_telem (i32.const 1) (i32.const 0) (i32.const 64)))
            ;; Submit command from address 100 (len 16)
            (drop (call $submit_cmd (i32.const 1) (i32.const 100) (i32.const 16)))
            ;; Relinquish tick
            (call $yield_tick)
        )
    )
    "#;

    let mut vm =
        VmInstance::instantiate(1, wat.as_bytes(), VmConfig::default()).expect("instantiate vm");

    let telem = TelemetryPacket {
        machine_id: 1,
        machine_type: 5,
        status_flags: 0,
        temperature_celsius: 25,
        power_draw_watts: 0,
        item_count: 0,
        next_item_id: 0,
        captured_at_tick: 1,
    };
    vm.update_telemetry(telem);

    let outcome = vm.execute_tick().expect("execute tick");
    assert!(matches!(outcome, TickOutcome::Completed { .. }));

    let commands = vm.drain_submitted_commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].machine_id, 1);
    match &commands[0].command {
        ActuatorCommand::GeneratorIgnite(cmd) => {
            assert_eq!(cmd.header.magic, PacketHeader::MAGIC);
            assert_eq!(cmd.power_level, 100);
        }
        _ => panic!("expected generator ignite command"),
    }
}

#[test]
fn test_vm_resumable_loop_with_yield() {
    let wat = r#"
    (module
        (import "env" "host_yield_tick" (func $yield_tick))
        (import "env" "host_log" (func $log (param i32 i32 i32)))
        (memory (export "memory") 1 2)
        (data (i32.const 0) "TickStep")
        (func (export "firmware_main")
            (loop $infinite
                ;; Emit log
                (call $log (i32.const 1) (i32.const 0) (i32.const 8))
                ;; Yield fuel for this tick
                (call $yield_tick)
                ;; Resume loop next tick
                (br $infinite)
            )
        )
    )
    "#;

    let mut vm =
        VmInstance::instantiate(1, wat.as_bytes(), VmConfig::default()).expect("instantiate vm");

    // Tick 1
    let outcome1 = vm.execute_tick().expect("tick 1");
    assert!(matches!(outcome1, TickOutcome::Completed { .. }));
    let logs1 = vm.drain_log_messages();
    assert_eq!(logs1.len(), 1);
    assert_eq!(logs1[0].1, "TickStep");

    // Tick 2: Should resume from inside the loop!
    let outcome2 = vm.execute_tick().expect("tick 2");
    assert!(matches!(outcome2, TickOutcome::Completed { .. }));
    let logs2 = vm.drain_log_messages();
    assert_eq!(logs2.len(), 1);
    assert_eq!(logs2[0].1, "TickStep");

    // Tick 3: Should resume again!
    let outcome3 = vm.execute_tick().expect("tick 3");
    assert!(matches!(outcome3, TickOutcome::Completed { .. }));
    let logs3 = vm.drain_log_messages();
    assert_eq!(logs3.len(), 1);
    assert_eq!(logs3[0].1, "TickStep");
}

#[test]
fn test_vm_fuel_exhaustion_on_unyielding_loop() {
    let wat = r#"
    (module
        (func (export "firmware_main")
            (loop $infinite
                (br $infinite)
            )
        )
    )
    "#;

    let mut vm = VmInstance::instantiate(
        1,
        wat.as_bytes(),
        VmConfig {
            fuel_per_tick: 1_000,
            ..Default::default()
        },
    )
    .expect("instantiate vm");

    let outcome = vm.execute_tick().expect("execute tick");
    assert_eq!(outcome, TickOutcome::FuelExhausted);
}

#[test]
fn test_vm_out_of_bounds_memory_protection() {
    let wat = r#"
    (module
        (import "env" "host_query_telemetry" (func $query_telem (param i32 i32 i32) (result i32)))
        (memory (export "memory") 1 1) ;; 1 page = 64KB (65536 bytes)
        (func (export "firmware_main")
            ;; Attempt out-of-bounds write beyond 64KB
            (drop (call $query_telem (i32.const 1) (i32.const 65500) (i32.const 100)))
        )
    )
    "#;

    let mut vm =
        VmInstance::instantiate(1, wat.as_bytes(), VmConfig::default()).expect("instantiate vm");

    let telem = TelemetryPacket {
        machine_id: 1,
        machine_type: 1,
        status_flags: 0,
        temperature_celsius: 20,
        power_draw_watts: 0,
        item_count: 0,
        next_item_id: 0,
        captured_at_tick: 0,
    };
    vm.update_telemetry(telem);

    let outcome = vm.execute_tick().expect("execute tick");
    assert!(matches!(outcome, TickOutcome::Completed { .. }));
}
