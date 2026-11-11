use rustbound_protocol::MachineError;
use rustbound_sim::grid::{Direction, SubTileProgress, TileCoord};
use rustbound_sim::machines::{
    ConveyorBeltComponent, MachineId, QuarryDrillComponent, StorageBayComponent,
};
use rustbound_sim::schedule::SimulationRunner;
use rustbound_sim::snapshot::{MachineSnapshot, SnapshotRingBuffer, WorldSnapshot};
use std::time::Instant;

#[test]
fn test_tile_coordinates_and_directions() {
    let origin = TileCoord::new(0, 0, 0);
    let north = origin.step(Direction::North);
    assert_eq!(north, TileCoord::new(0, 1, 0));

    let east = origin.step(Direction::East);
    assert_eq!(east, TileCoord::new(1, 0, 0));

    assert_eq!(origin.manhattan_distance(TileCoord::new(3, -4, 5)), 12);
}

#[test]
fn test_subtile_progress() {
    let mut progress = SubTileProgress::default();
    assert!(!progress.is_complete());

    progress.advance(32_768); // half tile
    assert!(!progress.is_complete());

    progress.advance(32_768); // full tile (65,536)
    assert!(progress.is_complete());

    progress.reset();
    assert_eq!(progress.0, 0);
}

#[test]
fn test_quarry_drill_extraction_and_cooling() {
    let mut drill = QuarryDrillComponent::new(100);
    assert_eq!(drill.temperature_celsius, 20);

    // Extract 5 times: 5 * 5°C = 25°C increase -> 45°C
    for _ in 0..5 {
        let res = drill.trigger_extraction();
        assert!(res.is_ok());
    }
    assert_eq!(drill.temperature_celsius, 45);
    assert_eq!(drill.total_extracted, 5);

    // Push past max temperature (80°C)
    for _ in 0..10 {
        let _ = drill.trigger_extraction();
    }
    assert!(drill.is_overheated);
    assert_eq!(
        drill.trigger_extraction(),
        Err(MachineError::ThermalOverheat)
    );
}

#[test]
fn test_conveyor_advancement_exit_to_intake() {
    let mut conveyor = ConveyorBeltComponent::new();
    assert!(conveyor.is_intake_clear());

    // Feed item 101 onto slot 0
    assert!(conveyor.feed(101).is_ok());
    assert!(!conveyor.is_intake_clear());

    // Advance 1 tick: slot 0 should move to slot 1, intake is clear again
    // (Simulate advancement logic directly)
    for i in (0..(rustbound_sim::CONVEYOR_SLOT_COUNT - 1)).rev() {
        if conveyor.slots[i + 1].is_none() && conveyor.slots[i].is_some() {
            conveyor.slots[i + 1] = conveyor.slots[i].take();
        }
    }

    assert_eq!(conveyor.slots[0], None);
    assert_eq!(conveyor.slots[1], Some(101));
    assert!(conveyor.is_intake_clear());

    // Feed second item 102 onto slot 0
    assert!(conveyor.feed(102).is_ok());

    // Advance another tick: slot 1 -> slot 2, slot 0 -> slot 1
    for i in (0..(rustbound_sim::CONVEYOR_SLOT_COUNT - 1)).rev() {
        if conveyor.slots[i + 1].is_none() && conveyor.slots[i].is_some() {
            conveyor.slots[i + 1] = conveyor.slots[i].take();
        }
    }
    assert_eq!(conveyor.slots[2], Some(101));
    assert_eq!(conveyor.slots[1], Some(102));
    assert_eq!(conveyor.slots[0], None);
}

#[test]
fn test_storage_bay_capacity() {
    let mut storage = StorageBayComponent::new(3);
    assert_eq!(storage.stored_count(), 0);

    assert!(storage.deposit(1).is_ok());
    assert!(storage.deposit(2).is_ok());
    assert!(storage.deposit(3).is_ok());
    assert_eq!(storage.stored_count(), 3);

    // Over-capacity deposit fails
    assert_eq!(storage.deposit(4), Err(MachineError::StorageFull));
}

#[test]
fn test_snapshot_ring_buffer_rewind() {
    let mut ring = SnapshotRingBuffer::new();
    assert!(ring.is_empty());

    // Push 75 snapshots; capacity is 60
    for tick in 0..75 {
        ring.push(WorldSnapshot {
            tick,
            machines: vec![MachineSnapshot {
                machine_id: 1,
                temperature_celsius: 25,
                is_jammed: false,
                is_overheated: false,
                item_count: 0,
            }],
        });
    }

    assert_eq!(ring.len(), 60);

    // Pop latest should yield tick 74
    let latest = ring.pop_latest().unwrap();
    assert_eq!(latest.tick, 74);
    assert_eq!(ring.len(), 59);

    // Pop next should yield tick 73
    let prev = ring.pop_latest().unwrap();
    assert_eq!(prev.tick, 73);
}

#[test]
fn test_headless_simulation_performance() {
    let mut runner = SimulationRunner::new();

    // Spawn 100 entities into the simulation
    for i in 0..100 {
        runner.world_mut().spawn((
            MachineId(i),
            QuarryDrillComponent::new((i as u64) * 1000),
            ConveyorBeltComponent::new(),
        ));
    }

    let start = Instant::now();
    runner.step_ticks(1_000); // 1,000 ticks = 50 seconds of simulated world time
    let elapsed = start.elapsed();

    assert_eq!(runner.current_tick(), 1_000);
    // 1,000 ticks across 100 entities in debug mode must execute under 500ms (real time is 50,000ms)
    assert!(
        elapsed.as_millis() < 500,
        "1,000 headless ticks took too long: {:?}",
        elapsed
    );
}
