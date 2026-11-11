//! Parser for structured JSON compiler diagnostics emitted by rustc.

use serde::{Deserialize, Serialize};
use std::string::String;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "note")]
    Note,
    #[serde(rename = "help")]
    Help,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCode {
    pub code: String,
    pub explanation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticSpan {
    pub file_name: String,
    pub byte_start: u32,
    pub byte_end: u32,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub is_primary: bool,
    pub label: Option<String>,
    pub suggested_replacement: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RustcDiagnostic {
    pub message: String,
    pub code: Option<DiagnosticCode>,
    pub level: DiagnosticLevel,
    #[serde(default)]
    pub spans: Vec<DiagnosticSpan>,
    #[serde(default)]
    pub children: Vec<RustcDiagnostic>,
    pub rendered: Option<String>,
}

impl RustcDiagnostic {
    /// Returns `true` if this diagnostic is an error.
    #[inline]
    pub fn is_error(&self) -> bool {
        self.level == DiagnosticLevel::Error
    }

    /// Returns `true` if this diagnostic is a borrow checker ownership error (`E0382` or `E0502`).
    #[inline]
    pub fn is_borrow_error(&self) -> bool {
        if let Some(ref code) = self.code {
            code.code == "E0382" || code.code == "E0502" || code.code == "E0506"
        } else {
            false
        }
    }

    /// Returns the primary span where this diagnostic points.
    pub fn primary_span(&self) -> Option<&DiagnosticSpan> {
        self.spans.iter().find(|s| s.is_primary)
    }
}

/// Parses newline-delimited JSON streams emitted by `rustc --error-format=json`.
pub fn parse_diagnostics_stream(stream: &str) -> Vec<RustcDiagnostic> {
    let mut diagnostics = Vec::new();

    for line in stream.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Only parse JSON objects that contain diagnostic fields
        if trimmed.starts_with('{')
            && let Ok(diag) = serde_json::from_str::<RustcDiagnostic>(trimmed)
        {
            diagnostics.push(diag);
        }
    }

    diagnostics
}
