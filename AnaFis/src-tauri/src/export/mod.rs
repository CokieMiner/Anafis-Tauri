// Export module - handles exporting spreadsheet data to various formats
//
// Architecture:
// - PRIMARY (Lossless): anafispread (native format - full IWorkbookData snapshots)
// - INTERCHANGE (Text): csv, tsv, txt (for external application interaction)
// - COLUMNAR: parquet (efficient binary columnar format)
// - READ-ONLY: html, markdown, tex (document/report generation)
//
// Submodules:
// - text: CSV, TSV, TXT exports
// - html: HTML table exports
// - markdown: Markdown table exports
// - tex: LaTeX table exports
// - parquet: Apache Parquet exports
// - anafispread: Custom AnaFis spreadsheet format

pub mod anafispread;
pub mod html;
pub mod markdown;
pub mod parquet;
pub mod tex;
pub mod text;

use crate::error::{CommandResult, export_error, validation_error};
use serde::{Deserialize, Serialize};

/// Export format types supported by the application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Comma-separated values
    Csv,
    /// Tab-separated values
    Tsv,
    /// Custom delimiter text
    Txt,
    /// Apache Parquet
    Parquet,
    /// LaTeX table
    Tex,
    /// HTML table
    Html,
    /// Markdown table
    Markdown,
    /// `AnaFis` spreadsheet format
    #[serde(rename = "anafispread")]
    AnaFisSpread,
}

/// Export configuration passed from frontend
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportConfig {
    /// Range mode used (for logging/debugging): 'sheet', 'all', or 'custom'
    pub range: String,
    /// Export format
    pub format: ExportFormat,
    /// Format-specific options
    pub options: ExportOptions,
}

/// Options for configuring exports
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptions {
    /// Include header row
    #[serde(default)]
    pub include_headers: bool,
    /// Delimiter character (default: ',')
    pub delimiter: Option<String>,
}

/// Frontend config structure (simplified)
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExportConfigFrontend {
    pub delimiter: Option<String>,
}

/// Main export dispatcher function that routes to the appropriate format handler
#[tauri::command]
pub async fn export_data(
    data: Vec<serde_json::Value>,
    file_path: String,
    format: ExportFormat,
    config: ExportConfigFrontend,
) -> CommandResult<()> {
    let export_config = ExportConfig {
        range: "custom".to_string(),
        format,
        options: ExportOptions {
            include_headers: false, // Default value
            delimiter: config.delimiter,
        },
    };

    match export_config.format {
        ExportFormat::Csv | ExportFormat::Tsv | ExportFormat::Txt => {
            text::export_to_text(data, file_path, export_config)
                .map_err(|e| export_error(format!("Text export failed: {e}")))
        }
        ExportFormat::Parquet => parquet::export_to_parquet(data, file_path, export_config)
            .map_err(|e| export_error(format!("Parquet export failed: {e}"))),
        ExportFormat::Html => html::export_to_html(data, file_path, export_config)
            .map_err(|e| export_error(format!("HTML export failed: {e}"))),
        ExportFormat::Markdown => markdown::export_to_markdown(data, file_path, export_config)
            .map_err(|e| export_error(format!("Markdown export failed: {e}"))),
        ExportFormat::Tex => tex::export_to_latex(data, file_path, export_config)
            .map_err(|e| export_error(format!("LaTeX export failed: {e}"))),
        ExportFormat::AnaFisSpread => {
            // For anafispread, we need to pass the data directly (not as array)
            let workbook_data = if data.len() == 1 {
                data[0].clone()
            } else {
                return Err(validation_error(
                    "Invalid data format for AnaFis Spreadsheet export".to_string(),
                    Some("data".to_string()),
                ));
            };
            anafispread::export_anafispread(workbook_data, file_path)
                .map_err(|e| export_error(format!("AnaFis spread export failed: {e}")))
        }
    }
}
