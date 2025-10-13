// Export module - handles exporting spreadsheet data to various formats
//
// Submodules:
// - text: CSV, TSV, TXT exports
// - json: JSON exports
// - excel: Excel (.xlsx) exports (future)
// - anafispread: Custom AnaFis spreadsheet format (future)

pub mod text;
pub mod json;

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
    /// JSON format
    Json,
    /// Excel 2007+ format (future)
    Xlsx,
    /// AnaFis spreadsheet format (future)
    #[serde(rename = "anafispread")]
    AnaFisSpread,
}

/// Export configuration passed from frontend
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportConfig {
    /// Range to export: 'selection', 'sheet', 'all', or specific range
    pub range: String,
    /// Export format
    pub format: ExportFormat,
    /// Format-specific options
    pub options: ExportOptions,
}

/// Options for configuring exports
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptions {
    // General options
    /// Include header row
    #[serde(default = "default_true")]
    pub include_headers: bool,
    /// Include formulas (if false, evaluate them)
    #[serde(default)]
    pub include_formulas: bool,
    /// Include formatting metadata
    #[serde(default)]
    pub include_formatting: bool,
    /// Include metadata
    #[serde(default)]
    pub include_metadata: bool,
    /// Include uncertainties
    #[serde(default)]
    pub include_uncertainties: bool,
    
    // Text format options (CSV, TSV, TXT)
    /// Delimiter character (default: ',' for CSV, '\t' for TSV)
    pub delimiter: Option<String>,
    /// Character encoding (default: utf8)
    #[serde(default = "default_utf8")]
    pub encoding: String,
    /// Line ending style (default: crlf)
    #[serde(default = "default_crlf")]
    pub line_ending: String,
    /// Quote character (default: '"')
    #[serde(default = "default_quote")]
    pub quote_char: String,
    
    // JSON options
    /// JSON format: 'array', 'object', or 'records'
    #[serde(default = "default_json_format")]
    pub json_format: String,
    /// Pretty print JSON (default: true)
    #[serde(default = "default_true")]
    pub pretty_print: bool,
    
    // Compression options
    /// Compress output file (gzip)
    #[serde(default)]
    pub compress: bool,
}

// Default value functions for serde
fn default_true() -> bool {
    true
}

fn default_utf8() -> String {
    "utf8".to_string()
}

fn default_crlf() -> String {
    "crlf".to_string()
}

fn default_quote() -> String {
    "\"".to_string()
}

fn default_json_format() -> String {
    "records".to_string()
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_headers: true,
            include_formulas: false,
            include_formatting: false,
            include_metadata: false,
            include_uncertainties: false,
            delimiter: None,
            encoding: default_utf8(),
            line_ending: default_crlf(),
            quote_char: default_quote(),
            json_format: default_json_format(),
            pretty_print: true,
            compress: false,
        }
    }
}
