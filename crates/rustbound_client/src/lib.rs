//! Client presentation layer, 3D industrial viewport, in-game IDE, and mission runner for Rustbound.

extern crate alloc;

pub mod camera;
pub mod dock;
pub mod editor;
pub mod environment;
pub mod inspector;
pub mod machines;
pub mod menu;
pub mod missions;
pub mod session;
pub mod ui;
pub mod visualizers;

pub use camera::{GameCamera, compute_camera_transform, orbit_camera_system, spawn_camera};
pub use dock::{RustboundTab, RustboundTabViewer, TabContext, create_default_dock_layout};
pub use editor::CodeEditorState;
pub use environment::{EnvironmentEntity, cleanup_environment, spawn_environment};
pub use inspector::{InspectorState, MachineTelemetryView};
pub use machines::{MachineMarker, MachineVisualType, machine_pick_system, spawn_machines};
pub use menu::{apply_ferric_theme, cleanup_menu_camera, menu_ui_system, setup_menu_camera};
pub use missions::{MissionDefinition, MissionObjective};
pub use session::RustboundSession;
pub use ui::{in_game_ui_system, trigger_compilation_and_deploy};
pub use visualizers::{CustodyRibbon, extract_custody_ribbons};

use bevy::prelude::*;

/// Primary application game state.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

/// Global interactive session resource managing active missions, simulation stepping, and UI modals.
#[derive(Resource)]
pub struct GameSession {
    pub session: RustboundSession,
    pub current_mission: u32,
    pub sim_timer: f32,
    pub mission_complete: bool,
    pub show_mission_briefing: bool,
    pub show_mission_complete: bool,
}

impl Default for GameSession {
    fn default() -> Self {
        Self::new(1)
    }
}

impl GameSession {
    pub fn new(mission_id: u32) -> Self {
        Self {
            session: RustboundSession::new(mission_id),
            current_mission: mission_id,
            sim_timer: 0.0,
            mission_complete: false,
            show_mission_briefing: true,
            show_mission_complete: false,
        }
    }

    pub fn setup_mission(&mut self, mission_id: u32) {
        self.current_mission = mission_id;
        self.session.setup_mission(mission_id);
        self.mission_complete = false;
        self.show_mission_briefing = true;
        self.show_mission_complete = false;
    }
}
