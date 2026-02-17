// AnaFis Spreadsheet (.anafispread) import
//
// Import handler for AnaFis native format - decompresses and reads full IWorkbookData snapshot.
// This format preserves EVERYTHING for lossless restore.
//
// File Structure:
// - Bytes 0-7:   Magic number "ANAFIS\x01\x00" (identifies file type)
// - Bytes 8-11:  Format version (u32, little-endian)
// - Bytes 12+:   Gzip-compressed JSON data

use flate2::read::GzDecoder;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};

// Maximum file size: 100MB (reasonable limit for spreadsheet files)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Magic number for .anafispread files: "ANAFIS" + version marker
const MAGIC_NUMBER: &[u8; 8] = b"ANAFIS\x01\x00";
const SUPPORTED_VERSION: u32 = 1;

/// Import data from AnaFis Spreadsheet (.anafispread) format
///
/// Reads the compressed IWorkbookData snapshot and returns it for direct loading into Univer.
#[tauri::command]
pub async fn import_anafis_spread(file_path: String) -> Result<Value, String> {
    // Open the file
    let mut file = File::open(&file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    // Check file size for security
    let metadata = file
        .metadata()
        .map_err(|e| format!("Failed to read file metadata: {}", e))?;

    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!(
            "File too large: {} MB (maximum: {} MB)",
            metadata.len() / (1024 * 1024),
            MAX_FILE_SIZE / (1024 * 1024)
        ));
    }

    // Read and verify magic number
    let mut magic_buf = [0u8; 8];
    file.read_exact(&mut magic_buf)
        .map_err(|e| format!("Failed to read file header: {}", e))?;

    if &magic_buf != MAGIC_NUMBER {
        return Err(
            "Invalid file format: Not an AnaFis Spreadsheet file (magic number mismatch)"
                .to_string(),
        );
    }

    // Read and verify format version
    let mut version_buf = [0u8; 4];
    file.read_exact(&mut version_buf)
        .map_err(|e| format!("Failed to read version: {}", e))?;

    let version = u32::from_le_bytes(version_buf);
    if version != SUPPORTED_VERSION {
        return Err(format!(
            "Unsupported file version: {} (expected: {})",
            version, SUPPORTED_VERSION
        ));
    }

    // Decompress the remaining data (all .anafispread files are gzip compressed)
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);

    // Parse JSON
    let data: Value = serde_json::from_reader(reader)
        .map_err(|e| format!("Failed to parse AnaFis Spreadsheet file: {}", e))?;

    // Validate format in JSON metadata
    if let Some(format) = data.get("format").and_then(|f| f.as_str()) {
        if format != "anafis_spreadsheet" {
            return Err(format!("Invalid AnaFis Spreadsheet format: {}", format));
        }
    } else {
        return Err("Missing format identifier in AnaFis Spreadsheet file".to_string());
    }

    // Extract the workbook data
    let workbook = data
        .get("workbook")
        .ok_or("Missing workbook data in AnaFis Spreadsheet file")?;

    // Return the full IWorkbookData snapshot
    Ok(workbook.clone())
}
