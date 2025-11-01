// Markdown format export
//
// Handles exporting data to Markdown table format.

use std::fs::File;
use std::io::{BufWriter, Write};
use serde_json::Value;
use super::{ExportConfig, DataStructure};

/// Export data to Markdown format
#[tauri::command]
pub async fn export_to_markdown(
    data: Vec<Vec<Value>>,
    file_path: String,
    config: ExportConfig,
) -> Result<(), String> {
    // Validate data structure - Markdown only supports single-sheet 2D arrays
    if !matches!(config.data_structure, DataStructure::Array2D) {
        return Err(format!(
            "Markdown export only supports single-sheet data (Array2D). Received: {:?}. Please export each sheet separately.",
            config.data_structure
        ));
    }

    if data.is_empty() {
        return Err("No data to export".to_string());
    }

    // Create file with buffered writer
    let file = File::create(&file_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);

    // Process data rows - all rows are treated as data
    for row in data.iter() {
        // Format row cells
        let formatted_cells: Vec<String> = row.iter().map(|cell| {
            let cell_content = match cell {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                _ => cell.to_string(),
            };
            // Escape pipe characters in markdown
            cell_content.replace("|", "\\|")
        }).collect();

        // Write the row
        let row_str = formatted_cells.join(" | ");
        writeln!(writer, "| {} |", row_str)
            .map_err(|e| format!("Failed to write row: {}", e))?;
    }

    writer.flush()
        .map_err(|e| format!("Failed to flush writer: {}", e))?;

    Ok(())
}