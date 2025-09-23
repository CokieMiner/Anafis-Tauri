use super::cell::UnifiedCell;
use super::state::SpreadsheetState;
use tauri::{State, AppHandle};
use std::fs;
use std::path::PathBuf;

fn get_save_path(app: &AppHandle) -> PathBuf {
    let path = app.path().app_data_dir().unwrap();
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.join("spreadsheet.json")
}

#[tauri::command]
pub fn load_spreadsheet(app: AppHandle, state: State<SpreadsheetState>) -> Result<Vec<Vec<UnifiedCell>>, String> {
    let path = get_save_path(&app);
    if path.exists() {
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let spreadsheet_data: Vec<Vec<UnifiedCell>> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        *state.data.lock().unwrap() = spreadsheet_data.clone();
        Ok(spreadsheet_data)
    } else {
        // Return the default empty state
        Ok(state.data.lock().unwrap().clone())
    }
}

#[tauri::command]
pub fn save_spreadsheet(state: State<SpreadsheetState>, app: AppHandle) -> Result<(), String> {
    let path = get_save_path(&app);
    let data = state.data.lock().unwrap();
    let json = serde_json::to_string(&*data).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_spreadsheet_data(state: State<SpreadsheetState>) -> Vec<Vec<UnifiedCell>> {
    state.data.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_cell(
    row: usize,
    col: usize,
    cell: UnifiedCell,
    state: State<SpreadsheetState>,
) -> Result<(), String> {
    let mut data = state.data.lock().unwrap();
    if row < data.len() && col < data[row].len() {
        data[row][col] = cell;
        Ok(())
    } else {
        Err("Cell coordinates out of bounds".to_string())
    }
}
