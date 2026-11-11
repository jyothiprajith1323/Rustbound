//! Mission progression definitions, objectives, and starter templates.

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionObjective {
    pub description: String,
    pub completed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionDefinition {
    pub id: u32,
    pub title: String,
    pub environment: String,
    pub briefing: String,
    pub starter_code: String,
    pub objectives: Vec<MissionObjective>,
}

impl MissionDefinition {
    /// Loads Mission 1: "Cold Boot"
    pub fn mission_1_cold_boot() -> Self {
        Self {
            id: 1,
            title: String::from("Mission 01: Cold Boot"),
            environment: String::from("Dropship Bay"),
            briefing: String::from(
                "Aethelgard's sub-zero atmosphere has drained our primary battery reserves. \
                 You must bind to the auxiliary generator (Machine #0) and execute an ignition sequence \
                 to bring the base power grid online.",
            ),
            starter_code: String::from(
                r#"#![no_std]
extern crate rustbound_api;
use rustbound_api::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn firmware_main() {
    // Bind to Auxiliary Generator (Machine ID: 0)
    let generator = Generator::bind(0).expect("failed to bind generator");

    // Ignite the generator if it is not already online
    if !generator.is_ignited() {
        generator.ignite().expect("failed to ignite generator");
        rustbound_api::log("Generator ignition sequence confirmed online");
    }

    rustbound_api::yield_tick();
}
"#,
            ),
            objectives: alloc::vec![
                MissionObjective {
                    description: String::from("Bind to Auxiliary Generator #0"),
                    completed: false,
                },
                MissionObjective {
                    description: String::from("Bring generator online (50 kW output)"),
                    completed: false,
                },
            ],
        }
    }

    /// Loads Mission 2: "Subterranean Extraction"
    pub fn mission_2_iron_extraction() -> Self {
        Self {
            id: 2,
            title: String::from("Mission 02: Subterranean Extraction"),
            environment: String::from("Rusted Quarry"),
            briefing: String::from(
                "Our fabrication facilities need raw materials. Deploy firmware to Excavator Drill #0 \
                 to extract 20 iron ore chunks. Watch the thermal core: if temperature reaches 80°C, \
                 the drill will suffer thermal shutdown. Call `yield_tick()` to allow the radiator to cool.",
            ),
            starter_code: String::from(
                r#"#![no_std]
extern crate rustbound_api;
use rustbound_api::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn firmware_main() {
    let drill = QuarryDrill::bind(0).expect("failed to bind drill");

    loop {
        // Safe thermal ceiling is 70°C (cutoff is 80°C)
        if drill.temperature() < 70 {
            if let Ok(_ore) = drill.extract() {
                rustbound_api::log("Ore chunk extracted");
            }
        }

        rustbound_api::yield_tick();
    }
}
"#,
            ),
            objectives: alloc::vec![
                MissionObjective {
                    description: String::from("Extract 20 standard iron ore chunks"),
                    completed: false,
                },
                MissionObjective {
                    description: String::from("Maintain drill core temperature below 80°C"),
                    completed: false,
                },
            ],
        }
    }

    /// Loads Mission 3: "Single Owner Custody"
    pub fn mission_3_single_owner_custody() -> Self {
        Self {
            id: 3,
            title: String::from("Mission 03: Single Owner Custody"),
            environment: String::from("Automated Processing Yard"),
            briefing: String::from(
                "Extract ore from Drill #0 and feed it onto Conveyor Belt #1 for automated transit \
                 into Storage Silo #2. Physical resources have affine custody: moving an ore token transfers \
                 custody permanently. Do not attempt to duplicate or double-feed the same item.",
            ),
            starter_code: String::from(
                r#"#![no_std]
extern crate rustbound_api;
use rustbound_api::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn firmware_main() {
    let drill = QuarryDrill::bind(0).expect("drill");
    let conveyor = ConveyorBelt::bind(1).expect("conveyor");

    loop {
        if conveyor.is_intake_clear() && drill.temperature() < 70 {
            if let Ok(ore) = drill.extract() {
                // Physical custody moves into conveyor.feed(ore)!
                let _ = conveyor.feed(ore);
            }
        }

        rustbound_api::yield_tick();
    }
}
"#,
            ),
            objectives: alloc::vec![
                MissionObjective {
                    description: String::from(
                        "Extract and route 10 ore chunks through conveyor intake"
                    ),
                    completed: false,
                },
                MissionObjective {
                    description: String::from("Resolve physical custody without borrow errors"),
                    completed: false,
                },
            ],
        }
    }

    /// Loads a mission by numeric ID (1..=3).
    pub fn load(id: u32) -> Option<Self> {
        match id {
            1 => Some(Self::mission_1_cold_boot()),
            2 => Some(Self::mission_2_iron_extraction()),
            3 => Some(Self::mission_3_single_owner_custody()),
            _ => None,
        }
    }
}
