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

    /// Loads Mission 4: "Immutable Quarry Interlock"
    pub fn mission_4_immutable_quarry_interlock() -> Self {
        Self {
            id: 4,
            title: String::from("Mission 04: Immutable Interlock"),
            environment: String::from("Subsurface Borehole"),
            briefing: String::from(
                "Excavator Drill #0's depth stepper controller is locked in an immutable register. \
                 In Rust, variables are immutable by default. Declare `let mut depth_step = 0;` to \
                 accumulate depth iterations, step the drill down, and extract 25 ore chunks.",
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
    
    // In Rust, variables are immutable by default.
    // Use `let mut` to allow variable re-assignment:
    let mut depth_step = 0;

    loop {
        if drill.temperature() < 70 {
            if let Ok(_ore) = drill.extract() {
                depth_step += 1;
                rustbound_api::log("Drill advanced to next stratum");
            }
        }

        rustbound_api::yield_tick();
    }
}
"#,
            ),
            objectives: alloc::vec![
                MissionObjective {
                    description: String::from("Declare mutable depth counter with `let mut`"),
                    completed: false,
                },
                MissionObjective {
                    description: String::from("Extract 25 ore chunks across deepening stratum"),
                    completed: false,
                },
            ],
        }
    }

    /// Loads Mission 5: "Thermal Governor"
    pub fn mission_5_thermal_governor() -> Self {
        Self {
            id: 5,
            title: String::from("Mission 05: Thermal Governor"),
            environment: String::from("Geothermal Smelter Flume"),
            briefing: String::from(
                "Write an expression-based helper function `fn should_cool(temp: i32) -> bool` \
                 that returns `true` when temperature exceeds 65°C without trailing semicolons. \
                 Keep the core within the optimal 40°C–70°C band and extract 30 ore chunks.",
            ),
            starter_code: String::from(
                r#"#![no_std]
extern crate rustbound_api;
use rustbound_api::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// In Rust, blocks and function bodies evaluate to their final expression without semicolons!
fn should_cool(temp: i32) -> bool {
    temp > 65
}

#[unsafe(no_mangle)]
pub extern "C" fn firmware_main() {
    let drill = QuarryDrill::bind(0).expect("drill");

    loop {
        let current_temp = drill.temperature();

        if should_cool(current_temp) {
            rustbound_api::log("Cooling cycle engaged");
        } else if let Ok(_ore) = drill.extract() {
            rustbound_api::log("Extraction cycle complete");
        }

        rustbound_api::yield_tick();
    }
}
"#,
            ),
            objectives: alloc::vec![
                MissionObjective {
                    description: String::from(
                        "Implement `should_cool(temp: i32) -> bool` expression return"
                    ),
                    completed: false,
                },
                MissionObjective {
                    description: String::from(
                        "Extract 30 ore chunks while preventing thermal shutdown (<80°C)"
                    ),
                    completed: false,
                },
            ],
        }
    }

    /// Loads Mission 6: "Automated Gantry Sorting"
    pub fn mission_6_automated_gantry_sorting() -> Self {
        Self {
            id: 6,
            title: String::from("Mission 06: Automated Gantry Sorting"),
            environment: String::from("Overhead Transfer Bay"),
            briefing: String::from(
                "Coordinate Transfer Crane #3 to pick up extracted ore chunks from Drill #0 \
                 and transport them across the facility into Storage Bay #2. Physical custody \
                 of the item moves through the crane and into the containment silo.",
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
    let crane = TransferCrane::bind(3).expect("crane");
    let storage = StorageBay::bind(2).expect("storage");

    loop {
        if drill.temperature() < 70 {
            if let Ok(ore) = drill.extract() {
                // Move crane trolley to pickup and dropoff coordinates
                let _ = crane.move_to(0, 0);
                let _ = crane.move_to(4, 0);

                // Deposit item directly into storage bay
                let _ = storage.deposit(ore);
                rustbound_api::log("Ore chunk delivered by overhead gantry");
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
                        "Command TransferCrane trolley across grid coordinates"
                    ),
                    completed: false,
                },
                MissionObjective {
                    description: String::from(
                        "Transfer 15 ore chunks from Drill into Storage with zero leaks"
                    ),
                    completed: false,
                },
            ],
        }
    }

    /// Loads a mission by numeric ID (1..=6).
    pub fn load(id: u32) -> Option<Self> {
        match id {
            1 => Some(Self::mission_1_cold_boot()),
            2 => Some(Self::mission_2_iron_extraction()),
            3 => Some(Self::mission_3_single_owner_custody()),
            4 => Some(Self::mission_4_immutable_quarry_interlock()),
            5 => Some(Self::mission_5_thermal_governor()),
            6 => Some(Self::mission_6_automated_gantry_sorting()),
            _ => None,
        }
    }
}
