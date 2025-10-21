// AnaFis Spreadsheet (.anafispread) export
//
// Custom JSON-based format for AnaFis spreadsheet data.
// Preserves everything: cell values, formulas, formatting, metadata, uncertainties.

use std::fs::File;
use serde_json::json;
use super::{ExportConfig, ExportFormat};

/// Export data to AnaFis Spreadsheet (.anafispread) format
#[tauri::command]
pub async fn export_to_anafis_spread(
    data: Vec<serde_json::Value>,
    file_path: String,
    config: ExportConfig,
) -> Result<(), String> {
    // Validate format
    if !matches!(config.format, ExportFormat::AnaFisSpread) {
        return Err("Invalid format for AnaFis Spreadsheet export".to_string());
    }

    // Filter data based on export options
    let filtered_data = filter_data_by_options(&data, &config);

    // Create comprehensive export structure
    let export_data = json!({
        "version": "1.0",
        "format": "anafis_spreadsheet",
        "metadata": {
            "created": chrono::Utc::now().to_rfc3339(),
            "export_options": {
                "include_headers": config.options.include_headers,
                "include_formulas": config.options.include_formulas,
                "include_formatting": config.options.include_formatting,
                "include_metadata": config.options.include_metadata
            }
        },
        "data": filtered_data
    });

    // Write to file with optional compression
    let file = File::create(&file_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    if config.options.compress {
        // TODO: Implement gzip compression
        // For now, write uncompressed
        serde_json::to_writer_pretty(file, &export_data)
            .map_err(|e| format!("Failed to write AnaFis Spreadsheet file: {}", e))?;
    } else {
        serde_json::to_writer_pretty(file, &export_data)
            .map_err(|e| format!("Failed to write AnaFis Spreadsheet file: {}", e))?;
    }

    Ok(())
}

/// Filter data based on export options
fn filter_data_by_options(data: &Vec<serde_json::Value>, config: &ExportConfig) -> Vec<serde_json::Value> {
    data.iter().map(|item| {
        if let Some(sheet_obj) = item.as_object() {
            // Multi-sheet format: { name: string, data: array }
            if let (Some(name), Some(sheet_data)) = (sheet_obj.get("name"), sheet_obj.get("data")) {
                if let Some(data_array) = sheet_data.as_array() {
                    let filtered_sheet_data = filter_sheet_data(data_array, config);
                    let mut new_sheet_obj = serde_json::Map::new();
                    new_sheet_obj.insert("name".to_string(), name.clone());
                    new_sheet_obj.insert("data".to_string(), serde_json::Value::Array(filtered_sheet_data));
                    serde_json::Value::Object(new_sheet_obj)
                } else {
                    item.clone()
                }
            } else {
                item.clone()
            }
        } else if let Some(data_array) = item.as_array() {
            // Single sheet format: array of arrays
            let filtered_data = filter_sheet_data(data_array, config);
            serde_json::Value::Array(filtered_data)
        } else {
            item.clone()
        }
    }).collect()
}

/// Filter sheet data (array of rows) based on export options
fn filter_sheet_data(data: &Vec<serde_json::Value>, config: &ExportConfig) -> Vec<serde_json::Value> {
    data.iter().map(|row| {
        if let Some(row_array) = row.as_array() {
            let filtered_row: Vec<serde_json::Value> = row_array.iter().map(|cell| {
                filter_cell(cell, config)
            }).collect();
            serde_json::Value::Array(filtered_row)
        } else {
            row.clone()
        }
    }).collect()
}

/// Filter individual cell based on export options
fn filter_cell(cell: &serde_json::Value, config: &ExportConfig) -> serde_json::Value {
    if let Some(cell_obj) = cell.as_object() {
        let mut filtered_cell = serde_json::Map::new();

        // Always include value
        if let Some(v) = cell_obj.get("v") {
            filtered_cell.insert("v".to_string(), v.clone());
        }

        // Include formula only if requested
        if config.options.include_formulas {
            if let Some(f) = cell_obj.get("f") {
                filtered_cell.insert("f".to_string(), f.clone());
            }
        }

        // Include formatting only if requested
        if config.options.include_formatting {
            if let Some(style) = cell_obj.get("style") {
                filtered_cell.insert("style".to_string(), style.clone());
            }
        }

        // Include metadata only if requested
        if config.options.include_metadata {
            if let Some(meta) = cell_obj.get("meta") {
                filtered_cell.insert("meta".to_string(), meta.clone());
            }
        }

        if filtered_cell.is_empty() {
            // If no data to include, return the original value if it's just a simple value
            if cell_obj.contains_key("v") && cell_obj.len() == 1 {
                cell_obj.get("v").unwrap_or(cell).clone()
            } else {
                serde_json::Value::Object(filtered_cell)
            }
        } else {
            serde_json::Value::Object(filtered_cell)
        }
    } else {
        // Simple value cell
        cell.clone()
    }
}