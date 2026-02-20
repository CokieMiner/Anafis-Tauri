// LaTeX format export - simplified
//
// Exports data to LaTeX table format (2D array)

use super::ExportConfig;
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Export data to LaTeX format (simplified - expects 2D array)
#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri commands require owned types for arguments"
)]
pub fn export_to_latex(
    data: Vec<serde_json::Value>,
    file_path: String,
    _config: ExportConfig,
) -> Result<(), String> {
    if data.is_empty() {
        return Err("No data to export".to_string());
    }

    // Create file with buffered writer
    let file = File::create(&file_path).map_err(|e| format!("Failed to create file: {e}"))?;
    let mut writer = BufWriter::new(file);

    // LaTeX table environment
    writeln!(writer, "\\begin{{table}}[h]").map_err(|e| format!("Failed to write LaTeX: {e}"))?;
    writeln!(writer, "\\centering").map_err(|e| format!("Failed to write LaTeX: {e}"))?;

    // Determine number of columns
    let num_cols = data
        .iter()
        .filter_map(|row| row.as_array().map(std::vec::Vec::len))
        .max()
        .unwrap_or(0);
    if num_cols == 0 {
        return Err("No columns found in data".to_string());
    }

    // Default column alignment (left-aligned)
    let column_alignment = "l".repeat(num_cols);

    // Tabular environment
    writeln!(writer, "\\begin{{tabular}}{{{column_alignment}}}")
        .map_err(|e| format!("Failed to write LaTeX: {e}"))?;

    // Process data rows
    for row in &data {
        let Some(row_array) = row.as_array() else {
            continue;
        };

        // Format row cells
        let formatted_cells: Vec<String> = row_array
            .iter()
            .map(|cell| match cell {
                Value::String(s) => latex_escape(s),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                _ => cell.to_string(),
            })
            .collect();

        // Write the row
        let row_str = formatted_cells.join(" & ");
        writeln!(writer, "{row_str} \\\\")
            .map_err(|e| format!("Failed to write LaTeX row: {e}"))?;
    }
    writeln!(writer, "\\end{{tabular}}").map_err(|e| format!("Failed to write LaTeX: {e}"))?;
    writeln!(writer, "\\end{{table}}").map_err(|e| format!("Failed to write LaTeX: {e}"))?;

    writer
        .flush()
        .map_err(|e| format!("Failed to flush writer: {e}"))?;

    Ok(())
}

/// Escape LaTeX special characters
fn latex_escape(text: &str) -> String {
    text.replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}
