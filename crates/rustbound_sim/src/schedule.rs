//! Fixed 20 Hz simulation schedule and world runner.

use crate::machines::{conveyor_advancement_system, drill_cooling_system};
use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;

/// High-level runner encapsulating the deterministic simulation world and 20 Hz schedule.
pub struct SimulationRunner {
    world: World,
    schedule: Schedule,
    tick_count: u64,
}

impl Default for SimulationRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationRunner {
    pub fn new() -> Self {
        let world = World::new();
        let mut schedule = Schedule::default();

        // Register core simulation systems in topological order
        schedule.add_systems((drill_cooling_system, conveyor_advancement_system));

        Self {
            world,
            schedule,
            tick_count: 0,
        }
    }

    /// Access the underlying ECS world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Mutably access the underlying ECS world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Returns the current total simulation tick count.
    #[inline]
    pub fn current_tick(&self) -> u64 {
        self.tick_count
    }

    /// Advances the simulation by exactly one 50ms tick (20 Hz).
    pub fn step_tick(&mut self) {
        self.schedule.run(&mut self.world);
        self.tick_count = self.tick_count.saturating_add(1);
    }

    /// Runs the simulation for `n` discrete ticks in a tight deterministic loop.
    pub fn step_ticks(&mut self, n: u64) {
        for _ in 0..n {
            self.step_tick();
        }
    }
}
