// Text format exports: CSV, TSV, TXT
//
// Handles exporting data to delimited text formats with proper escaping,
// quoting, and encoding support.

use std::fs::File;
use std::io::{BufWriter, Write};
use serde_json::Value;
use super::{ExportConfig, ExportFormat, DataStructure};

/// Export data to CSV/TSV/TXT format
#[tauri::command]
pub async fn export_to_text(
    data: Vec<Vec<Value>>,
    file_path: String,
    config: ExportConfig,
) -> Result<(), String> {
    // Validate data structure - text formats only support single-sheet 2D arrays
    if !matches!(config.data_structure, DataStructure::Array2D) {
        return Err(format!(
            "Text formats (CSV/TSV/TXT) only support single-sheet data (Array2D). Received: {:?}. Please export each sheet separately.",
            config.data_structure
        ));
    }

    // Determine delimiter based on format
    let delimiter = match config.format {
        ExportFormat::Csv => config.options.delimiter.as_deref().unwrap_or(","),
        ExportFormat::Tsv => "\t",
        ExportFormat::Txt => config.options.delimiter.as_deref().unwrap_or("|"),
        _ => return Err("Invalid format for text export".to_string()),
    };

    let quote_char = config.options.quote_char.chars().next().unwrap_or('"');
    let line_ending = match config.options.line_ending.as_str() {
        "lf" => "\n",
        "crlf" => "\r\n",
        _ => "\r\n", // default to CRLF
    };

    // Create file with buffered writer for performance
    let file = File::create(&file_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);

    // Export data rows
    for row in data.iter() {
        // Don't skip empty rows - preserve them to maintain data structure
        let formatted_row: Vec<String> = row.iter().map(|cell| {
            format_cell_value(cell, delimiter, quote_char)
        }).collect();

        write!(writer, "{}{}", formatted_row.join(delimiter), line_ending)
            .map_err(|e| format!("Failed to write row: {}", e))?;
    }

    writer.flush()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_cell_value() {
        let quote = '"';
        
        // Test simple values
        assert_eq!(format_cell_value(&json!(42), ",", quote), "42");
        assert_eq!(format_cell_value(&json!("hello"), ",", quote), "hello");
        assert_eq!(format_cell_value(&json!(true), ",", quote), "true");
        assert_eq!(format_cell_value(&json!(null), ",", quote), "");
        
        // Test values that need quoting
        assert_eq!(format_cell_value(&json!("hello,world"), ",", quote), "\"hello,world\"");
        assert_eq!(format_cell_value(&json!("say \"hi\""), ",", quote), "\"say \"\"hi\"\"\"");
        assert_eq!(format_cell_value(&json!("line\nbreak"), ",", quote), "\"line\nbreak\"");
    }

    #[test]
    fn test_format_cell_value_with_pipe_delimiter() {
        let quote = '"';
        
        // Comma should not trigger quoting with pipe delimiter
        assert_eq!(format_cell_value(&json!("hello,world"), "|", quote), "hello,world");
        
        // Pipe should trigger quoting
        assert_eq!(format_cell_value(&json!("hello|world"), "|", quote), "\"hello|world\"");
    }
}
