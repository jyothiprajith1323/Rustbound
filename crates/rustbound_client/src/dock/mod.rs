//! Multi-pane docking system wrapping egui_dock for in-game IDE and telemetry inspection.

use crate::editor::CodeEditorState;
use crate::inspector::{InspectorState, MachineTelemetryView};
use crate::visualizers::borrow::extract_custody_ribbons;
use alloc::collections::BTreeMap;
use alloc::string::String;
use egui_dock::{DockState, NodeIndex, TabViewer};
use rustbound_protocol::packets::TelemetryPacket;

/// Available tab panes in the Rustbound docking workspace.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RustboundTab {
    CodeEditor,
    MachineInspector,
    BorrowVisualizer,
    Diagnostics,
    Terminal,
}

impl RustboundTab {
    pub fn title(&self) -> &'static str {
        match self {
            Self::CodeEditor => "Source Editor",
            Self::MachineInspector => "Machine Telemetry",
            Self::BorrowVisualizer => "Borrow Checker Visualizer",
            Self::Diagnostics => "Compiler Diagnostics",
            Self::Terminal => "Machine Terminal",
        }
    }
}

/// Creates the default 4-pane IDE docking layout.
pub fn create_default_dock_layout() -> DockState<RustboundTab> {
    let mut state = DockState::new(alloc::vec![RustboundTab::CodeEditor]);
    let [main_pane, bottom_pane] = state.main_surface_mut().split_below(
        NodeIndex::root(),
        0.7,
        alloc::vec![RustboundTab::Diagnostics],
    );
    let [_editor_pane, _inspector_pane] = state.main_surface_mut().split_right(
        main_pane,
        0.65,
        alloc::vec![RustboundTab::MachineInspector],
    );
    let [_diag_pane, _term_pane] =
        state
            .main_surface_mut()
            .split_right(bottom_pane, 0.5, alloc::vec![RustboundTab::Terminal]);

    // Push BorrowVisualizer as a tab in the editor pane
    state
        .main_surface_mut()
        .push_to_first_leaf(RustboundTab::BorrowVisualizer);

    state
}

/// Context passed to the TabViewer during egui UI passes.
pub struct TabContext<'a> {
    pub editor: &'a mut CodeEditorState,
    pub inspector: &'a mut InspectorState,
    pub terminal_logs: &'a [(u32, String)],
    pub telemetry_map: &'a BTreeMap<u32, TelemetryPacket>,
}

/// TabViewer rendering UI controls for each active docking pane.
pub struct RustboundTabViewer<'a> {
    pub context: TabContext<'a>,
}

impl<'a> TabViewer for RustboundTabViewer<'a> {
    type Tab = RustboundTab;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(*tab)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            RustboundTab::CodeEditor => {
                ui.heading("Aethelgard Firmware Editor");
                ui.separator();
                let mut text = self.context.editor.text();
                if ui
                    .add(
                        egui::TextEdit::multiline(&mut text)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY),
                    )
                    .changed()
                {
                    *self.context.editor = CodeEditorState::from_str(&text);
                }
            }
            RustboundTab::MachineInspector => {
                ui.heading("Machine Inspector & Bus Telemetry");
                ui.separator();

                if self.context.telemetry_map.is_empty() {
                    ui.label("No machines detected on local bus.");
                } else {
                    for (id, telem) in self.context.telemetry_map {
                        let view = MachineTelemetryView::from_packet(telem);
                        ui.group(|ui| {
                            ui.label(format!("Machine #{}: {}", id, view.machine_type_name));
                            ui.label(format!("Status: {}", view.status));
                            ui.colored_label(
                                egui::Color32::from_rgb(
                                    view.thermal_status_color().0,
                                    view.thermal_status_color().1,
                                    view.thermal_status_color().2,
                                ),
                                format!("Temperature: {}°C", view.temperature_celsius),
                            );
                            ui.label(format!("Power Draw: {} W", view.power_draw_watts));
                            ui.label(format!("Stored Items: {}", view.item_count));
                        });
                    }
                }
            }
            RustboundTab::BorrowVisualizer => {
                ui.heading("Physical Custody & Borrow Flow");
                ui.separator();

                let ribbons = extract_custody_ribbons(self.context.editor);
                if ribbons.is_empty() {
                    ui.label("No active ownership or borrow violations detected.");
                    ui.label("Physical item conservation verified.");
                } else {
                    for ribbon in &ribbons {
                        ui.group(|ui| {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("Custody Conflict: Variable `{}`", ribbon.variable_name),
                            );
                            ui.label(format!(
                                "  [1] Transferred: Line {}, Col {} ({})",
                                ribbon.move_line, ribbon.move_col, ribbon.move_label
                            ));
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 120, 120),
                                format!(
                                    "  [2] Illegal Duplicate Use: Line {}, Col {} ({})",
                                    ribbon.error_line, ribbon.error_col, ribbon.error_label
                                ),
                            );
                            ui.separator();
                            ui.label(&ribbon.explanation);
                        });
                    }
                }
            }
            RustboundTab::Diagnostics => {
                ui.heading("Compiler Diagnostics");
                ui.separator();

                let diags = self.context.editor.diagnostics();
                if diags.is_empty() {
                    ui.colored_label(egui::Color32::GREEN, "Build clean: 0 errors, 0 warnings.");
                } else {
                    for diag in diags {
                        let color = if diag.is_error() {
                            egui::Color32::RED
                        } else {
                            egui::Color32::YELLOW
                        };
                        ui.colored_label(color, format!("[{:?}] {}", diag.level, diag.message));
                    }
                }
            }
            RustboundTab::Terminal => {
                ui.heading("Serial Telemetry Output");
                ui.separator();

                if self.context.terminal_logs.is_empty() {
                    ui.label("Terminal idle. No incoming messages.");
                } else {
                    for (level, msg) in self.context.terminal_logs {
                        ui.label(format!("[LOG {}] {}", level, msg));
                    }
                }
            }
        }
    }
}
