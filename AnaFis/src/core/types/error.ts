// Standardized error types for Tauri commands
//
// This module provides consistent error response types that match the Rust backend.
// Error responses include error codes, messages, and optional details for better
// frontend error handling and user experience.

export const API_VERSION = '1.0.0';

export enum ErrorCode {
  // Generic errors
  InternalError = 'INTERNAL_ERROR',
  InvalidInput = 'INVALID_INPUT',
  NotFound = 'NOT_FOUND',
  PermissionDenied = 'PERMISSION_DENIED',
  Timeout = 'TIMEOUT',

  // File system errors
  FileNotFound = 'FILE_NOT_FOUND',
  FileAccessDenied = 'FILE_ACCESS_DENIED',
  FileCorrupted = 'FILE_CORRUPTED',
  PathValidationFailed = 'PATH_VALIDATION_FAILED',

  // Database errors
  DatabaseError = 'DATABASE_ERROR',
  DatabaseConnectionFailed = 'DATABASE_CONNECTION_FAILED',
  RecordNotFound = 'RECORD_NOT_FOUND',
  DuplicateRecord = 'DUPLICATE_RECORD',

  // Conversion/calculation errors
  ConversionFailed = 'CONVERSION_FAILED',
  InvalidUnit = 'INVALID_UNIT',
  IncompatibleUnits = 'INCOMPATIBLE_UNITS',
  CalculationError = 'CALCULATION_ERROR',

  // Import/Export errors
  ImportFailed = 'IMPORT_FAILED',
  ExportFailed = 'EXPORT_FAILED',
  UnsupportedFormat = 'UNSUPPORTED_FORMAT',
  ParsingError = 'PARSING_ERROR',

  // Data validation errors
  ValidationError = 'VALIDATION_ERROR',
  MissingRequiredField = 'MISSING_REQUIRED_FIELD',
  InvalidDataType = 'INVALID_DATA_TYPE',
}

export interface ErrorResponse {
  version: string;
  code: ErrorCode;
  message: string;
  details?: string;
  field?: string;
}

// Type alias for command results using standardized errors
export type CommandResult<T> = T | ErrorResponse;

// Helper functions for error handling
export function isErrorResponse<T>(result: CommandResult<T>): result is ErrorResponse {
  return typeof result === 'object' && result !== null && 'code' in result && 'message' in result;
}

export function getErrorMessage(error: ErrorResponse): string {
  if (error.field) {
    return `${error.field}: ${error.message}`;
  }
  return error.message;
}

export function getDetailedErrorMessage(error: ErrorResponse): string {
  let message = getErrorMessage(error);
  if (error.details) {
    message += `\n\nDetails: ${error.details}`;
  }
  return message;
}