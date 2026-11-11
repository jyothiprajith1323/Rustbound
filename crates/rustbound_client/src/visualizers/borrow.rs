//! Ownership and borrowing visualizer computing physical custody ribbons for E0382 errors.

use crate::editor::CodeEditorState;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// A visual custody ribbon connecting the site where an item was moved to the site
/// where an illegal subsequent access was attempted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustodyRibbon {
    pub variable_name: String,
    pub move_line: usize,
    pub move_col: usize,
    pub move_label: String,
    pub error_line: usize,
    pub error_col: usize,
    pub error_label: String,
    pub explanation: String,
}

/// Analyzes compiler diagnostics in the code editor and generates visual custody ribbons.
pub fn extract_custody_ribbons(editor: &CodeEditorState) -> Vec<CustodyRibbon> {
    let mut ribbons = Vec::new();

    for diag in editor.diagnostics() {
        if !diag.is_borrow_error() {
            continue;
        }

        // Extract variable name from message: "use of moved value: `ore`"
        let var_name = if let Some(start) = diag.message.find('`') {
            if let Some(end) = diag.message[start + 1..].find('`') {
                diag.message[start + 1..start + 1 + end].to_string()
            } else {
                String::from("token")
            }
        } else {
            String::from("token")
        };

        // Extract error span (primary)
        let Some(error_span) = diag.primary_span() else {
            continue;
        };

        // Find move span from related spans or children
        let mut move_line = error_span.line_start.saturating_sub(1);
        let mut move_col = error_span.column_start;
        let mut move_label = String::from("value moved into machine custody here");

        for span in &diag.spans {
            if let Some(label) = &span.label
                && (label.contains("moved here") || label.contains("move occurs"))
            {
                move_line = span.line_start;
                move_col = span.column_start;
                move_label = label.clone();
                break;
            }
        }

        ribbons.push(CustodyRibbon {
            variable_name: var_name,
            move_line,
            move_col,
            move_label,
            error_line: error_span.line_start,
            error_col: error_span.column_start,
            error_label: error_span
                .label
                .clone()
                .unwrap_or_else(|| String::from("value used here after move")),
            explanation: String::from(
                "Rust's borrow checker enforces the physical conservation of mass. \
                 Custody of this token was transferred; duplicate consumption is forbidden.",
            ),
        });
    }

    ribbons
}
