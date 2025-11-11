/**
 * Constants used throughout the univer spreadsheet implementation
 */

// Default sheet dimensions
export const DEFAULT_SHEET_ROWS = 1000;
export const DEFAULT_SHEET_COLS = 52;

// Default workbook dimensions (fallback values for data conversion)
export const DEFAULT_WORKBOOK_ROWS = 1000;
export const DEFAULT_WORKBOOK_COLS = 52;

// Performance tuning constants
export const EXPONENTIAL_BACKOFF_BASE_MS = 100;

// Error messages
export const ERROR_MESSAGES = {
  // Cell reference errors
  INVALID_CELL_REFERENCE: 'Invalid cell reference format',
  MALFORMED_CELL_REFERENCE: 'Malformed cell reference',
  INVALID_RANGE_REFERENCE: 'Invalid range reference',
  MALFORMED_RANGE_REFERENCE: 'Malformed range reference',

  // Workbook/sheet errors
  NO_ACTIVE_WORKBOOK: 'No active workbook available',
  NO_ACTIVE_WORKBOOK_PROTECTION: 'No active workbook available for protection application',
  SHEET_NOT_FOUND: 'Sheet not found',
  NO_SHEETS_FOUND: 'No sheets found in snapshot',

  // Import/Export errors
  NO_FILE_PATH: 'No file path',
  SPREADSHEET_NOT_READY: 'Spreadsheet not ready',
  WORKBOOK_SNAPSHOT_NOT_SUPPORTED: 'Workbook snapshot loading not supported',
  INVALID_SNAPSHOT_DATA: 'Invalid snapshot data',
  NO_DATA_TO_EXPORT: 'No data to export',
  WORKBOOK_SNAPSHOT_EXPORT_NOT_SUPPORTED: 'Workbook snapshot not supported',
  CUSTOM_RANGE_REQUIRED: 'Please specify a custom range',

  // Protection errors
  PROTECTION_APPLICATION_CANCELLED: 'Protection application cancelled',

  // Range validation errors
  INVALID_RANGE_OBJECT: 'Invalid range object: missing required properties',

  // Data conversion errors
  SHEET_ID_NOT_FOUND: (sheetId: string) => `Sheet with ID ${sheetId} not found`,

  // Formula errors
  LAMBERT_W_UNDEFINED: 'Lambert W function is undefined for x < -1/e',
  HERMITE_NEGATIVE_DEGREE: 'Hermite polynomial degree must be a non-negative integer',
  MODULUS_OUT_OF_RANGE: 'Modulus k must be in (-1, 1)',
  ZETA_FUNCTION_POLE: 'Zeta function has a pole at s = 1',
  ZETA_FUNCTION_LIMITATION: 'Zeta function implementation limited to s > 1',

  // General operation errors
  OPERATION_FAILED: (operation: string) => `Failed to ${operation}`,
  UNKNOWN_ERROR: 'Unknown error occurred',

  // Validation errors
  INPUT_MUST_BE_NON_EMPTY_STRING: 'Input must be a non-empty string',
  INVALID_COLUMN_LETTER_FORMAT: (letter: string) => `Invalid column letter format: ${letter}. Must contain only uppercase letters A-Z.`,
} as const;

// HTTP status codes for error classification
export const HTTP_STATUS = {
  // Client errors (400-499)
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
  FORBIDDEN: 403,
  NOT_FOUND: 404,
  REQUEST_TIMEOUT: 408,
  TOO_MANY_REQUESTS: 429,

  // Server errors (500-599)
  INTERNAL_SERVER_ERROR: 500,
  NOT_IMPLEMENTED: 501,
  BAD_GATEWAY: 502,
  SERVICE_UNAVAILABLE: 503,
  GATEWAY_TIMEOUT: 504,
  HTTP_VERSION_NOT_SUPPORTED: 505,
  VARIANT_ALSO_NEGOTIATES: 506,
  INSUFFICIENT_STORAGE: 507,
  LOOP_DETECTED: 508,
  NOT_EXTENDED: 510,
  NETWORK_AUTHENTICATION_REQUIRED: 511,
} as const;

// Error codes for spreadsheet operations
export const ERROR_CODES = {
  VALIDATION_ERROR: 'VALIDATION_ERROR',
  INVALID_RANGE: 'INVALID_RANGE',
  INVALID_CELL_REF: 'INVALID_CELL_REF',
  PERMISSION_DENIED: 'PERMISSION_DENIED',
  UNAUTHORIZED: 'UNAUTHORIZED',
  FORBIDDEN: 'FORBIDDEN',
  BAD_REQUEST: 'BAD_REQUEST',
  MALFORMED_INPUT: 'MALFORMED_INPUT',
  OPERATION_FAILED: 'OPERATION_FAILED',
  SPREADSHEET_UNKNOWN_ERROR: 'SPREADSHEET_UNKNOWN_ERROR',
} as const;

// Potentially retryable HTTP client errors
export const POTENTIALLY_RETRYABLE_CLIENT_ERRORS = new Set([
  HTTP_STATUS.REQUEST_TIMEOUT,
  HTTP_STATUS.TOO_MANY_REQUESTS,
]);

// Non-retryable server errors (permanent server-side problems)
export const NON_RETRYABLE_SERVER_ERRORS = new Set([
  HTTP_STATUS.NOT_IMPLEMENTED,
  HTTP_STATUS.HTTP_VERSION_NOT_SUPPORTED,
  HTTP_STATUS.VARIANT_ALSO_NEGOTIATES,
  HTTP_STATUS.INSUFFICIENT_STORAGE,
  HTTP_STATUS.LOOP_DETECTED,
  HTTP_STATUS.NOT_EXTENDED,
  HTTP_STATUS.NETWORK_AUTHENTICATION_REQUIRED,
]);

// Non-retryable error codes
export const NON_RETRYABLE_ERROR_CODES = new Set([
  ERROR_CODES.VALIDATION_ERROR,
  ERROR_CODES.INVALID_RANGE,
  ERROR_CODES.INVALID_CELL_REF,
  ERROR_CODES.PERMISSION_DENIED,
  ERROR_CODES.UNAUTHORIZED,
  ERROR_CODES.FORBIDDEN,
  ERROR_CODES.BAD_REQUEST,
  ERROR_CODES.MALFORMED_INPUT,
]);

// Non-retryable error types
export const NON_RETRYABLE_ERROR_TYPES = new Set([
  'validation',
  'permission',
  'authorization',
  'authentication',
  'bad_request',
  'malformed',
]);

// Validation patterns for error message classification
export const VALIDATION_ERROR_PATTERNS = [
  'invalid cell reference',
  'invalid range reference',
  'malformed cell reference',
  'malformed range reference',
] as const;

export const PERMISSION_ERROR_PATTERNS = [
  'access denied',
  'insufficient permissions',
  'not authorized',
] as const;