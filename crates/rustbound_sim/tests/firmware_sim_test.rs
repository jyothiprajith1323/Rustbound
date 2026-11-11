//! Integration tests verifying that WebAssembly guest firmware successfully drives
//! physical machine actuators inside the deterministic Bevy ECS simulation runner.

use rustbound_runtime::{VmConfig, VmInstance};
use rustbound_sim::{
    GeneratorComponent, MachineFirmware, MachineId, MachineTerminalLog, QuarryDrillComponent,
    SimulationRunner,
};

#[test]
fn test_firmware_controls_generator_in_simulation() {
    let mut runner = SimulationRunner::new();

    // Wasm program that binds to generator 1 and issues GeneratorIgniteCommand
    let wat = r#"
    (module
        (import "env" "host_query_telemetry" (func $query (param i32 i32 i32) (result i32)))
        (import "env" "host_submit_command" (func $submit (param i32 i32 i32) (result i32)))
        (import "env" "host_yield_tick" (func $yield))
        (memory (export "memory") 1 2)
        (data (i32.const 100) "\52\42\01\32\00\00\00\00\64\00\00\00\00\00\00\00") ;; GeneratorIgniteCommand (cmd=50)
        (func (export "firmware_main")
            (drop (call $query (i32.const 1) (i32.const 0) (i32.const 64)))
            (drop (call $submit (i32.const 1) (i32.const 100) (i32.const 16)))
            (call $yield)
        )
    )
    "#;

    let vm =
        VmInstance::instantiate(1, wat.as_bytes(), VmConfig::default()).expect("instantiate vm");

    let entity = runner
        .world_mut()
        .spawn((
            MachineId(1),
            GeneratorComponent::new(),
            MachineFirmware::new(vm),
        ))
        .id();

    // Verify initial unignited state
    let gen_before = runner.world().get::<GeneratorComponent>(entity).unwrap();
    assert!(!gen_before.is_ignited);
    assert_eq!(gen_before.power_output_watts, 0);

    // Step simulation by 1 tick (50ms)
    runner.step_tick();

    // Verify generator has been ignited by guest firmware execution
    let gen_after = runner.world().get::<GeneratorComponent>(entity).unwrap();
    assert!(gen_after.is_ignited);
    assert_eq!(gen_after.power_output_watts, 50_000);
}

#[test]
fn test_firmware_controls_quarry_drill_extraction() {
    let mut runner = SimulationRunner::new();

    // Wasm program that submits DrillExtractCommand and logs a message
    let wat = r#"
    (module
        (import "env" "host_submit_command" (func $submit (param i32 i32 i32) (result i32)))
        (import "env" "host_log" (func $log (param i32 i32 i32)))
        (import "env" "host_yield_tick" (func $yield))
        (memory (export "memory") 1 2)
        ;; DrillExtractCommand: magic=RB (0x52, 0x42), version=1, cmd=10 (0x0A), reserved=0, target_vein=0, power=2
        (data (i32.const 100) "\52\42\01\0a\00\00\00\00\00\00\00\00\02\00\00\00")
        (data (i32.const 200) "DrillEngaged")
        (func (export "firmware_main")
            (drop (call $submit (i32.const 2) (i32.const 100) (i32.const 16)))
            (call $log (i32.const 1) (i32.const 200) (i32.const 12))
            (call $yield)
        )
    )
    "#;

    let vm =
        VmInstance::instantiate(2, wat.as_bytes(), VmConfig::default()).expect("instantiate vm");

    let entity = runner
        .world_mut()
        .spawn((
            MachineId(2),
            QuarryDrillComponent::new(100),
            MachineFirmware::new(vm),
            MachineTerminalLog::new(),
        ))
        .id();

    // Verify initial state
    let drill_before = runner.world().get::<QuarryDrillComponent>(entity).unwrap();
    assert_eq!(drill_before.total_extracted, 0);
    assert_eq!(drill_before.temperature_celsius, 20);

    // Step 1 tick
    runner.step_tick();

    // Verify drill extracted 1 chunk and heat increased
    let drill_after = runner.world().get::<QuarryDrillComponent>(entity).unwrap();
    assert_eq!(drill_after.total_extracted, 1);
    // Initial temp 20 + 5 (heat) - 2 (cooling) = 23
    assert_eq!(drill_after.temperature_celsius, 23);

    // Verify terminal log was recorded
    let terminal_log = runner.world().get::<MachineTerminalLog>(entity).unwrap();
    assert_eq!(terminal_log.entries.len(), 1);
    assert_eq!(terminal_log.entries[0].1, "DrillEngaged");
}

#[test]
fn test_firmware_loop_continuous_extraction_and_cooldown() {
    let mut runner = SimulationRunner::new();

    // Wasm program with loop: extract -> yield -> loop
    let wat = r#"
    (module
        (import "env" "host_submit_command" (func $submit (param i32 i32 i32) (result i32)))
        (import "env" "host_yield_tick" (func $yield))
        (memory (export "memory") 1 2)
        (data (i32.const 100) "\52\42\01\0a\00\00\00\00\00\00\00\00\02\00\00\00")
        (func (export "firmware_main")
            (loop $extract_loop
                (drop (call $submit (i32.const 3) (i32.const 100) (i32.const 16)))
                (call $yield)
                (br $extract_loop)
            )
        )
    )
    "#;

    let vm =
        VmInstance::instantiate(3, wat.as_bytes(), VmConfig::default()).expect("instantiate vm");

    let entity = runner
        .world_mut()
        .spawn((
            MachineId(3),
            QuarryDrillComponent::new(500),
            MachineFirmware::new(vm),
        ))
        .id();

    // Advance 5 ticks
    runner.step_ticks(5);

    let drill = runner.world().get::<QuarryDrillComponent>(entity).unwrap();
    assert_eq!(drill.total_extracted, 5);
    assert_eq!(drill.next_item_counter, 505);
}
