//! Compiler worker invoking rustc directly targeting wasm32-unknown-unknown.

use crate::diagnostics::{RustcDiagnostic, parse_diagnostics_stream};
use crate::sandbox::{SandboxConfig, SandboxError, run_with_timeout};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("sandbox violation: {0}")]
    Sandbox(#[from] SandboxError),
    #[error("failed to execute rustc: {0}")]
    Io(#[from] std::io::Error),
    #[error("compilation failed with errors: {0} diagnostics")]
    CompilationFailed(usize),
}

/// Output artifacts returned after a compilation run.
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub success: bool,
    pub diagnostics: Vec<RustcDiagnostic>,
    pub wasm_bytes: Option<Vec<u8>>,
}

/// Worker responsible for executing isolated rustc compilations.
pub struct CompilerWorker {
    pub rustc_path: PathBuf,
    pub sdk_rlib_path: PathBuf,
    pub search_paths: Vec<PathBuf>,
    pub sandbox: SandboxConfig,
}

impl Default for CompilerWorker {
    fn default() -> Self {
        Self {
            rustc_path: PathBuf::from("rustc"),
            sdk_rlib_path: PathBuf::from("librustbound_api.rlib"),
            search_paths: Vec::new(),
            sandbox: SandboxConfig::default(),
        }
    }
}

impl CompilerWorker {
    pub fn new(rustc_path: PathBuf, sdk_rlib_path: PathBuf) -> Self {
        Self {
            rustc_path,
            sdk_rlib_path,
            search_paths: Vec::new(),
            sandbox: SandboxConfig::default(),
        }
    }

    /// Adds a library or dependency search directory (`-L <path>`).
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Performs a fast syntax and type check emitting metadata only.
    pub fn check(&self, source_path: &Path) -> Result<Vec<RustcDiagnostic>, CompilerError> {
        let mut cmd = Command::new(&self.rustc_path);
        cmd.arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--crate-type")
            .arg("cdylib")
            .arg("--edition")
            .arg("2024")
            .arg("--emit=metadata")
            .arg("--error-format=json")
            .arg("--json=diagnostic-rendered-ansi")
            .arg(source_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for search_path in &self.search_paths {
            cmd.arg("-L").arg(search_path);
        }

        if self.sdk_rlib_path.exists() {
            cmd.arg("--extern")
                .arg(format!("rustbound_api={}", self.sdk_rlib_path.display()));
        }

        let (_status, _stdout, stderr) = run_with_timeout(cmd, &self.sandbox)?;
        Ok(parse_diagnostics_stream(&stderr))
    }

    /// Compiles player source code into a standalone WebAssembly firmware module.
    pub fn compile(
        &self,
        source_path: &Path,
        out_wasm_path: &Path,
    ) -> Result<CompileResult, CompilerError> {
        let mut cmd = Command::new(&self.rustc_path);
        cmd.arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--crate-type")
            .arg("cdylib")
            .arg("--edition")
            .arg("2024")
            .arg("-C")
            .arg("opt-level=z")
            .arg("-C")
            .arg("panic=abort")
            .arg("-C")
            .arg("link-arg=--allow-undefined")
            .arg("-o")
            .arg(out_wasm_path)
            .arg("--error-format=json")
            .arg("--json=diagnostic-rendered-ansi")
            .arg(source_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for search_path in &self.search_paths {
            cmd.arg("-L").arg(search_path);
        }

        if self.sdk_rlib_path.exists() {
            cmd.arg("--extern")
                .arg(format!("rustbound_api={}", self.sdk_rlib_path.display()));
        }

        let (status, _stdout, stderr) = run_with_timeout(cmd, &self.sandbox)?;
        let diagnostics = parse_diagnostics_stream(&stderr);
        let success = status.success() && out_wasm_path.exists();

        let wasm_bytes = if success {
            std::fs::read(out_wasm_path).ok()
        } else {
            None
        };

        Ok(CompileResult {
            success,
            diagnostics,
            wasm_bytes,
        })
    }
}
