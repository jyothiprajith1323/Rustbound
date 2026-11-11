//! Comprehensive integration tests for in-game IDE, borrow visualizer ribbons,
//! telemetry inspector, docking workspace, and mission objective progression.

use rustbound_client::dock::{RustboundTab, create_default_dock_layout};
use rustbound_client::editor::CodeEditorState;
use rustbound_client::inspector::MachineTelemetryView;
use rustbound_client::missions::MissionDefinition;
use rustbound_client::session::RustboundSession;
use rustbound_client::visualizers::borrow::extract_custody_ribbons;
use rustbound_compiler::diagnostics::{
    DiagnosticCode, DiagnosticLevel, DiagnosticSpan, RustcDiagnostic,
};
use rustbound_protocol::packets::TelemetryPacket;

#[test]
fn test_editor_ropey_buffer_insert_delete_and_spans() {
    let mut editor = CodeEditorState::from_str("fn main() {\n    let x = 10;\n}\n");
    assert_eq!(editor.line_count(), 4);
    assert_eq!(editor.char_count(), 30);

    // Insert comment at beginning
    editor.insert(0, "// Rustbound\n");
    assert_eq!(editor.line_count(), 5);
    assert!(editor.text().starts_with("// Rustbound\nfn main()"));

    // Delete comment
    editor.delete(0, 13);
    assert_eq!(editor.line_count(), 4);
    assert!(editor.text().starts_with("fn main()"));

    // Verify byte to line & column mapping
    // "fn main() {\n" is 12 bytes
    // Next line starts at byte 12: "    let x = 10;\n"
    let (line, col) = editor.byte_to_line_and_col(16); // 4 spaces in on line 2
    assert_eq!(line, 2);
    assert_eq!(col, 5);

    // Add diagnostics
    let diag = RustcDiagnostic {
        message: String::from("unused variable: `x`"),
        code: Some(DiagnosticCode {
            code: String::from("unused_variables"),
            explanation: None,
        }),
        level: DiagnosticLevel::Warning,
        spans: vec![DiagnosticSpan {
            file_name: String::from("src/main.rs"),
            byte_start: 16,
            byte_end: 17,
            line_start: 2,
            line_end: 2,
            column_start: 5,
            column_end: 6,
            is_primary: true,
            label: Some(String::from("unused")),
            suggested_replacement: None,
        }],
        children: Vec::new(),
        rendered: Some(String::from("warning: unused variable `x`")),
    };

    editor.set_diagnostics(vec![diag]);
    assert_eq!(editor.diagnostics_for_line(2).len(), 1);
    assert_eq!(editor.diagnostics_for_line(1).len(), 0);
}

#[test]
fn test_borrow_visualizer_custody_ribbon_extraction() {
    let mut editor = CodeEditorState::from_str(
        "let ore = drill.extract()?;\nconveyor_a.feed(ore)?;\nconveyor_b.feed(ore)?;\n",
    );

    // Synthesize E0382 compiler diagnostic
    let diag = RustcDiagnostic {
        message: String::from("use of moved value: `ore`"),
        code: Some(DiagnosticCode {
            code: String::from("E0382"),
            explanation: Some(String::from("Borrow of moved value")),
        }),
        level: DiagnosticLevel::Error,
        spans: vec![
            DiagnosticSpan {
                file_name: String::from("src/main.rs"),
                byte_start: 28,
                byte_end: 50,
                line_start: 2,
                line_end: 2,
                column_start: 1,
                column_end: 23,
                is_primary: false,
                label: Some(String::from("value moved here")),
                suggested_replacement: None,
            },
            DiagnosticSpan {
                file_name: String::from("src/main.rs"),
                byte_start: 51,
                byte_end: 73,
                line_start: 3,
                line_end: 3,
                column_start: 17,
                column_end: 20,
                is_primary: true,
                label: Some(String::from("value used here after move")),
                suggested_replacement: None,
            },
        ],
        children: Vec::new(),
        rendered: Some(String::from("error[E0382]: use of moved value: `ore`")),
    };

    editor.set_diagnostics(vec![diag]);

    let ribbons = extract_custody_ribbons(&editor);
    assert_eq!(ribbons.len(), 1);

    let ribbon = &ribbons[0];
    assert_eq!(ribbon.variable_name, "ore");
    assert_eq!(ribbon.move_line, 2);
    assert_eq!(ribbon.move_label, "value moved here");
    assert_eq!(ribbon.error_line, 3);
    assert_eq!(ribbon.error_col, 17);
    assert_eq!(ribbon.error_label, "value used here after move");
    assert!(ribbon.explanation.contains("conservation of mass"));
}

#[test]
fn test_machine_inspector_status_and_gauges() {
    // Test nominal drill
    let nominal_packet = TelemetryPacket {
        machine_id: 10,
        machine_type: 1,
        status_flags: 1,
        temperature_celsius: 45,
        power_draw_watts: 1500,
        item_count: 5,
        next_item_id: 106,
        captured_at_tick: 50,
    };
    let view_nom = MachineTelemetryView::from_packet(&nominal_packet);
    assert_eq!(view_nom.machine_type_name, "Subterranean Quarry Drill");
    assert_eq!(view_nom.status, "OPERATIONAL: ONLINE");
    assert!(!view_nom.is_overheated);
    assert_eq!(view_nom.thermal_status_color(), (34, 197, 94)); // Green

    // Test overheated drill
    let overheat_packet = TelemetryPacket {
        machine_id: 10,
        machine_type: 1,
        status_flags: 4, // Overheated
        temperature_celsius: 82,
        power_draw_watts: 1500,
        item_count: 5,
        next_item_id: 106,
        captured_at_tick: 51,
    };
    let view_over = MachineTelemetryView::from_packet(&overheat_packet);
    assert_eq!(view_over.status, "FAULT: THERMAL OVERHEAT");
    assert!(view_over.is_overheated);
    assert_eq!(view_over.thermal_status_color(), (239, 68, 68)); // Red
}

#[test]
fn test_docking_layout_initialization() {
    let layout = create_default_dock_layout();
    assert_eq!(layout.surfaces_count(), 1);

    // Verify all 5 tabs exist in the layout
    let all_tabs: Vec<RustboundTab> = layout.iter_all_tabs().map(|(_, t)| *t).collect();
    assert!(all_tabs.contains(&RustboundTab::CodeEditor));
    assert!(all_tabs.contains(&RustboundTab::MachineInspector));
    assert!(all_tabs.contains(&RustboundTab::BorrowVisualizer));
    assert!(all_tabs.contains(&RustboundTab::Diagnostics));
    assert!(all_tabs.contains(&RustboundTab::Terminal));
}

#[test]
fn test_mission_definitions_and_loading() {
    let m1 = MissionDefinition::load(1).expect("mission 1");
    assert_eq!(m1.title, "Mission 01: Cold Boot");
    assert_eq!(m1.environment, "Dropship Bay");
    assert_eq!(m1.objectives.len(), 2);

    let m2 = MissionDefinition::load(2).expect("mission 2");
    assert_eq!(m2.title, "Mission 02: Subterranean Extraction");
    assert_eq!(m2.environment, "Rusted Quarry");
    assert_eq!(m2.objectives.len(), 2);

    let m3 = MissionDefinition::load(3).expect("mission 3");
    assert_eq!(m3.title, "Mission 03: Single Owner Custody");
    assert_eq!(m3.environment, "Automated Processing Yard");
    assert_eq!(m3.objectives.len(), 2);

    assert!(MissionDefinition::load(999).is_none());
}

#[test]
fn test_session_mission_1_cold_boot_progression() {
    let mut session = RustboundSession::new(1);
    assert_eq!(session.mission.id, 1);
    assert!(!session.is_mission_completed());

    // WAT bytecode for Mission 1 solution: binds to Generator 0 and calls ignite()
    let wat = r#"
    (module
        (import "env" "host_query_telemetry" (func $query (param i32 i32 i32) (result i32)))
        (import "env" "host_submit_command" (func $submit (param i32 i32 i32) (result i32)))
        (import "env" "host_yield_tick" (func $yield))
        (memory (export "memory") 1 2)
        (data (i32.const 100) "\52\42\01\32\00\00\00\00\64\00\00\00\00\00\00\00") ;; GeneratorIgniteCommand (cmd=50)
        (func (export "firmware_main")
            (drop (call $query (i32.const 0) (i32.const 0) (i32.const 64)))
            (drop (call $submit (i32.const 0) (i32.const 100) (i32.const 16)))
            (call $yield)
        )
    )
    "#;

    // Deploy bytecode to Generator (Machine 0)
    assert!(session.deploy_firmware_bytes(0, wat.as_bytes()).is_ok());

    // Before tick: not completed
    assert!(!session.is_mission_completed());

    // Step simulation by 1 tick
    session.step_simulation();

    // After tick: generator is ignited, mission objectives fulfilled!
    assert!(session.is_mission_completed());
}
