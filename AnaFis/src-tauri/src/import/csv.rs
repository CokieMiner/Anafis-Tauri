// CSV/TSV/TXT import handler
//
// Imports delimited text files and converts them to Univer-compatible workbook data.
// Supports:
// - CSV (comma-separated)
// - TSV (tab-separated)
// - TXT (custom delimiter)
// - Various encodings (UTF-8, Latin1, etc.)
// - First row as header option
// - Skip rows option

use std::fs::File;
use std::io::{BufRead, BufReader};
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use std::collections::HashMap;
use serde_json::Value;
use super::ImportResponse;

/// Detect file encoding by reading the first few bytes
fn detect_encoding(file_path: &str) -> Result<&'static Encoding, String> {
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0u8; 1024];
    let bytes_read = reader.read_until(b'\n', &mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Try to detect encoding based on BOM or content
    if bytes_read >= 3 && buffer[0] == 0xEF && buffer[1] == 0xBB && buffer[2] == 0xBF {
        return Ok(UTF_8); // UTF-8 with BOM
    }
    
    // Check if it's valid UTF-8
    if std::str::from_utf8(&buffer[..bytes_read]).is_ok() {
        return Ok(UTF_8);
    }
    
    // Default to Windows-1252 (common for legacy files)
    Ok(WINDOWS_1252)
}

/// Parse a CSV file with custom delimiter
pub fn parse_delimited_file(
    file_path: &str,
    delimiter: char,
    skip_rows: usize,
    first_row_as_header: bool,
    encoding_name: Option<&str>,
) -> Result<ImportResponse, String> {
    // Determine encoding
    let encoding = if let Some(enc_name) = encoding_name {
        match enc_name.to_lowercase().as_str() {
            "utf-8" | "utf8" => UTF_8,
            "latin1" | "iso-8859-1" | "windows-1252" | "cp1252" => WINDOWS_1252,
            _ => detect_encoding(file_path)?,
        }
    } else {
        detect_encoding(file_path)?
    };

    // Read file with detected encoding
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let mut reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut buffer = Vec::new();
    
    while reader.read_until(b'\n', &mut buffer).map_err(|e| format!("Failed to read file: {}", e))? > 0 {
        let (decoded, _, had_errors) = encoding.decode(&buffer);
        if had_errors {
            return Err("Encoding error: file contains invalid characters".to_string());
        }
        lines.push(decoded.into_owned());
        buffer.clear();
    }

    // Skip rows if requested
    let lines: Vec<String> = lines.into_iter().skip(skip_rows).collect();
    
    if lines.is_empty() {
        return Err("File is empty or all rows were skipped".to_string());
    }

    // Parse CSV rows
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut max_columns = 0;

    for line in &lines {
        let line = line.trim_end_matches(&['\r', '\n'][..]);
        if line.is_empty() {
            continue;
        }

        // Simple CSV parsing (handles quoted fields)
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes {
                        // Check for escaped quote
                        if chars.peek() == Some(&'"') {
                            current_field.push('"');
                            chars.next();
                        } else {
                            in_quotes = false;
                        }
                    } else {
                        in_quotes = true;
                    }
                }
                c if c == delimiter && !in_quotes => {
                    fields.push(current_field.clone());
                    current_field.clear();
                }
                c => {
                    current_field.push(c);
                }
            }
        }
        fields.push(current_field);

        max_columns = max_columns.max(fields.len());
        rows.push(fields);
    }

    // If first row as header, skip it (we'll handle headers in the frontend)
    let data_start = if first_row_as_header && !rows.is_empty() { 1 } else { 0 };
    
    // Convert to JSON values
    let mut sheet_data: Vec<Vec<Value>> = Vec::new();
    for row in rows.iter().skip(data_start) {
        let mut json_row = Vec::new();
        for field in row {
            // Try to parse as number
            if let Ok(num) = field.parse::<f64>() {
                json_row.push(Value::Number(serde_json::Number::from_f64(num).unwrap_or_else(|| {
                    serde_json::Number::from(0)
                })));
            } else {
                json_row.push(Value::String(field.clone()));
            }
        }
        // Pad row to max_columns
        while json_row.len() < max_columns {
            json_row.push(Value::Null);
        }
        sheet_data.push(json_row);
    }

    // Create response
    let mut sheets = HashMap::new();
    sheets.insert("Sheet1".to_string(), sheet_data);

    Ok(ImportResponse { sheets })
}

/// Import CSV file
pub async fn import_csv(
    file_path: &str,
    skip_rows: usize,
    first_row_as_header: bool,
    encoding: Option<&str>,
) -> Result<ImportResponse, String> {
    parse_delimited_file(file_path, ',', skip_rows, first_row_as_header, encoding)
}

/// Import TSV file
pub async fn import_tsv(
    file_path: &str,
    skip_rows: usize,
    first_row_as_header: bool,
    encoding: Option<&str>,
) -> Result<ImportResponse, String> {
    parse_delimited_file(file_path, '\t', skip_rows, first_row_as_header, encoding)
}

/// Import TXT file with custom delimiter
pub async fn import_txt(
    file_path: &str,
    delimiter: &str,
    skip_rows: usize,
    first_row_as_header: bool,
    encoding: Option<&str>,
) -> Result<ImportResponse, String> {
    let delim_char = delimiter.chars().next()
        .ok_or("Delimiter must be at least one character")?;
    parse_delimited_file(file_path, delim_char, skip_rows, first_row_as_header, encoding)
}
