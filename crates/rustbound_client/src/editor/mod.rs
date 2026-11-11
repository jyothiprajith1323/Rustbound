//! In-game text buffer and compiler diagnostic span mapping using Ropey.

use alloc::string::String;
use alloc::vec::Vec;
use ropey::Rope;
use rustbound_compiler::diagnostics::RustcDiagnostic;

/// State for the embedded firmware code editor.
pub struct CodeEditorState {
    buffer: Rope,
    cursor: usize,
    diagnostics: Vec<RustcDiagnostic>,
}

impl Default for CodeEditorState {
    fn default() -> Self {
        Self::new()
    }
}

impl core::str::FromStr for CodeEditorState {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new_with_text(s))
    }
}

impl CodeEditorState {
    pub fn new() -> Self {
        Self {
            buffer: Rope::new(),
            cursor: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn new_with_text(text: &str) -> Self {
        Self {
            buffer: Rope::from_str(text),
            cursor: 0,
            diagnostics: Vec::new(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &str) -> Self {
        Self::new_with_text(text)
    }

    /// Returns the complete text content of the buffer as a String.
    pub fn text(&self) -> String {
        self.buffer.to_string()
    }

    /// Returns the number of lines in the text buffer.
    pub fn line_count(&self) -> usize {
        self.buffer.len_lines()
    }

    /// Returns the character length of the buffer.
    pub fn char_count(&self) -> usize {
        self.buffer.len_chars()
    }

    /// Returns the byte length of the buffer.
    pub fn byte_count(&self) -> usize {
        self.buffer.len_bytes()
    }

    /// Inserts text at the specified character offset.
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        let idx = char_idx.min(self.buffer.len_chars());
        self.buffer.insert(idx, text);
        self.cursor = idx + text.chars().count();
    }

    /// Deletes character range `[start_char, end_char)`.
    pub fn delete(&mut self, start_char: usize, end_char: usize) {
        let max_len = self.buffer.len_chars();
        let start = start_char.min(max_len);
        let end = end_char.min(max_len);
        if start < end {
            self.buffer.remove(start..end);
            self.cursor = start;
        }
    }

    /// Maps a raw 0-indexed byte offset to 1-indexed (line, column).
    pub fn byte_to_line_and_col(&self, byte_idx: usize) -> (usize, usize) {
        let clamped_byte = byte_idx.min(self.buffer.len_bytes());
        let char_idx = self.buffer.byte_to_char(clamped_byte);
        let line_idx = self.buffer.char_to_line(char_idx);
        let line_start_char = self.buffer.line_to_char(line_idx);
        let col = char_idx.saturating_sub(line_start_char);
        (line_idx + 1, col + 1)
    }

    /// Sets active compiler diagnostics for highlight rendering.
    pub fn set_diagnostics(&mut self, diagnostics: Vec<RustcDiagnostic>) {
        self.diagnostics = diagnostics;
    }

    /// Returns all active compiler diagnostics.
    pub fn diagnostics(&self) -> &[RustcDiagnostic] {
        &self.diagnostics
    }

    /// Returns all compiler diagnostics affecting the specified 1-indexed line.
    pub fn diagnostics_for_line(&self, line: usize) -> Vec<&RustcDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|diag| {
                diag.spans
                    .iter()
                    .any(|span| span.line_start <= line && line <= span.line_end)
            })
            .collect()
    }
}
