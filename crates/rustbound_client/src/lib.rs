//! Client presentation layer, IDE docking interface, and mission runner for Rustbound.

extern crate alloc;

pub mod dock;
pub mod editor;
pub mod inspector;
pub mod missions;
pub mod session;
pub mod visualizers;

pub use dock::{RustboundTab, RustboundTabViewer, TabContext, create_default_dock_layout};
pub use editor::CodeEditorState;
pub use inspector::{InspectorState, MachineTelemetryView};
pub use missions::{MissionDefinition, MissionObjective};
pub use session::RustboundSession;
pub use visualizers::{CustodyRibbon, extract_custody_ribbons};
