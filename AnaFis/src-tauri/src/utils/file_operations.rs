// File operations utilities

use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::path::Path;

/// Save a PNG file from base64-encoded data
#[tauri::command]
pub fn save_png_file(path: String, data: String) -> Result<(), String> {
    // Decode base64 data
    let bytes = general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| format!("Failed to decode base64 data: {}", e))?;
    
    // Ensure parent directory exists
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }
    
    // Write file
    fs::write(&path, bytes)
        .map_err(|e| format!("Failed to write PNG file: {}", e))?;
    
    Ok(())
}

/// Save an image file from a data URL (format: "data:image/png;base64,...")
#[tauri::command]
pub fn save_image_from_data_url(data_url: String, path: String) -> Result<(), String> {
    // Split the data URL to extract base64 data
    let parts: Vec<&str> = data_url.split(',').collect();
    if parts.len() != 2 {
        return Err("Invalid data URL format. Expected 'data:image/[type];base64,[data]'".to_string());
    }
    
    let base64_data = parts[1];
    
    // Decode base64 data
    let bytes = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Failed to decode base64 data: {}", e))?;
    
    // Ensure parent directory exists
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }
    
    // Write file
    fs::write(&path, bytes)
        .map_err(|e| format!("Failed to write image file: {}", e))?;
    
    Ok(())
}

/// Save an SVG file from SVG content string
#[tauri::command]
pub fn save_svg_file(svg_content: String, path: String) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }
    
    // Write SVG content directly to file
    fs::write(&path, svg_content)
        .map_err(|e| format!("Failed to write SVG file: {}", e))?;
    
    Ok(())
}
