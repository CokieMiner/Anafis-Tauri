// Markdown format export - simplified
//
// Exports data to Markdown table format (2D array)

use super::ExportConfig;
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Export data to Markdown format (simplified - expects 2D array)
#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri commands require owned types for arguments"
)]
pub fn export_to_markdown(
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

    // Process data rows - all rows are treated as data
    for row in &data {
        let Some(row_array) = row.as_array() else {
            continue;
        };

        // Format row cells
        let formatted_cells: Vec<String> = row_array
            .iter()
            .map(|cell| {
                let cell_content = match cell {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => String::new(),
                    _ => cell.to_string(),
                };
                // Escape pipe characters in markdown
                cell_content.replace('|', "\\|")
            })
            .collect();

        // Write the row
        let row_str = formatted_cells.join(" | ");
        writeln!(writer, "| {row_str} |").map_err(|e| format!("Failed to write row: {e}"))?;
    }

    writer
        .flush()
        .map_err(|e| format!("Failed to flush writer: {e}"))?;

    Ok(())
}
