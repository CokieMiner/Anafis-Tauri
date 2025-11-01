// Apache Parquet (.parquet) export
//
// Handles exporting data to Apache Parquet format.
// Uses the polars crate for efficient columnar storage.

use std::fs::File;
use serde_json::Value;
use polars::prelude::*;
use super::{ExportConfig, ExportFormat, DataStructure};

/// Export data to Apache Parquet (.parquet) format
#[tauri::command]
pub async fn export_to_parquet(
    data: Vec<serde_json::Value>,
    file_path: String,
    config: ExportConfig,
) -> Result<(), String> {
    // Validate format
    if !matches!(config.format, ExportFormat::Parquet) {
        return Err("Invalid format for Parquet export".to_string());
    }

    // Validate data structure - Parquet only supports single-sheet 2D arrays
    if !matches!(config.data_structure, DataStructure::Array2D) {
        return Err(format!(
            "Parquet export only supports single-sheet data (Array2D). Received: {:?}. Please export each sheet separately.",
            config.data_structure
        ));
    }

    // Determine the maximum number of columns
    let max_cols = data.iter()
        .filter_map(|row| row.as_array())
        .map(|arr| arr.len())
        .max()
        .unwrap_or(0);

    if max_cols == 0 {
        return Err("No data to export".to_string());
    }

    // Create columns as Series
    let mut series_vec: Vec<polars::prelude::Series> = Vec::new();

    for col_idx in 0..max_cols {
        let mut string_values: Vec<String> = Vec::new();

        for row in &data {
            if let Some(row_array) = row.as_array() {
                if col_idx < row_array.len() {
                    let cell = &row_array[col_idx];
                    let str_value = match cell {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "".to_string(),
                        _ => cell.to_string(),
                    };
                    string_values.push(str_value);
                } else {
                    string_values.push("".to_string());
                }
            } else {
                string_values.push("".to_string());
            }
        }

        // Create series name
        // Parquet: Always auto-generate column names to preserve all data
        // include_headers is ignored for Parquet (columnar format with metadata)
        let series_name = format!("column_{}", col_idx + 1);

        // Create string series
        let series = Series::new(PlSmallStr::from(&series_name), string_values);
        series_vec.push(series);
    }

    // Create DataFrame from series
    let columns: Vec<Column> = series_vec.into_iter()
        .map(|s| s.into())
        .collect();
    let mut df = DataFrame::new(columns)
        .map_err(|e| format!("Failed to create DataFrame: {}", e))?;

    // Write to Parquet file
    let file = File::create(&file_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    ParquetWriter::new(file)
        .finish(&mut df)
        .map_err(|e| format!("Failed to write Parquet file: {}", e))?;

    Ok(())
}