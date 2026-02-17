// AnaFis Spreadsheet (.anafispread) export
//
// Custom format for AnaFis spreadsheet data - Native Univer IWorkbookData snapshot.
// This format preserves EVERYTHING: cell values, formulas, formatting, metadata,
// protection rules, resources, merged cells, and all Univer-specific data.
//
// This is the LOSSLESS native format for complete save/restore cycles.
// Format: Magic bytes + Version + Compressed (gzip) JSON of full IWorkbookData
//
// File Structure:
// - Bytes 0-7:   Magic number "ANAFIS\x01\x00" (identifies file type)
// - Bytes 8-11:  Format version (u32, little-endian, currently 1)
// - Bytes 12+:   Gzip-compressed JSON data

use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Magic number for .anafispread files: "ANAFIS" + version marker
const MAGIC_NUMBER: &[u8; 8] = b"ANAFIS\x01\x00";
const FORMAT_VERSION: u32 = 1;

/// Export data to AnaFis Spreadsheet (.anafispread) format
///
/// This format accepts the full IWorkbookData JSON snapshot from Univer's workbook.save()
/// and stores it with compression for complete lossless save/restore.
#[tauri::command]
pub async fn export_anafispread(data: Value, file_path: String) -> Result<(), String> {
    // For .anafispread, we expect the IWorkbookData snapshot directly
    let workbook_data = &data;

    // Create comprehensive export structure with metadata
    let export_data = serde_json::json!({
        "version": "1.0",
        "format": "anafis_spreadsheet",
        "compressed": true,
        "metadata": {
            "created": chrono::Utc::now().to_rfc3339(),
            "creator": "AnaFis",
            "description": "Complete Univer workbook snapshot with full fidelity"
        },
        "workbook": workbook_data
    });

    // Write to file with gzip compression (always compressed for .anafispread)
    let mut file = File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;

    // Write magic number to identify file type
    file.write_all(MAGIC_NUMBER)
        .map_err(|e| format!("Failed to write magic number: {}", e))?;

    // Write format version (u32, little-endian)
    file.write_all(&FORMAT_VERSION.to_le_bytes())
        .map_err(|e| format!("Failed to write version: {}", e))?;

    // Now write the compressed JSON data
    let encoder = GzEncoder::new(file, Compression::default());
    let writer = BufWriter::new(encoder);
    serde_json::to_writer(writer, &export_data)
        .map_err(|e| format!("Failed to write AnaFis Spreadsheet file: {}", e))?;

    Ok(())
}
