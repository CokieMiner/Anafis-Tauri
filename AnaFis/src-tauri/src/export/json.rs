// JSON format export
//
// Handles exporting data to JSON with multiple format options:
// - array: Simple 2D array [[1, 2], [3, 4]]
// - object: Object with named columns {col1: [1, 3], col2: [2, 4]}
// - records: Array of objects [{col1: 1, col2: 2}, {col1: 3, col2: 4}]

use std::fs::File;
use std::io::{BufWriter, Write};
use serde_json::{json, Value};
use super::ExportConfig;

/// Export data to JSON format
#[tauri::command]
pub async fn export_to_json(
    data: Vec<Vec<Value>>,
    file_path: String,
    config: ExportConfig,
) -> Result<(), String> {
    if data.is_empty() {
        return Err("No data to export".to_string());
    }

    // Convert data based on format option
    let json_data = match config.options.json_format.as_str() {
        "array" => format_as_array(&data),
        "object" => format_as_object(&data, config.options.include_headers),
        "records" => format_as_records(&data, config.options.include_headers),
        _ => return Err(format!("Unknown JSON format: {}", config.options.json_format)),
    }?;

    // Serialize to string with custom formatting
    let json_string = if config.options.pretty_print {
        // Use custom pretty printing that keeps arrays compact
        pretty_print_json(&json_data)?
    } else {
        serde_json::to_string(&json_data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?
    };

    // Write to file
    let file = File::create(&file_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);
    
    writer.write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to write JSON: {}", e))?;
    
    writer.flush()
        .map_err(|e| format!("Failed to flush writer: {}", e))?;

    Ok(())
}

/// Custom pretty print that keeps arrays on single lines
fn pretty_print_json(value: &Value) -> Result<String, String> {
    fn format_value(value: &Value, indent: usize, is_top_level: bool) -> String {
        let indent_str = "  ".repeat(indent);
        
        match value {
            Value::Array(arr) => {
                if arr.is_empty() {
                    return "[]".to_string();
                }
                
                // Check if this is an array of arrays (2D array) at top level
                let is_array_of_arrays = is_top_level && arr.iter().any(|v| matches!(v, Value::Array(_)));
                
                if is_array_of_arrays {
                    // Format as multi-line with each sub-array on its own line
                    let mut lines = vec!["[".to_string()];
                    for (i, item) in arr.iter().enumerate() {
                        let comma = if i < arr.len() - 1 { "," } else { "" };
                        let formatted = format_value(item, indent + 1, false);
                        lines.push(format!("{}  {}{}", indent_str, formatted, comma));
                    }
                    lines.push(format!("{}]", indent_str));
                    lines.join("\n")
                } else {
                    // Keep simple arrays on a single line
                    let items: Vec<String> = arr.iter()
                        .map(|v| match v {
                            Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "null".to_string(),
                            Value::Array(_) => format_value(v, 0, false),
                            Value::Object(_) => format_value(v, indent + 1, false),
                        })
                        .collect();
                    format!("[{}]", items.join(", "))
                }
            }
            Value::Object(obj) => {
                if obj.is_empty() {
                    return "{}".to_string();
                }
                
                let mut lines = vec!["{".to_string()];
                let entries: Vec<_> = obj.iter().collect();
                
                for (i, (key, val)) in entries.iter().enumerate() {
                    let comma = if i < entries.len() - 1 { "," } else { "" };
                    let formatted_val = format_value(val, indent + 1, false);
                    lines.push(format!("{}  \"{}\": {}{}", indent_str, key, formatted_val, comma));
                }
                
                lines.push(format!("{}}}", indent_str));
                lines.join("\n")
            }
            Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
        }
    }
    
    Ok(format_value(value, 0, true))
}

/// Format data as simple 2D array
fn format_as_array(data: &[Vec<Value>]) -> Result<Value, String> {
    Ok(json!(data))
}

/// Format data as object with named columns
/// Example: {"Time": [0, 1, 2], "Temperature": [20, 21, 22]}
fn format_as_object(data: &[Vec<Value>], has_headers: bool) -> Result<Value, String> {
    if data.is_empty() {
        return Ok(json!({}));
    }

    let (headers, data_rows) = if has_headers && data.len() > 1 {
        // First row is headers - use values or auto-generate for empty cells
        let headers: Vec<String> = data[0].iter().enumerate().map(|(i, val)| {
            match val {
                Value::String(s) if !s.trim().is_empty() => s.trim().to_string(),
                Value::Number(n) => n.to_string(),
                _ => format!("Column{}", i + 1),
            }
        }).collect();
        
        // Check if ALL headers are auto-generated (meaning entire first row was empty)
        let all_auto_generated = headers.iter().all(|h| h.starts_with("Column"));
        
        if all_auto_generated {
            // Entire first row was empty, treat it as data instead
            let col_count = data[0].len();
            let headers: Vec<String> = (1..=col_count)
                .map(|i| format!("Column{}", i))
                .collect();
            (headers, data)
        } else {
            // At least some headers are valid, skip first row
            (headers, &data[1..])
        }
    } else {
        // No headers - generate column names and use all data
        let col_count = data[0].len();
        let headers: Vec<String> = (1..=col_count)
            .map(|i| format!("Column{}", i))
            .collect();
        (headers, data)
    };

    // Build column-oriented object with preserved order
    // serde_json::Map uses BTreeMap which sorts keys alphabetically
    // We need to build it in order to preserve column positions
    let mut result = serde_json::Map::new();
    
    for (col_idx, header) in headers.iter().enumerate() {
        let column_values: Vec<&Value> = data_rows.iter()
            .filter_map(|row| row.get(col_idx))
            .collect();
        result.insert(header.clone(), json!(column_values));
    }

    Ok(Value::Object(result))
}

/// Format data as array of record objects
/// Example: [{"Time": 0, "Temperature": 20}, {"Time": 1, "Temperature": 21}]
fn format_as_records(data: &[Vec<Value>], has_headers: bool) -> Result<Value, String> {
    if data.is_empty() {
        return Ok(json!([]));
    }

    let (headers, data_rows) = if has_headers && data.len() > 1 {
        // First row is headers - use values or auto-generate for empty cells
        let headers: Vec<String> = data[0].iter().enumerate().map(|(i, val)| {
            match val {
                Value::String(s) if !s.trim().is_empty() => s.trim().to_string(),
                Value::Number(n) => n.to_string(),
                _ => format!("Column{}", i + 1),
            }
        }).collect();
        
        // Check if ALL headers are auto-generated (meaning entire first row was empty)
        let all_auto_generated = headers.iter().all(|h| h.starts_with("Column"));
        
        if all_auto_generated {
            // Entire first row was empty, treat it as data instead
            let col_count = data[0].len();
            let headers: Vec<String> = (1..=col_count)
                .map(|i| format!("Column{}", i))
                .collect();
            (headers, data)
        } else {
            // At least some headers are valid, skip first row
            (headers, &data[1..])
        }
    } else {
        // No headers - generate column names and use all data
        let col_count = data[0].len();
        let headers: Vec<String> = (1..=col_count)
            .map(|i| format!("Column{}", i))
            .collect();
        (headers, data)
    };

    // Build array of record objects
    let records: Vec<Value> = data_rows.iter().map(|row| {
        let mut record = serde_json::Map::new();
        for (col_idx, header) in headers.iter().enumerate() {
            let value = row.get(col_idx).cloned().unwrap_or(Value::Null);
            record.insert(header.clone(), value);
        }
        Value::Object(record)
    }).collect();

    Ok(json!(records))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_as_array() {
        let data = vec![
            vec![json!("Time"), json!("Temp")],
            vec![json!(0), json!(20)],
            vec![json!(1), json!(21)],
        ];
        
        let result = format_as_array(&data).unwrap();
        assert_eq!(result, json!([
            ["Time", "Temp"],
            [0, 20],
            [1, 21]
        ]));
    }

    #[test]
    fn test_format_as_object_with_headers() {
        let data = vec![
            vec![json!("Time"), json!("Temp")],
            vec![json!(0), json!(20)],
            vec![json!(1), json!(21)],
        ];
        
        let result = format_as_object(&data, true).unwrap();
        assert_eq!(result, json!({
            "Time": [0, 1],
            "Temp": [20, 21]
        }));
    }

    #[test]
    fn test_format_as_records_with_headers() {
        let data = vec![
            vec![json!("Time"), json!("Temp")],
            vec![json!(0), json!(20)],
            vec![json!(1), json!(21)],
        ];
        
        let result = format_as_records(&data, true).unwrap();
        assert_eq!(result, json!([
            {"Time": 0, "Temp": 20},
            {"Time": 1, "Temp": 21}
        ]));
    }

    #[test]
    fn test_format_as_records_without_headers() {
        let data = vec![
            vec![json!(0), json!(20)],
            vec![json!(1), json!(21)],
        ];
        
        let result = format_as_records(&data, false).unwrap();
        assert_eq!(result, json!([
            {"Column1": 0, "Column2": 20},
            {"Column1": 1, "Column2": 21}
        ]));
    }
}
