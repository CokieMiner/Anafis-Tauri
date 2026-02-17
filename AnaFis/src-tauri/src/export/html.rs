// HTML format export - simplified
//
// Exports data to HTML table format (2D array)

use super::ExportConfig;
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Export data to HTML format (simplified - expects 2D array)
#[tauri::command]
pub async fn export_to_html(
    data: Vec<serde_json::Value>,
    file_path: String,
    _config: ExportConfig,
) -> Result<(), String> {
    if data.is_empty() {
        return Err("No data to export".to_string());
    }

    // Build HTML content
    let mut html = String::new();

    // HTML header
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<title>Exported Data</title>\n");

    // Basic CSS styling
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
    html.push_str("th { background-color: #f2f2f2; font-weight: bold; }\n");
    html.push_str("tr:nth-child(even) { background-color: #f9f9f9; }\n");
    html.push_str("tr:hover { background-color: #f5f5f5; }\n");
    html.push_str("</style>\n");

    html.push_str("</head>\n<body>\n");

    // Table
    html.push_str("<table>\n<tbody>\n");

    // Process data rows - all rows are treated as data
    for row in data.iter() {
        let row_array = match row.as_array() {
            Some(arr) => arr,
            None => continue,
        };

        html.push_str("<tr>\n");

        for cell in row_array {
            let cell_content = match cell {
                Value::String(s) => html_escape(s),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                _ => cell.to_string(),
            };

            html.push_str(&format!("<td>{}</td>\n", cell_content));
        }

        html.push_str("</tr>\n");
    }

    html.push_str("</tbody>\n</table>\n");
    html.push_str("</body>\n</html>\n");

    // Write to file
    let file = File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);

    writer
        .write_all(html.as_bytes())
        .map_err(|e| format!("Failed to write HTML: {}", e))?;

    writer
        .flush()
        .map_err(|e| format!("Failed to flush writer: {}", e))?;

    Ok(())
}

/// Escape HTML special characters
fn html_escape(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}
