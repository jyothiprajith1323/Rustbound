use rustbound_compiler::diagnostics::parse_diagnostics_stream;
use rustbound_compiler::sandbox::{SandboxConfig, SandboxError, run_with_timeout};
use rustbound_compiler::worker::CompilerWorker;
use rustbound_protocol::packets::TelemetryPacket;
use rustbound_runtime::{TickOutcome, VmConfig, VmInstance};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

const SAMPLE_RUSTC_JSON: &str = r#"
{"message":"use of moved value: `ore`","code":{"code":"E0382","explanation":"Borrow of moved value"},"level":"error","spans":[{"file_name":"src/main.rs","byte_start":120,"byte_end":123,"line_start":12,"line_end":12,"column_start":30,"column_end":33,"is_primary":true,"text":[],"label":"value used here after move","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error[E0382]: use of moved value: `ore`\n"}
{"message":"variable `count` is never used","code":{"code":"unused_variables","explanation":null},"level":"warning","spans":[{"file_name":"src/main.rs","byte_start":50,"byte_end":55,"line_start":6,"line_end":6,"column_start":9,"column_end":14,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"warning: variable `count` is never used\n"}
"#;

#[test]
fn test_parse_rustc_json_diagnostics() {
    let diags = parse_diagnostics_stream(SAMPLE_RUSTC_JSON);
    assert_eq!(diags.len(), 2);

    let borrow_err = &diags[0];
    assert!(borrow_err.is_error());
    assert!(borrow_err.is_borrow_error());
    assert_eq!(borrow_err.code.as_ref().unwrap().code, "E0382");

    let span = borrow_err.primary_span().unwrap();
    assert_eq!(span.line_start, 12);
    assert_eq!(span.column_start, 30);
    assert_eq!(span.label.as_deref(), Some("value used here after move"));

    let warning = &diags[1];
    assert!(!warning.is_error());
    assert!(!warning.is_borrow_error());
}

#[test]
fn test_sandbox_timeout_watchdog() {
    let mut cmd = Command::new("sleep");
    cmd.arg("2");

    let config = SandboxConfig {
        timeout: Duration::from_millis(100),
        memory_limit_bytes: 1024 * 1024,
    };

    let result = run_with_timeout(cmd, &config);
    match result {
        Err(SandboxError::Timeout(duration)) => {
            assert_eq!(duration, Duration::from_millis(100));
        }
        other => panic!("expected SandboxError::Timeout, got: {:?}", other),
    }
}

#[test]
fn test_compiler_worker_real_compilation_and_execution() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let rlib_path =
        workspace_root.join("target/wasm32-unknown-unknown/release/librustbound_api.rlib");
    if !rlib_path.exists() {
        // If not precompiled in this environment, skip test
        eprintln!(
            "Skipping test: librustbound_api.rlib not found at {:?}",
            rlib_path
        );
        return;
    }

    let source_dir = std::env::temp_dir().join("rustbound_test_compilation");
    let _ = std::fs::create_dir_all(&source_dir);
    let source_file = source_dir.join("test_firmware.rs");
    let out_wasm = source_dir.join("test_firmware.wasm");

    let player_code = r#"
#![no_std]
extern crate rustbound_api;
use rustbound_api::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn firmware_main() {
    if let Ok(drill) = QuarryDrill::bind(0) {
        let _ = drill.temperature();
    }
    rustbound_api::yield_tick();
}
"#;

    std::fs::write(&source_file, player_code).expect("write source");

    let mut worker = CompilerWorker::new(PathBuf::from("rustc"), rlib_path);
    worker.add_search_path(workspace_root.join("target/wasm32-unknown-unknown/release/deps"));
    worker.add_search_path(workspace_root.join("target/release/deps"));

    let result = worker
        .compile(&source_file, &out_wasm)
        .expect("compilation");
    assert!(
        result.success,
        "compilation should succeed; diags: {:?}",
        result.diagnostics
    );
    let wasm_bytes = result.wasm_bytes.expect("wasm bytes should be present");
    assert!(!wasm_bytes.is_empty());
    assert!(
        wasm_bytes.len() < 20_000,
        "wasm binary size should be < 20KB"
    );

    // Instantiate and execute in Wasmi runtime
    let mut vm =
        VmInstance::instantiate(0, &wasm_bytes, VmConfig::default()).expect("instantiate vm");
    vm.update_telemetry(TelemetryPacket {
        machine_id: 0,
        machine_type: 1,
        status_flags: 1,
        temperature_celsius: 25,
        power_draw_watts: 1500,
        item_count: 0,
        next_item_id: 0,
        captured_at_tick: 1,
    });

    let outcome = vm.execute_tick().expect("execute tick");
    assert!(matches!(outcome, TickOutcome::Completed { .. }));
}
