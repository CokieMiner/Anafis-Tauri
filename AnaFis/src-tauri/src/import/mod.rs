//! Data Import Module
//!
//! This module handles importing data from various file formats into the application.
//!
//! ## Supported Formats
//! - **LOSSLESS**: anafispread (native format - full IWorkbookData snapshots)
//! - **TEXT INTERCHANGE**: csv, tsv, txt (for external application data)
//! - **COLUMNAR**: parquet (efficient binary columnar format)
//!
//! The module handles parsing and converting various file formats to Univer-compatible workbook data.

use crate::error::{CommandResult, file_not_found, import_error, validation_error};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// Validate and canonicalize a file path to prevent directory traversal
/// Returns the canonicalized path if valid, or an error if invalid
fn validate_and_canonicalize_path(file_path: &str) -> Result<PathBuf, String> {
    // Canonicalize the path to resolve any .. or . components
    let path = Path::new(file_path);
    let canonical_path = path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path '{}': {}", file_path, e))?;

    // Verify the canonicalized path is within allowed directories
    // For security, restrict to user's home directory and system temp directories
    let home_dir = home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    let temp_dir = env::temp_dir();

    let allowed_paths = [home_dir.as_path(), temp_dir.as_path()];

    let is_allowed = allowed_paths
        .iter()
        .any(|allowed| canonical_path.starts_with(allowed));

    if !is_allowed {
        return Err(format!(
            "Access denied: path '{}' is outside allowed directories",
            canonical_path.display()
        ));
    }

    // Verify file exists
    if !canonical_path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    Ok(canonical_path)
}

// Submodules for specific format parsers
pub mod anafispread;
pub mod csv;
pub mod parquet;

/// Import options sent from frontend (simplified structure)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOptionsFrontend {
    pub format: String,
    #[serde(default)]
    pub skip_rows: usize,
    #[serde(default)]
    pub delimiter: String,
    #[serde(default)]
    pub encoding: String,
}
/// File metadata extracted from import files
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub extension: String,
    pub row_count: Option<usize>,
    pub column_count: Option<usize>,
    pub has_formulas: Option<bool>,
    pub has_formatting: Option<bool>,
}

/// Frontend-compatible import response structure
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResponse {
    pub sheets: HashMap<String, Vec<Vec<serde_json::Value>>>,
}

/// Main import command - called from frontend
#[tauri::command]
pub async fn import_spreadsheet_file(
    file_path: String,
    options: ImportOptionsFrontend,
) -> CommandResult<ImportResponse> {
    // Validate and canonicalize path to prevent directory traversal
    let canonical_path = validate_and_canonicalize_path(&file_path)
        .map_err(|e| validation_error(e, Some("file_path".to_string())))?;

    // Route to appropriate parser based on format
    match options.format.as_str() {
        "csv" => {
            csv::import_csv(
                &canonical_path.to_string_lossy(),
                options.skip_rows,
                false, // We'll handle headers in the frontend
                Some(&options.encoding),
            )
            .await
            .map_err(|e| import_error(format!("CSV import failed: {}", e)))
        }
        "tsv" => {
            csv::import_tsv(
                &canonical_path.to_string_lossy(),
                options.skip_rows,
                false, // We'll handle headers in the frontend
                Some(&options.encoding),
            )
            .await
            .map_err(|e| import_error(format!("TSV import failed: {}", e)))
        }
        "txt" => {
            csv::import_txt(
                &canonical_path.to_string_lossy(),
                &options.delimiter,
                options.skip_rows,
                false, // We'll handle headers in the frontend
                Some(&options.encoding),
            )
            .await
            .map_err(|e| import_error(format!("TXT import failed: {}", e)))
        }
        "anafispread" => {
            // Special case: .anafispread should use direct import
            Err(import_error(
                "Use import_anafis_spread_direct for .anafispread files".to_string(),
            ))
        }
        "parquet" => parquet::import_parquet(&canonical_path.to_string_lossy())
            .await
            .map_err(|e| import_error(format!("Parquet import failed: {}", e))),
        _ => Err(validation_error(
            format!("Unsupported format: {}", options.format),
            Some("format".to_string()),
        )),
    }
}
/// Direct import command for .anafispread format
/// Returns raw IWorkbookData without conversion for lossless snapshot loading
#[tauri::command]
pub async fn import_anafis_spread_direct(file_path: String) -> CommandResult<serde_json::Value> {
    // Validate and canonicalize path to prevent directory traversal
    let canonical_path = validate_and_canonicalize_path(&file_path)
        .map_err(|e| validation_error(e, Some("file_path".to_string())))?;

    // Return raw IWorkbookData without any conversion
    // This preserves the complete Univer snapshot structure for lossless restore
    anafispread::import_anafis_spread(canonical_path.to_string_lossy().to_string())
        .await
        .map_err(|e| import_error(format!("AnaFis spread import failed: {}", e)))
}

/// Get file metadata - called before import to show file info
/// For TXT files, pass delimiter parameter to use correct column detection
#[tauri::command]
pub async fn get_file_metadata(
    file_path: String,
    delimiter: Option<String>,
) -> CommandResult<FileMetadata> {
    // Validate and canonicalize path to prevent directory traversal
    let canonical_path = validate_and_canonicalize_path(&file_path)
        .map_err(|e| validation_error(e, Some("file_path".to_string())))?;

    // Get file size
    let metadata = std::fs::metadata(&canonical_path)
        .map_err(|e| file_not_found(format!("Failed to read file metadata: {}", e)))?;
    let size = metadata.len();

    // Get extension
    let extension = Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Try to get row/column count for supported formats
    let (row_count, column_count) = match extension.as_str() {
        "csv" | "tsv" | "txt" => {
            // For text-based formats, read all lines to determine dimensions
            // For TXT, use the provided delimiter or default to "|"
            match get_text_file_dimensions(&canonical_path, &extension, delimiter.as_deref()).await
            {
                Ok((rows, cols)) => (Some(rows), Some(cols)),
                Err(_) => (None, None), // Silently fail - metadata is optional
            }
        }
        "parquet" => {
            // For parquet, we could implement dimension detection but for now return None
            (None, None)
        }
        "anafispread" => {
            // For anafispread, we could parse the JSON to get dimensions but for now return None
            (None, None)
        }
        _ => (None, None),
    };

    Ok(FileMetadata {
        path: file_path,
        size,
        extension,
        row_count,
        column_count,
        has_formulas: None,
        has_formatting: None,
    })
}

/// Helper function to get dimensions of text-based files (CSV, TSV, TXT)
/// For TXT files, delimiter parameter can be provided; if None, uses default "|"
async fn get_text_file_dimensions(
    file_path: &Path,
    extension: &str,
    txt_delimiter: Option<&str>,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let file = File::open(file_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Count ALL lines and determine max columns
    let mut row_count = 0;
    let mut max_columns = 0;

    // Determine delimiter based on file extension
    let delimiter = match extension {
        "csv" => ",",
        "tsv" => "\t",
        "txt" => {
            // For TXT files, use provided delimiter or default to "|"
            txt_delimiter.unwrap_or("|")
        }
        _ => ",",
    };

    // Read all lines to get accurate count
    while let Some(line) = lines.next_line().await? {
        if !line.trim().is_empty() {
            row_count += 1;
            let columns = line.split(delimiter).count();
            max_columns = max_columns.max(columns);
        }
    }

    Ok((row_count, max_columns))
}
