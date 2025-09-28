use crate::spreadsheet::core::SpreadsheetEngine;
use crate::spreadsheet::types::*;
use std::collections::HashMap;
use tauri::command;

#[command]
pub async fn set_spreadsheet_cell_value(cell_ref: String, content: String) -> Result<(), String> {
    SpreadsheetEngine::set_cell_value(&cell_ref, content)
}

#[command]
pub async fn get_spreadsheet_cell_value(cell_ref: String) -> Result<Option<UnifiedCell>, String> {
    SpreadsheetEngine::get_cell_value(&cell_ref)
}

#[command]
pub async fn set_spreadsheet_active_cell(row: usize, col: usize) -> Result<(), String> {
    SpreadsheetEngine::set_active_cell(row, col)
}

#[command]
pub async fn get_spreadsheet_active_cell() -> Result<Option<CellReference>, String> {
    SpreadsheetEngine::get_active_cell()
}

#[command]
pub async fn get_spreadsheet_state() -> Result<SpreadsheetState, String> {
    let state = SpreadsheetEngine::get_state();
    let state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(state.clone())
}

// ===== CELL RANGE OPERATIONS =====

#[command]
pub async fn set_spreadsheet_selection(start_row: usize, start_col: usize, end_row: usize, end_col: usize) -> Result<(), String> {
    SpreadsheetEngine::set_selection(start_row, start_col, end_row, end_col)
}

#[command]
pub async fn get_spreadsheet_selection() -> Result<Option<CellRange>, String> {
    SpreadsheetEngine::get_selection()
}

#[command]
pub async fn clear_spreadsheet_selection() -> Result<(), String> {
    SpreadsheetEngine::clear_selection()
}

#[command]
pub async fn apply_formula_to_range(range_str: String, formula_template: String) -> Result<Vec<String>, String> {
    let range = SpreadsheetEngine::parse_cell_range(&range_str)?;
    SpreadsheetEngine::apply_formula_to_range(&range, &formula_template)
}

#[command]
pub async fn get_range_cells(range_str: String) -> Result<HashMap<String, UnifiedCell>, String> {
    let range = SpreadsheetEngine::parse_cell_range(&range_str)?;
    SpreadsheetEngine::get_range_cells(&range)
}

#[command]
pub async fn clear_range(range_str: String) -> Result<(), String> {
    let range = SpreadsheetEngine::parse_cell_range(&range_str)?;
    SpreadsheetEngine::clear_range(&range)
}

#[command]
pub async fn delete_range(range_str: String) -> Result<(), String> {
    let range = SpreadsheetEngine::parse_cell_range(&range_str)?;
    SpreadsheetEngine::delete_range(&range)
}

// ===== DEPENDENCY TRACKING =====

#[command]
pub async fn add_dependency(dependent: String, dependency: String) -> Result<(), String> {
    SpreadsheetEngine::add_dependency(&dependent, &dependency)
}

#[command]
pub async fn remove_dependency(dependent: String, dependency: String) -> Result<(), String> {
    SpreadsheetEngine::remove_dependency(&dependent, &dependency)
}

#[command]
pub async fn get_dependencies(cell_ref: String) -> Result<Vec<String>, String> {
    SpreadsheetEngine::get_dependencies(&cell_ref)
}

#[command]
pub async fn get_dependents(cell_ref: String) -> Result<Vec<String>, String> {
    SpreadsheetEngine::get_dependents(&cell_ref)
}

#[command]
pub async fn check_circular_reference(cell_ref: String) -> Result<Option<Vec<String>>, String> {
    SpreadsheetEngine::check_circular_reference(&cell_ref)
}

#[command]
pub async fn get_calculation_order() -> Result<Vec<String>, String> {
    SpreadsheetEngine::get_calculation_order()
}

// ===== UNDO/REDO FUNCTIONALITY =====

#[command]
pub async fn create_spreadsheet_snapshot(description: String) -> Result<(), String> {
    SpreadsheetEngine::create_snapshot(description)
}

#[command]
pub async fn undo_spreadsheet() -> Result<bool, String> {
    SpreadsheetEngine::undo()
}

#[command]
pub async fn redo_spreadsheet() -> Result<bool, String> {
    SpreadsheetEngine::redo()
}

#[command]
pub async fn get_version_history() -> Result<Vec<SpreadsheetSnapshot>, String> {
    SpreadsheetEngine::get_version_history()
}

#[command]
pub async fn get_current_version() -> Result<usize, String> {
    SpreadsheetEngine::get_current_version()
}

#[command]
pub async fn can_undo() -> Result<bool, String> {
    SpreadsheetEngine::can_undo()
}

#[command]
pub async fn can_redo() -> Result<bool, String> {
    SpreadsheetEngine::can_redo()
}

// ===== CELL REFERENCE VALIDATION =====

#[command]
pub async fn validate_cell_reference(cell_ref: String) -> Result<bool, String> {
    SpreadsheetEngine::validate_cell_reference(&cell_ref)
}

#[command]
pub async fn parse_cell_range(range_str: String) -> Result<CellRange, String> {
    SpreadsheetEngine::parse_cell_range(&range_str)
}

#[command]
pub async fn delete_spreadsheet_cell(cell_ref: String) -> Result<(), String> {
    SpreadsheetEngine::delete_cell(&cell_ref)
}

#[command]
pub async fn clear_spreadsheet_cell(cell_ref: String) -> Result<(), String> {
    SpreadsheetEngine::clear_cell(&cell_ref)
}

// ===== UNCERTAINTY CELL OPERATIONS =====

#[command]
pub async fn detect_uncertainty_mode(input: String) -> Result<bool, String> {
    Ok(input.contains("±") || input.contains("+/-") || input.contains("+-"))
}

#[command]
pub async fn toggle_uncertainty_cell_mode(cell_ref: String, enable: bool) -> Result<(), String> {
    SpreadsheetEngine::toggle_uncertainty_mode(&cell_ref, enable)
}

#[command]
pub async fn set_uncertainty_cell_value(
    cell_ref: String, 
    value: f64, 
    uncertainty: f64, 
    uncertainty_type: String
) -> Result<(), String> {
    let uncertainty_type = match uncertainty_type.as_str() {
        "absolute" => UncertaintyType::Absolute,
        "percentage" => UncertaintyType::Percentage,
        "standard_deviation" => UncertaintyType::StandardDeviation,
        "standard_error" => UncertaintyType::StandardError,
        _ => return Err("Invalid uncertainty type".to_string()),
    };
    
    SpreadsheetEngine::set_uncertainty_cell_value(&cell_ref, value, uncertainty, uncertainty_type)
}

#[command]
pub async fn get_uncertainty_cell_components(cell_ref: String) -> Result<Option<UncertaintyComponents>, String> {
    SpreadsheetEngine::get_uncertainty_components(&cell_ref)
}

#[command]
pub async fn convert_uncertainty_type(
    cell_ref: String, 
    target_type: String
) -> Result<UncertaintyComponents, String> {
    let target_type = match target_type.as_str() {
        "absolute" => UncertaintyType::Absolute,
        "percentage" => UncertaintyType::Percentage,
        "standard_deviation" => UncertaintyType::StandardDeviation,
        "standard_error" => UncertaintyType::StandardError,
        _ => return Err("Invalid uncertainty type".to_string()),
    };
    
    SpreadsheetEngine::convert_uncertainty_type(&cell_ref, target_type)
}

#[command]
pub async fn handle_uncertainty_cell_click(
    cell_ref: String,
    click_x: f64,
    cell_width: f64
) -> Result<UncertaintyClickResult, String> {
    SpreadsheetEngine::handle_uncertainty_cell_click(&cell_ref, click_x, cell_width)
}