//! Auxiliary base generator simulation component.

use bevy_ecs::component::Component;
use rustbound_protocol::MachineError;

/// ECS component modeling an auxiliary base power generator.
#[derive(Clone, Debug, Component)]
pub struct GeneratorComponent {
    pub is_ignited: bool,
    pub power_output_watts: u32,
}

impl Default for GeneratorComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl GeneratorComponent {
    pub fn new() -> Self {
        Self {
            is_ignited: false,
            power_output_watts: 0,
        }
    }

    /// Triggers generator ignition, bringing power online.
    pub fn ignite(&mut self) -> Result<(), MachineError> {
        self.is_ignited = true;
        self.power_output_watts = 50_000; // 50 kW nominal output
        Ok(())
    }
}
