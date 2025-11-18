// Tauri commands for Data Library
use tauri::{Manager, State};
use std::sync::Mutex;

use super::database::DataLibraryDatabase;
use super::models::*;
use super::statistics::calculate_statistics;
use crate::error::{CommandResult, database_error, internal_error, export_error};

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
) -> CommandResult<String> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.save_sequence(request)
        .map_err(|e| database_error(format!("Failed to save sequence: {}", e)))
}

#[tauri::command]
pub fn get_sequences(
    search: SearchRequest,
    state: State<DataLibraryState>,
) -> CommandResult<SequenceListResponse> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.get_sequences_paginated(&search)
        .map_err(|e| database_error(format!("Failed to get sequences: {}", e)))
}

#[tauri::command]
pub fn get_sequence(
    id: String,
    state: State<DataLibraryState>,
) -> CommandResult<Option<DataSequence>> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.get_sequence(&id)
        .map_err(|e| database_error(format!("Failed to get sequence: {}", e)))
}

#[tauri::command]
pub fn update_sequence(
    request: UpdateSequenceRequest,
    state: State<DataLibraryState>,
) -> CommandResult<()> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.update_sequence(request)
        .map_err(|e| database_error(format!("Failed to update sequence: {}", e)))
}

#[tauri::command]
pub fn delete_sequence(
    id: String,
    state: State<DataLibraryState>,
) -> CommandResult<()> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.delete_sequence(&id)
        .map_err(|e| database_error(format!("Failed to delete sequence: {}", e)))
}

#[tauri::command]
pub fn get_sequence_stats(
    id: String,
    state: State<DataLibraryState>,
) -> CommandResult<Option<SequenceStatistics>> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    let sequence = db.get_sequence(&id)
        .map_err(|e| database_error(format!("Failed to get sequence: {}", e)))?;
    
    Ok(sequence.map(|s| calculate_statistics(&s)))
}

#[tauri::command]
pub fn pin_sequence(
    id: String,
    is_pinned: bool,
    state: State<DataLibraryState>,
) -> CommandResult<()> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.update_sequence(UpdateSequenceRequest {
        id,
        name: None,
        description: None,
        tags: None,
        unit: None,
        is_pinned: Some(is_pinned),
    })
    .map_err(|e| database_error(format!("Failed to pin sequence: {}", e)))
}

#[tauri::command]
pub fn duplicate_sequence(
    id: String,
    new_name: String,
    state: State<DataLibraryState>,
) -> CommandResult<String> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.duplicate_sequence(&id, &new_name)
        .map_err(|e| database_error(format!("Failed to duplicate sequence: {}", e)))
}

#[tauri::command]
pub fn get_all_tags(
    state: State<DataLibraryState>,
) -> CommandResult<Vec<String>> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.get_all_tags()
        .map_err(|e| database_error(format!("Failed to get tags: {}", e)))
}

#[tauri::command]
pub fn export_sequences_csv(
    sequence_ids: Vec<String>,
    file_path: String,
    state: State<DataLibraryState>,
) -> CommandResult<()> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.export_to_csv(&sequence_ids, &file_path)
        .map_err(|e| export_error(format!("Failed to export to CSV: {}", e)))
}

#[tauri::command]
pub fn batch_import_sequences(
    request: BatchImportRequest,
    state: State<DataLibraryState>,
) -> CommandResult<BatchImportResponse> {
    let db = state.0.lock().map_err(|e| internal_error(format!("Failed to lock database: {}", e)))?;
    db.batch_import_sequences(request)
        .map_err(|e| database_error(format!("Batch import failed: {}", e)))
}
