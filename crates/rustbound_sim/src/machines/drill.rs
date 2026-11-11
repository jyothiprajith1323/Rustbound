//! Subterranean quarry excavator drill simulation component and cooling system.

use bevy_ecs::component::Component;
use rustbound_protocol::MachineError;

/// Unique machine identifier on the industrial bus.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Component)]
pub struct MachineId(pub u32);

/// ECS component modeling a heavy subterranean excavator drill.
#[derive(Clone, Debug, Component)]
pub struct QuarryDrillComponent {
    pub temperature_celsius: i32,
    pub ambient_temperature_celsius: i32,
    pub max_temperature_celsius: i32,
    pub heat_per_extraction: i32,
    pub cooldown_rate_per_tick: i32,
    pub total_extracted: u32,
    pub overheat_count: u32,
    pub is_overheated: bool,
    pub next_item_counter: u64,
}

impl QuarryDrillComponent {
    pub const DEFAULT_MAX_TEMP: i32 = 80;
    pub const DEFAULT_AMBIENT_TEMP: i32 = 20;

    pub fn new(initial_item_counter: u64) -> Self {
        Self {
            temperature_celsius: Self::DEFAULT_AMBIENT_TEMP,
            ambient_temperature_celsius: Self::DEFAULT_AMBIENT_TEMP,
            max_temperature_celsius: Self::DEFAULT_MAX_TEMP,
            heat_per_extraction: 5,
            cooldown_rate_per_tick: 2,
            total_extracted: 0,
            overheat_count: 0,
            is_overheated: false,
            next_item_counter: initial_item_counter,
        }
    }

    /// Triggers an extraction cycle, generating an ore item and thermal heat.
    pub fn trigger_extraction(&mut self) -> Result<u64, MachineError> {
        if self.is_overheated || self.temperature_celsius >= self.max_temperature_celsius {
            self.is_overheated = true;
            self.overheat_count = self.overheat_count.saturating_add(1);
            return Err(MachineError::ThermalOverheat);
        }

        self.temperature_celsius = self
            .temperature_celsius
            .saturating_add(self.heat_per_extraction);

        if self.temperature_celsius >= self.max_temperature_celsius {
            self.is_overheated = true;
            self.overheat_count = self.overheat_count.saturating_add(1);
        }

        self.total_extracted = self.total_extracted.saturating_add(1);
        self.next_item_counter = self.next_item_counter.saturating_add(1);
        let item_id = self.next_item_counter;

        Ok(item_id)
    }
}

/// Simulation system run at 20 Hz to cool idle drills toward ambient temperature.
pub fn drill_cooling_system(mut query: bevy_ecs::system::Query<&mut QuarryDrillComponent>) {
    for mut drill in &mut query {
        if drill.temperature_celsius > drill.ambient_temperature_celsius {
            drill.temperature_celsius = (drill.temperature_celsius - drill.cooldown_rate_per_tick)
                .max(drill.ambient_temperature_celsius);
        }

        // Recover from overheat once cooled below 65°C
        if drill.is_overheated && drill.temperature_celsius < 65 {
            drill.is_overheated = false;
        }
    }
}
