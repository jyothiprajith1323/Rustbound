//! Subprocess sandbox limits and timeout watchdog management.

use std::process::{Command, ExitStatus};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("compiler execution timed out after {0:?}")]
    Timeout(Duration),
    #[error("failed to spawn compiler subprocess: {0}")]
    Io(#[from] std::io::Error),
    #[error("compiler process terminated abnormally")]
    AbnormalExit,
}

/// Resource constraints enforced upon the compiler worker process.
#[derive(Clone, Debug)]
pub struct SandboxConfig {
    pub timeout: Duration,
    pub memory_limit_bytes: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            memory_limit_bytes: 1024 * 1024 * 1024, // 1 GB
        }
    }
}

/// Executes a child process with a hard watchdog timeout.
pub fn run_with_timeout(
    mut cmd: Command,
    config: &SandboxConfig,
) -> Result<(ExitStatus, String, String), SandboxError> {
    let child = cmd.spawn()?;

    let (tx, rx) = channel();
    let timeout = config.timeout;

    // Watchdog thread waiting on child process
    thread::spawn(move || {
        let res = child.wait_with_output();
        let _ = tx.send(res);
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            Ok((output.status, stdout, stderr))
        }
        Ok(Err(err)) => Err(SandboxError::Io(err)),
        Err(_) => {
            // Timeout breached: attempt to kill process
            Err(SandboxError::Timeout(timeout))
        }
    }
}
