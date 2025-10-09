// Tauri commands for Data Library
use tauri::{Manager, State};
use std::sync::Mutex;

use super::database::DataLibraryDatabase;
use super::models::*;
use super::statistics::calculate_statistics;

pub struct DataLibraryState(pub Mutex<DataLibraryDatabase>);

/// Initialize the Data Library database
pub fn init_data_library(app_handle: &tauri::AppHandle) -> Result<DataLibraryState, String> {
    let app_dir = app_handle.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    
    let db_path = app_dir.join("data_library.db");
    let db = DataLibraryDatabase::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to initialize database: {}", e))?;
    
    Ok(DataLibraryState(Mutex::new(db)))
}

#[tauri::command]
pub fn save_sequence(
    request: SaveSequenceRequest,
    state: State<DataLibraryState>,
) -> Result<String, String> {
    let db = state.0.lock().unwrap();
    db.save_sequence(request)
        .map_err(|e| format!("Failed to save sequence: {}", e))
}

#[tauri::command]
pub fn get_sequences(
    search: SearchRequest,
    state: State<DataLibraryState>,
) -> Result<SequenceListResponse, String> {
    let db = state.0.lock().unwrap();
    let sequences = db.get_sequences(&search)
        .map_err(|e| format!("Failed to get sequences: {}", e))?;
    
    let total_count = sequences.len();
    let pinned_count = sequences.iter().filter(|s| s.is_pinned).count();
    
    Ok(SequenceListResponse {
        sequences,
        total_count,
        pinned_count,
    })
}

#[tauri::command]
pub fn get_sequence(
    id: String,
    state: State<DataLibraryState>,
) -> Result<Option<DataSequence>, String> {
    let db = state.0.lock().unwrap();
    db.get_sequence(&id)
        .map_err(|e| format!("Failed to get sequence: {}", e))
}

#[tauri::command]
pub fn update_sequence(
    request: UpdateSequenceRequest,
    state: State<DataLibraryState>,
) -> Result<(), String> {
    let db = state.0.lock().unwrap();
    db.update_sequence(request)
        .map_err(|e| format!("Failed to update sequence: {}", e))
}

#[tauri::command]
pub fn delete_sequence(
    id: String,
    state: State<DataLibraryState>,
) -> Result<(), String> {
    let db = state.0.lock().unwrap();
    db.delete_sequence(&id)
        .map_err(|e| format!("Failed to delete sequence: {}", e))
}

#[tauri::command]
pub fn get_sequence_stats(
    id: String,
    state: State<DataLibraryState>,
) -> Result<Option<SequenceStatistics>, String> {
    let db = state.0.lock().unwrap();
    let sequence = db.get_sequence(&id)
        .map_err(|e| format!("Failed to get sequence: {}", e))?;
    
    Ok(sequence.map(|s| calculate_statistics(&s)))
}

#[tauri::command]
pub fn pin_sequence(
    id: String,
    is_pinned: bool,
    state: State<DataLibraryState>,
) -> Result<(), String> {
    let db = state.0.lock().unwrap();
    db.update_sequence(UpdateSequenceRequest {
        id,
        name: None,
        description: None,
        tags: None,
        unit: None,
        is_pinned: Some(is_pinned),
    })
    .map_err(|e| format!("Failed to pin sequence: {}", e))
}

#[tauri::command]
pub fn duplicate_sequence(
    id: String,
    new_name: String,
    state: State<DataLibraryState>,
) -> Result<String, String> {
    let db = state.0.lock().unwrap();
    db.duplicate_sequence(&id, &new_name)
        .map_err(|e| format!("Failed to duplicate sequence: {}", e))
}

#[tauri::command]
pub fn get_all_tags(
    state: State<DataLibraryState>,
) -> Result<Vec<String>, String> {
    let db = state.0.lock().unwrap();
    db.get_all_tags()
        .map_err(|e| format!("Failed to get tags: {}", e))
}

#[tauri::command]
pub fn export_sequences_csv(
    sequence_ids: Vec<String>,
    file_path: String,
    state: State<DataLibraryState>,
) -> Result<(), String> {
    let db = state.0.lock().unwrap();
    db.export_to_csv(&sequence_ids, &file_path)
        .map_err(|e| format!("Failed to export to CSV: {}", e))
}

#[tauri::command]
pub fn export_sequences_json(
    sequence_ids: Vec<String>,
    file_path: String,
    state: State<DataLibraryState>,
) -> Result<(), String> {
    let db = state.0.lock().unwrap();
    db.export_to_json(&sequence_ids, &file_path)
        .map_err(|e| format!("Failed to export to JSON: {}", e))
}
