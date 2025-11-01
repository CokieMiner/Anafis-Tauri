// JSON format export
//
// Handles exporting data to JSON with multiple format options:
// - array: Simple 2D array [[1, 2], [3, 4]]
// - object: Object with named columns {col1: [1, 3], col2: [2, 4]}
// - records: Array of objects [{col1: 1, col2: 2}, {col1: 3, col2: 4}]

use std::fs::File;
use std::io::{BufWriter, Write};
use serde_json::{json, Value};
use super::{ExportConfig, DataStructure};

/// Helper function to collect valid sheet rows from Value array
fn collect_sheet_rows(rows: &[Value]) -> Vec<Vec<Value>> {
    rows.iter()
        .filter_map(|row| {
            row.as_array().cloned()
        })
        .collect()
}

/// Helper function to convert a Value to a string for column naming
fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) if !s.trim().is_empty() => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => "value".to_string(),
    }
}

/// Helper function to format sheet data according to JSON format options
fn format_sheet_data(sheet_rows: &[Vec<Value>], json_format: &str, include_headers: bool) -> Result<Value, String> {
    match json_format {
        "array" => format_as_array(sheet_rows),
        "object" => format_as_object(sheet_rows, include_headers),
        "records" => format_as_records(sheet_rows, include_headers),
        _ => Err(format!("Unknown JSON format: {}", json_format)),
    }
}

/// Export data to JSON format
#[tauri::command]
pub async fn export_to_json(
    data: Vec<Value>,
    file_path: String,
    config: ExportConfig,
) -> Result<(), String> {
    if data.is_empty() {
        return Err("No data to export".to_string());
    }

    // Use explicit data structure marker instead of implicit detection
    let json_data = match config.data_structure {
        DataStructure::MultiSheetJson => {
            // Handle multi-sheet JSON format: [{ _multiSheet: true, data: { "Sheet1": [[...]], "Sheet2": [[...]] } }]
            if let Some(multi_sheet_obj) = data[0].as_object() {
                if let Some(sheets_data) = multi_sheet_obj.get("data") {
                    if let Some(sheets_obj) = sheets_data.as_object() {
                        // Convert each sheet according to the format option
                        let mut result = serde_json::Map::new();
                        
                        for (sheet_name, sheet_data) in sheets_obj {
                            if let Some(sheet_array) = sheet_data.as_array() {
                                // Convert Value array to Vec<Vec<Value>>
                                let sheet_rows = collect_sheet_rows(sheet_array);
                                
                                if !sheet_rows.is_empty() {
                                    let formatted_sheet = format_sheet_data(&sheet_rows, config.options.json_format.as_str(), config.options.include_headers)?;
                                    result.insert(sheet_name.clone(), formatted_sheet);
                                }
                            }
                        }
                        
                        Value::Object(result)
                    } else {
                        return Err("Invalid multi-sheet data structure: expected object".to_string());
                    }
                } else {
                    return Err("Multi-sheet data missing 'data' field".to_string());
                }
            } else {
                return Err("Invalid multi-sheet object".to_string());
            }
        }
        DataStructure::MultiSheetArray => {
            // Multi-sheet format: [{ name: "Sheet1", data: [[...]] }, { name: "Sheet2", data: [[...]] }]
            let mut result = serde_json::Map::new();
            
            for (sheet_index, sheet_value) in data.iter().enumerate() {
                if let Some(sheet_obj) = sheet_value.as_object() {
                    let sheet_name = if let Some(name_val) = sheet_obj.get("name") {
                        if let Some(name_str) = name_val.as_str() {
                            if !name_str.trim().is_empty() {
                                name_str.to_string()
                            } else {
                                format!("Sheet{}", sheet_index + 1)
                            }
                        } else {
                            format!("Sheet{}", sheet_index + 1)
                        }
                    } else {
                        format!("Sheet{}", sheet_index + 1)
                    };
                    
                    if let Some(sheet_data) = sheet_obj.get("data") {
                        if let Some(sheet_array) = sheet_data.as_array() {
                            // Convert Value array to Vec<Vec<Value>>
                            let sheet_rows = collect_sheet_rows(sheet_array);
                            
                            if !sheet_rows.is_empty() {
                                let formatted_sheet = format_sheet_data(&sheet_rows, config.options.json_format.as_str(), config.options.include_headers)?;
                                result.insert(sheet_name.to_string(), formatted_sheet);
                            }
                        }
                    }
                }
            }
            
            Value::Object(result)
        }
        DataStructure::Array2D => {
            // Single sheet data: convert Vec<Value> to Vec<Vec<Value>>
            let sheet_rows = collect_sheet_rows(&data);
            
            if sheet_rows.is_empty() {
                return Err("No valid data rows found".to_string());
            }
            
            // Convert data based on format option
            format_sheet_data(&sheet_rows, config.options.json_format.as_str(), config.options.include_headers)?
        }
    };

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
/// 
/// User controls via include_headers:
/// - true: Use first row as column names
/// - false: Auto-generate column names (Column1, Column2, etc.)
fn format_as_object(data: &[Vec<Value>], has_headers: bool) -> Result<Value, String> {
    if data.is_empty() {
        return Ok(json!({}));
    }

    let (headers, data_rows) = if has_headers && data.len() > 1 {
        // User explicitly said: use first row as column names
        // RESPECT their choice - no detection, no override
        let headers: Vec<String> = data[0].iter()
            .map(value_to_string)
            .collect();
        (headers, &data[1..])
    } else {
        // User said no headers - auto-generate column names
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
///
/// User controls via include_headers:
/// - true: Use first row as column names
/// - false: Auto-generate column names (Column1, Column2, etc.)
fn format_as_records(data: &[Vec<Value>], has_headers: bool) -> Result<Value, String> {
    if data.is_empty() {
        return Ok(json!([]));
    }

    let (headers, data_rows) = if has_headers && data.len() > 1 {
        // User explicitly said: use first row as column names
        // RESPECT their choice - no detection, no override
        let headers: Vec<String> = data[0].iter()
            .map(value_to_string)
            .collect();
        (headers, &data[1..])
    } else {
        // User said no headers - auto-generate column names
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
