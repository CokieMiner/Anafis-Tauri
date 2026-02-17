// Apache Parquet (.parquet) export - using arrow directly
//
// Exports data to Apache Parquet columnar format (2D array)

use super::ExportConfig;
use arrow::array::{ArrayRef, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use serde_json::Value;
use std::fs::File;
use std::sync::Arc;

/// Export data to Apache Parquet (.parquet) format (simplified - expects 2D array)
#[tauri::command]
pub async fn export_to_parquet(
    data: Vec<serde_json::Value>,
    file_path: String,
    _config: ExportConfig,
) -> Result<(), String> {
    // Determine the maximum number of columns
    let max_cols = data
        .iter()
        .filter_map(|row| row.as_array())
        .map(|arr| arr.len())
        .max()
        .unwrap_or(0);

    if max_cols == 0 {
        return Err("No data to export".to_string());
    }

    let num_rows = data.len();

    // Create schema with string columns
    let mut fields = Vec::new();
    for col_idx in 0..max_cols {
        let field_name = format!("column_{}", col_idx + 1);
        fields.push(Field::new(field_name, DataType::Utf8, true));
    }
    let schema = Arc::new(Schema::new(fields));

    // Create column arrays
    let mut columns: Vec<ArrayRef> = Vec::new();

    for col_idx in 0..max_cols {
        let mut string_values: Vec<Option<String>> = Vec::with_capacity(num_rows);

        for row in &data {
            if let Some(row_array) = row.as_array() {
                if col_idx < row_array.len() {
                    let cell = &row_array[col_idx];
                    let str_value = match cell {
                        Value::String(s) => Some(s.clone()),
                        Value::Number(n) => Some(n.to_string()),
                        Value::Bool(b) => Some(b.to_string()),
                        Value::Null => None,
                        _ => Some(cell.to_string()),
                    };
                    string_values.push(str_value);
                } else {
                    string_values.push(None);
                }
            } else {
                string_values.push(None);
            }
        }

        // Create StringArray from values
        let array = StringArray::from(string_values);
        columns.push(Arc::new(array) as ArrayRef);
    }

    // Create RecordBatch
    let batch = RecordBatch::try_new(schema.clone(), columns)
        .map_err(|e| format!("Failed to create RecordBatch: {}", e))?;

    // Write to Parquet file
    let file = File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;

    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))
        .map_err(|e| format!("Failed to create Parquet writer: {}", e))?;

    writer
        .write(&batch)
        .map_err(|e| format!("Failed to write RecordBatch: {}", e))?;

    writer
        .close()
        .map_err(|e| format!("Failed to close Parquet writer: {}", e))?;

    Ok(())
}
