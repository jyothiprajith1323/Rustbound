use rustbound_compiler::diagnostics::parse_diagnostics_stream;
use rustbound_compiler::sandbox::{SandboxConfig, SandboxError, run_with_timeout};
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
