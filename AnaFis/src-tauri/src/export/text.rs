// Text format exports: CSV, TSV, TXT - simplified
//
// Exports data to delimited text formats (2D array)

use super::{ExportConfig, ExportFormat};
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Export data to CSV/TSV/TXT format (simplified - expects 2D array)
#[tauri::command]
pub async fn export_to_text(
    data: Vec<serde_json::Value>,
    file_path: String,
    config: ExportConfig,
) -> Result<(), String> {
    // Determine delimiter based on format
    let delimiter = match config.format {
        ExportFormat::Csv => config.options.delimiter.as_deref().unwrap_or(","),
        ExportFormat::Tsv => "\t",
        ExportFormat::Txt => config.options.delimiter.as_deref().unwrap_or("|"),
        _ => return Err("Invalid format for text export".to_string()),
    };

    let quote_char = '"';
    let line_ending = "\r\n";

    // Create file with buffered writer for performance
    let file = File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);

    // Export data rows - each element should be an array
    for row_value in data.iter() {
        let row_array = match row_value.as_array() {
            Some(arr) => arr,
            None => continue,
        };

        // Format and write the row
        let formatted_row: Vec<String> = row_array
            .iter()
            .map(|cell| format_cell_value(cell, delimiter, quote_char))
            .collect();

        write!(writer, "{}{}", formatted_row.join(delimiter), line_ending)
            .map_err(|e| format!("Failed to write row: {}", e))?;
    }

    writer
        .flush()
        .map_err(|e| format!("Failed to flush writer: {}", e))?;

    Ok(())
}

/// Format a cell value for text export with proper escaping
fn format_cell_value(value: &Value, delimiter: &str, quote_char: char) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            // Check if we need to quote the value
            let needs_quoting = s.contains(delimiter)
                || s.contains(quote_char)
                || s.contains('\n')
                || s.contains('\r');

            if needs_quoting {
                // Escape quote characters by doubling them
                let escaped = s.replace(quote_char, &format!("{}{}", quote_char, quote_char));
                format!("{}{}{}", quote_char, escaped, quote_char)
            } else {
                s.clone()
            }
        }
        Value::Array(_) | Value::Object(_) => {
            // For complex types, serialize to JSON string
            serde_json::to_string(value).unwrap_or_default()
        }
    }
}
