// Parquet import handler
//
// Imports Apache Parquet files and converts them to Univer-compatible workbook data.
// Parquet is a columnar binary format commonly used for data science and analytics.

use super::ImportResponse;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Row;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;

/// Import Parquet file
pub fn import_parquet(file_path: &str) -> Result<ImportResponse, String> {
    // Open the parquet file
    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {e}"))?;

    let reader =
        SerializedFileReader::new(file).map_err(|e| format!("Failed to read parquet file: {e}"))?;

    // Get row iterator
    let row_iter = reader
        .get_row_iter(None)
        .map_err(|e| format!("Failed to get row iterator: {e}"))?;

    // Get schema to determine columns
    let schema = reader.metadata().file_metadata().schema();
    let num_columns = schema.get_fields().len();
    let column_names: Vec<String> = schema
        .get_fields()
        .iter()
        .map(|f| f.name().to_string())
        .collect();

    // Read all rows
    let mut sheet_data: Vec<Vec<Value>> = Vec::new();

    // Add header row with column names
    let header_row: Vec<Value> = column_names
        .iter()
        .map(|name| Value::String(name.clone()))
        .collect();
    sheet_data.push(header_row);

    // Read data rows
    for row in row_iter {
        let row = row.map_err(|e| format!("Failed to read row: {e}"))?;
        let json_row = convert_parquet_row_to_json(&row, num_columns)?;
        sheet_data.push(json_row);
    }

    // Create response
    let mut sheets = HashMap::new();
    sheets.insert("Sheet1".to_string(), sheet_data);

    Ok(ImportResponse { sheets })
}

/// Convert a Parquet row to JSON values
fn convert_parquet_row_to_json(row: &Row, num_columns: usize) -> Result<Vec<Value>, String> {
    let mut json_row = Vec::new();

    for i in 0..num_columns {
        let field = row
            .get_column_iter()
            .nth(i)
            .ok_or_else(|| format!("Column {i} not found"))?;

        let value = match field.1 {
            parquet::record::Field::Null => Value::Null,
            parquet::record::Field::Bool(b) => Value::Bool(*b),
            parquet::record::Field::Byte(b) => Value::Number(i64::from(*b).into()),
            parquet::record::Field::Short(s) => Value::Number(i64::from(*s).into()),
            parquet::record::Field::Int(i) => Value::Number(i64::from(*i).into()),
            parquet::record::Field::Long(l) => Value::Number((*l).into()),
            parquet::record::Field::UByte(b) => Value::Number(u64::from(*b).into()),
            parquet::record::Field::UShort(s) => Value::Number(u64::from(*s).into()),
            parquet::record::Field::UInt(i) => Value::Number(u64::from(*i).into()),
            parquet::record::Field::ULong(l) => Value::Number((*l).into()),
            parquet::record::Field::Float16(f) => Value::Number(
                serde_json::Number::from_f64(f.to_f64())
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ),
            parquet::record::Field::Float(f) => Value::Number(
                serde_json::Number::from_f64(f64::from(*f))
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ),
            parquet::record::Field::Double(d) => Value::Number(
                serde_json::Number::from_f64(*d).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
            parquet::record::Field::Str(s) => Value::String(s.clone()),
            parquet::record::Field::Bytes(b) => {
                // Convert bytes to hex string more efficiently
                use std::fmt::Write;
                let data = b.data();
                let mut hex_str = String::with_capacity(data.len() * 2);
                for byte in data {
                    let _ = write!(hex_str, "{byte:02x}");
                }
                Value::String(format!("0x{hex_str}"))
            }
            parquet::record::Field::Decimal(d) => {
                // Decimal doesn't implement Display, so format it manually
                Value::String(format!("{d:?}"))
            }
            parquet::record::Field::TimestampMillis(ts)
            | parquet::record::Field::TimestampMicros(ts) => Value::Number((*ts).into()),
            parquet::record::Field::TimeMillis(t) => Value::Number(i64::from(*t).into()),
            parquet::record::Field::TimeMicros(t) => Value::Number((*t).into()),
            parquet::record::Field::Date(d) => Value::Number(i64::from(*d).into()),
            parquet::record::Field::Group(g) => {
                // Complex nested type - convert to JSON string representation
                Value::String(format!("{g:?}"))
            }
            parquet::record::Field::ListInternal(l) => {
                // List type - convert to JSON string representation
                Value::String(format!("{l:?}"))
            }
            parquet::record::Field::MapInternal(m) => {
                // Map type - convert to JSON string representation
                Value::String(format!("{m:?}"))
            }
        };

        json_row.push(value);
    }

    Ok(json_row)
}
