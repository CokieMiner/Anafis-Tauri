// Standardized error handling for Tauri commands
//
// This module provides consistent error responses across all backend commands.
// Error responses include error codes, messages, and optional details for better
// frontend error handling and user experience.

use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// API version information
pub const API_VERSION: &str = "1.0.0";

/// Standardized error codes for different types of failures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Generic errors
    InternalError,
    InvalidInput,
    NotFound,
    PermissionDenied,
    Timeout,

    // File system errors
    FileNotFound,
    FileAccessDenied,
    FileCorrupted,
    PathValidationFailed,

    // Database errors
    DatabaseError,
    DatabaseConnectionFailed,
    RecordNotFound,
    DuplicateRecord,

    // Conversion/calculation errors
    ConversionFailed,
    InvalidUnit,
    IncompatibleUnits,
    CalculationError,

    // Import/Export errors
    ImportFailed,
    ExportFailed,
    UnsupportedFormat,
    ParsingError,

    // Data validation errors
    ValidationError,
    MissingRequiredField,
    InvalidDataType,

    // Window errors
    WindowError,
}

/// Standardized error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// API version
    pub version: String,
    /// Error code for programmatic handling
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Optional detailed information (for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Optional field name if error is related to a specific field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}: {}",
            to_string(&self.code).unwrap_or_else(|_| "UNKNOWN_ERROR".to_owned()),
            self.message
        )
    }
}

/// Type alias for command results using standardized errors
pub type CommandResult<T> = Result<T, ErrorResponse>;

pub fn file_not_found(path: impl Into<String>) -> ErrorResponse {
    ErrorResponse {
        version: API_VERSION.to_owned(),
        code: ErrorCode::FileNotFound,
        message: format!("File not found: {}", path.into()),
        details: None,
        field: None,
    }
}

pub fn validation_error(message: impl Into<String>, field: Option<String>) -> ErrorResponse {
    ErrorResponse {
        version: API_VERSION.to_owned(),
        code: ErrorCode::ValidationError,
        message: message.into(),
        details: None,
        field,
    }
}

pub fn internal_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse {
        version: API_VERSION.to_owned(),
        code: ErrorCode::InternalError,
        message: message.into(),
        details: None,
        field: None,
    }
}

pub fn database_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse {
        version: API_VERSION.to_owned(),
        code: ErrorCode::DatabaseError,
        message: message.into(),
        details: None,
        field: None,
    }
}

pub fn conversion_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse {
        version: API_VERSION.to_owned(),
        code: ErrorCode::ConversionFailed,
        message: message.into(),
        details: None,
        field: None,
    }
}

pub fn import_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse {
        version: API_VERSION.to_owned(),
        code: ErrorCode::ImportFailed,
        message: message.into(),
        details: None,
        field: None,
    }
}

pub fn export_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse {
        version: API_VERSION.to_owned(),
        code: ErrorCode::ExportFailed,
        message: message.into(),
        details: None,
        field: None,
    }
}

pub fn window_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse {
        version: API_VERSION.to_owned(),
        code: ErrorCode::WindowError,
        message: message.into(),
        details: None,
        field: None,
    }
}
