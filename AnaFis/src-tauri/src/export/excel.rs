// Excel format export (.xlsx)
//
// Handles exporting data to Excel format using rust_xlsxwriter.
// Supports formatting, multiple sheets, and Excel-specific features.

use serde_json::Value;
use rust_xlsxwriter::{Workbook, Format, Color};
use super::{ExportConfig, ExportFormat, DataStructure};

/// Export data to Excel (.xlsx) format
#[tauri::command]
pub async fn export_to_excel(
    data: Vec<serde_json::Value>,
    file_path: String,
    config: ExportConfig,
) -> Result<(), String> {
    // Validate format
    if !matches!(config.format, ExportFormat::Xlsx) {
        return Err("Invalid format for Excel export".to_string());
    }

    // Create a new Excel file
    let mut workbook = Workbook::new();

    // Use explicit data structure marker instead of implicit detection
    match config.data_structure {
        DataStructure::MultiSheetArray => {
            // Multi-sheet data: [{name: "Sheet1", data: [[...], [...]]}, ...]
            for sheet_value in data {
                if let Some(sheet_obj) = sheet_value.as_object() {
                    let sheet_name = sheet_obj.get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Sheet");
                    let sheet_data = sheet_obj.get("data")
                        .and_then(|d| d.as_array())
                        .ok_or_else(|| format!("Invalid sheet data structure for sheet '{}'", sheet_name))?;

                    // Create worksheet with name
                    let worksheet = workbook.add_worksheet();
                    worksheet.set_name(sheet_name).map_err(|e| format!("Failed to set worksheet name '{}': {}", sheet_name, e))?;

                    // Write sheet data
                    write_sheet_data(worksheet, sheet_data, &config)?;
                }
            }
        }
        DataStructure::Array2D => {
            // Single sheet data: [[...], [...]]
            let worksheet = workbook.add_worksheet();
            write_sheet_data(worksheet, &data, &config)?;
        }
        DataStructure::MultiSheetJson => {
            return Err("Excel export does not support MultiSheetJson data structure. Use MultiSheetArray format instead.".to_string());
        }
    }

    // Save the file
    workbook.save(&file_path).map_err(|e| format!("Failed to save Excel file: {}", e))?;

    Ok(())
}

/// Helper function to write data to a worksheet
fn write_sheet_data(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    data: &[serde_json::Value],
    config: &ExportConfig,
) -> Result<(), String> {

    // Create formats
    let data_format = Format::new();

    // Check if this is rich export (contains CellValue objects)
    let is_rich_export = config.options.include_formulas || config.options.include_formatting || config.options.include_metadata;

    // Write data rows
    for (row_idx, row) in data.iter().enumerate() {
        // Get row as array
        let row_array = match row.as_array() {
            Some(arr) => arr,
            None => continue, // Skip non-array rows
        };

        // Skip completely empty rows
        if row_array.iter().all(|cell| matches!(cell, Value::Null) || cell.as_str() == Some("")) {
            continue;
        }

        for (col_idx, cell) in row_array.iter().enumerate() {
            // Excel export: all rows are data, no special header handling
            let format = &data_format;

            if is_rich_export {
                // Handle CellValue objects for rich export (data + formulas + styling)
                if let Some(cell_obj) = cell.as_object() {
                    // Extract value
                    let value = cell_obj.get("v");
                    // Extract formula
                    let formula = cell_obj.get("f");
                    // Extract style
                    let style = cell_obj.get("style");

                    // Create format from style if available
                    let cell_format = if let Some(style_val) = style {
                        create_format_from_style(style_val)
                    } else {
                        data_format.clone()
                    };

                    if let Some(formula) = formula.and_then(|f| f.as_str()) {
                        // Check if formulas should be included based on config
                        if config.options.include_formulas {
                            // Write formula with format
                            worksheet.write_formula_with_format(row_idx as u32, col_idx as u16, formula, &cell_format)
                                .map_err(|e| format!("Failed to write formula cell: {}", e))?;
                        } else {
                            // Write evaluated value instead of formula
                            if let Some(val) = value {
                                write_cell_with_format(worksheet, row_idx as u32, col_idx as u16, val, &cell_format)?;
                            } else {
                                // No value and formula not included - write blank
                                worksheet.write_blank(row_idx as u32, col_idx as u16, &cell_format)
                                    .map_err(|e| format!("Failed to write blank cell: {}", e))?;
                            }
                        }
                    } else if let Some(val) = value {
                        write_cell_with_format(worksheet, row_idx as u32, col_idx as u16, val, &cell_format)?;
                    } else {
                        // Empty cell with format
                        worksheet.write_blank(row_idx as u32, col_idx as u16, &cell_format)
                            .map_err(|e| format!("Failed to write blank cell: {}", e))?;
                    }
                } else {
                    // Not a CellValue object, treat as simple value
                    // Original logic for basic export
                let cell_format = &data_format;
                    write_cell_with_format(worksheet, row_idx as u32, col_idx as u16, cell, cell_format)?;
                }
            } else {
                // Original logic for basic export
                match cell {
                    Value::String(s) => {
                        worksheet.write_string_with_format(row_idx as u32, col_idx as u16, s, format)
                            .map_err(|e| format!("Failed to write string cell: {}", e))?;
                    }
                    Value::Number(n) => {
                        if let Some(num) = n.as_f64() {
                            worksheet.write_number_with_format(row_idx as u32, col_idx as u16, num, format)
                                .map_err(|e| format!("Failed to write number cell: {}", e))?;
                        } else {
                            worksheet.write_string_with_format(row_idx as u32, col_idx as u16, n.to_string(), format)
                                .map_err(|e| format!("Failed to write number cell: {}", e))?;
                        }
                    }
                    Value::Bool(b) => {
                        worksheet.write_boolean_with_format(row_idx as u32, col_idx as u16, *b, format)
                            .map_err(|e| format!("Failed to write boolean cell: {}", e))?;
                    }
                    Value::Null => {
                        // Write empty cell
                        worksheet.write_blank(row_idx as u32, col_idx as u16, format)
                            .map_err(|e| format!("Failed to write empty cell: {}", e))?;
                    }
                    _ => {
                        // Write as string for other types
                        worksheet.write_string_with_format(row_idx as u32, col_idx as u16, cell.to_string(), format)
                            .map_err(|e| format!("Failed to write cell: {}", e))?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Helper function to write a cell with format
fn write_cell_with_format(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    value: &serde_json::Value,
    format: &Format,
) -> Result<(), String> {
    match value {
        Value::String(s) => {
            worksheet.write_string_with_format(row, col, s, format)
                .map_err(|e| format!("Failed to write string cell: {}", e))?;
        }
        Value::Number(n) => {
            if let Some(num) = n.as_f64() {
                worksheet.write_number_with_format(row, col, num, format)
                    .map_err(|e| format!("Failed to write number cell: {}", e))?;
            } else {
                worksheet.write_string_with_format(row, col, n.to_string(), format)
                    .map_err(|e| format!("Failed to write number cell: {}", e))?;
            }
        }
        Value::Bool(b) => {
            worksheet.write_boolean_with_format(row, col, *b, format)
                .map_err(|e| format!("Failed to write boolean cell: {}", e))?;
        }
        Value::Null => {
            worksheet.write_blank(row, col, format)
                .map_err(|e| format!("Failed to write blank cell: {}", e))?;
        }
        _ => {
            // Write as string for other types
            worksheet.write_string_with_format(row, col, value.to_string(), format)
                .map_err(|e| format!("Failed to write cell: {}", e))?;
        }
    }

    Ok(())
}

/// Helper function to create format from style object
fn create_format_from_style(style: &serde_json::Value) -> Format {
    let mut format = Format::new();

    if let Some(style_obj) = style.as_object() {
        // Extract background color
        // Background color can appear under several keys depending on source
        let bg_candidates = ["bg", "background", "backgroundColor", "bgColor", "fill", "bgcolor", "background_color"];
        for key in bg_candidates.iter() {
            if let Some(bg) = style_obj.get(*key) {
                // Try nested rgb field first
                if let Some(bg_obj) = bg.as_object() {
                    if let Some(rgb_val) = bg_obj.get("rgb").or_else(|| bg_obj.get("color")).or_else(|| bg_obj.get("value")) {
                        if let Some(color_str) = rgb_val.as_str() {
                            if let Some(color) = parse_rgb_color(color_str) {
                                format = format.set_background_color(color);
                                break;
                            }
                        }
                    }
                }

                // Try direct string value
                if let Some(color_str) = bg.as_str() {
                    if let Some(color) = parse_rgb_color(color_str) {
                        format = format.set_background_color(color);
                        break;
                    }
                }
            }
        }

        // Extract text / font color. Support several key names.
        let fg_candidates = ["cl", "color", "fontColor", "fgColor", "font_color", "fontcolor"];
        for key in fg_candidates.iter() {
            if let Some(cl) = style_obj.get(*key) {
                if let Some(cl_obj) = cl.as_object() {
                    if let Some(rgb_val) = cl_obj.get("rgb").or_else(|| cl_obj.get("color")).or_else(|| cl_obj.get("value")) {
                        if let Some(color_str) = rgb_val.as_str() {
                            if let Some(color) = parse_rgb_color(color_str) {
                                format = format.set_font_color(color);
                                break;
                            }
                        }
                    }
                }

                if let Some(color_str) = cl.as_str() {
                    if let Some(color) = parse_rgb_color(color_str) {
                        format = format.set_font_color(color);
                        break;
                    }
                }
            }
        }

        // Extract bold
        if let Some(bl) = style_obj.get("bl") {
            if let Some(bl_num) = bl.as_i64() {
                if bl_num == 1 {
                    format = format.set_bold();
                }
            }
        }

        // Extract italic
        if let Some(it) = style_obj.get("it") {
            if let Some(it_num) = it.as_i64() {
                if it_num == 1 {
                    format = format.set_italic();
                }
            }
        }

        // Extract font size
        if let Some(fs) = style_obj.get("fs") {
            if let Some(font_size) = fs.as_f64() {
                format = format.set_font_size(font_size);
            }
        }

        // Extract font family
        if let Some(ff) = style_obj.get("ff") {
            if let Some(font_family) = ff.as_str() {
                format = format.set_font_name(font_family);
            }
        }
    }

    format
}

/// Helper function to parse RGB color string like "rgb(0, 0, 255)" or "#0000FF"
fn parse_rgb_color(color_str: &str) -> Option<Color> {
    let s = color_str.trim();

    // rgba(...) or rgb(...)
    if (s.starts_with("rgb(") && s.ends_with(")")) || (s.starts_with("rgba(") && s.ends_with(")")) {
        let inner = s.split_once('(')?.1.trim_end_matches(')');
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].trim().parse::<u8>(),
                parts[1].trim().parse::<u8>(),
                parts[2].trim().parse::<u8>()
            ) {
                return Some(Color::RGB((r as u32) << 16 | (g as u32) << 8 | b as u32));
            }
        }
    }

    // Hex like #RRGGBB or RRGGBB
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.len() == 6 {
        if let Ok(color_val) = u32::from_str_radix(hex, 16) {
            return Some(Color::RGB(color_val));
        }
    }

    // ARGB like AARRGGBB (take last 6 chars as RRGGBB)
    if hex.len() == 8 {
        if let Ok(color_val) = u32::from_str_radix(&hex[2..], 16) {
            return Some(Color::RGB(color_val));
        }
    }

    // Decimal integer color (e.g., 16711680)
    if let Ok(num) = s.parse::<u32>() {
        return Some(Color::RGB(num));
    }

    // Failed to parse - log warning for debugging
    eprintln!("Warning: Failed to parse color format '{}' - using default color", color_str);
    None
}