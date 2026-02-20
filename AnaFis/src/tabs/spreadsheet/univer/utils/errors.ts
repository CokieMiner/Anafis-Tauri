// errors.ts - Comprehensive error classes and handling utilities for spreadsheet operations
import {
  EXPONENTIAL_BACKOFF_BASE_MS,
  HTTP_STATUS,
  NON_RETRYABLE_ERROR_CODES,
  NON_RETRYABLE_ERROR_TYPES,
  NON_RETRYABLE_SERVER_ERRORS,
  PERMISSION_ERROR_PATTERNS,
  POTENTIALLY_RETRYABLE_CLIENT_ERRORS,
  VALIDATION_ERROR_PATTERNS,
} from './constants';

// ============================================================================
// ERROR CODES AND TYPES
// ============================================================================

/**
 * Standardized error codes for spreadsheet operations
 */
export enum SpreadsheetErrorCode {
  // Validation errors
  INVALID_RANGE = 'INVALID_RANGE',

  // Permission errors
  PERMISSION_DENIED = 'PERMISSION_DENIED',

  // Operation errors
  OPERATION_FAILED = 'OPERATION_FAILED',
  NETWORK_ERROR = 'NETWORK_ERROR',
  TIMEOUT = 'TIMEOUT',
  RESOURCE_EXHAUSTED = 'RESOURCE_EXHAUSTED',

  // Data errors
  FORMULA_ERROR = 'FORMULA_ERROR',

  // System errors
  SPREADSHEET_NOT_READY = 'SPREADSHEET_NOT_READY',
  INVALID_OPERATION = 'INVALID_OPERATION',
  UNKNOWN_ERROR = 'UNKNOWN_ERROR',
}

/**
 * Error severity levels
 */
export enum ErrorSeverity {
  LOW = 'low', // Minor issues, operation can continue
  MEDIUM = 'medium', // Significant issues, user attention needed
  HIGH = 'high', // Critical issues, operation failed
  FATAL = 'fatal', // System-level failures
}

/**
 * Error categories for better error handling
 */
export enum ErrorCategory {
  VALIDATION = 'validation',
  PERMISSION = 'permission',
  NETWORK = 'network',
  DATA = 'data',
  SYSTEM = 'system',
  USER = 'user',
}

// ============================================================================
// ERROR CLASSES
// ============================================================================

/**
 * Base spreadsheet error class with enhanced context
 */
export class SpreadsheetError extends Error {
  public readonly code: SpreadsheetErrorCode;
  public readonly category: ErrorCategory;
  public readonly severity: ErrorSeverity;
  public readonly operation: string | undefined;
  public readonly context: Record<string, unknown> | undefined;
  public readonly originalError: Error | undefined;
  public readonly timestamp: Date;
  public readonly isRetryable: boolean;

  constructor(
    message: string,
    code: SpreadsheetErrorCode,
    category: ErrorCategory,
    severity: ErrorSeverity,
    options: {
      operation?: string | undefined;
      context?: Record<string, unknown> | undefined;
      originalError?: Error | undefined;
      isRetryable?: boolean | undefined;
    } = {}
  ) {
    super(message);
    this.name = 'SpreadsheetError';
    this.code = code;
    this.category = category;
    this.severity = severity;
    this.operation = options.operation;
    this.context = options.context;
    this.originalError = options.originalError;
    this.timestamp = new Date();
    this.isRetryable = options.isRetryable ?? this.determineRetryability();
  }

  /**
   * Determine if this error is retryable based on its properties
   */
  private determineRetryability(): boolean {
    // Validation and permission errors are never retryable
    if (
      this.category === ErrorCategory.VALIDATION ||
      this.category === ErrorCategory.PERMISSION
    ) {
      return false;
    }

    // Network and system errors might be retryable
    if (
      this.category === ErrorCategory.NETWORK ||
      this.category === ErrorCategory.SYSTEM
    ) {
      return this.code !== SpreadsheetErrorCode.INVALID_OPERATION;
    }

    // Data errors depend on the specific code
    if (this.category === ErrorCategory.DATA) {
      return (
        this.code === SpreadsheetErrorCode.TIMEOUT ||
        this.code === SpreadsheetErrorCode.RESOURCE_EXHAUSTED
      );
    }

    return false;
  }

  /**
   * Get user-friendly error message
   */
  getUserMessage(): string {
    return formatErrorForUser(this);
  }

  /**
   * Get detailed error information for logging/debugging
   */
  getDetailedInfo(): Record<string, unknown> {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      category: this.category,
      severity: this.severity,
      operation: this.operation,
      context: this.context,
      timestamp: this.timestamp.toISOString(),
      isRetryable: this.isRetryable,
      stack: this.stack,
      originalError: this.originalError
        ? {
            name: this.originalError.name,
            message: this.originalError.message,
            stack: this.originalError.stack,
          }
        : undefined,
    };
  }

  /**
   * Create a new error with additional context
   */
  withContext(additionalContext: Record<string, unknown>): SpreadsheetError {
    return new SpreadsheetError(
      this.message,
      this.code,
      this.category,
      this.severity,
      {
        operation: this.operation,
        context: { ...this.context, ...additionalContext },
        originalError: this.originalError,
        isRetryable: this.isRetryable,
      }
    );
  }

  /**
   * Create a new error with different severity
   */
  withSeverity(severity: ErrorSeverity): SpreadsheetError {
    return new SpreadsheetError(
      this.message,
      this.code,
      this.category,
      severity,
      {
        operation: this.operation,
        context: this.context,
        originalError: this.originalError,
        isRetryable: this.isRetryable,
      }
    );
  }
}

/**
 * Validation-specific error
 */
export class SpreadsheetValidationError extends SpreadsheetError {
  public readonly field: string;

  constructor(
    message: string,
    field: string,
    operation?: string,
    context?: Record<string, unknown>
  ) {
    super(
      message,
      SpreadsheetErrorCode.INVALID_RANGE, // Default, can be overridden
      ErrorCategory.VALIDATION,
      ErrorSeverity.HIGH,
      { operation, context }
    );
    this.name = 'SpreadsheetValidationError';
    this.field = field;
  }
}

/**
 * Operation-specific error
 */
export class SpreadsheetOperationError extends SpreadsheetError {
  constructor(
    operation: string,
    originalError?: Error,
    context?: Record<string, unknown>
  ) {
    super(
      `Failed to ${operation}`,
      SpreadsheetErrorCode.OPERATION_FAILED,
      ErrorCategory.SYSTEM,
      ErrorSeverity.HIGH,
      { operation, originalError, context }
    );
    this.name = 'SpreadsheetOperationError';
  }
}

/**
 * Error thrown when a permission is denied
 */
class SpreadsheetPermissionError extends SpreadsheetError {
  constructor(
    message: string,
    operation?: string,
    context?: Record<string, unknown>
  ) {
    super(
      message,
      SpreadsheetErrorCode.PERMISSION_DENIED,
      ErrorCategory.PERMISSION,
      ErrorSeverity.HIGH,
      { operation, context }
    );
    this.name = 'SpreadsheetPermissionError';
  }
}

/**
 * Error thrown for network or connectivity issues
 */
class SpreadsheetNetworkError extends SpreadsheetError {
  constructor(
    message: string,
    operation?: string,
    context?: Record<string, unknown>
  ) {
    super(
      message,
      SpreadsheetErrorCode.NETWORK_ERROR,
      ErrorCategory.NETWORK,
      ErrorSeverity.MEDIUM,
      { operation, context }
    );
    this.name = 'SpreadsheetNetworkError';
  }
}

// ============================================================================
// ERROR HANDLING UTILITIES
// ============================================================================

/**
 * Normalize any error into a SpreadsheetError
 */
export function normalizeError(
  error: unknown,
  operation?: string
): SpreadsheetError {
  // Already a SpreadsheetError
  if (error instanceof SpreadsheetError) {
    return operation && !error.operation
      ? error.withContext({ operation })
      : error;
  }

  // Standard Error
  if (error instanceof Error) {
    return classifyError(error, operation);
  }

  // Object with message
  if (error && typeof error === 'object' && 'message' in error) {
    const err = error as { message: unknown; code?: unknown; type?: unknown };
    const message = String(err.message);
    const code =
      err.code !== null && typeof err.code === 'string' ? err.code : undefined;
    const type =
      err.type !== null && typeof err.type === 'string' ? err.type : undefined;

    return new SpreadsheetError(
      message,
      mapErrorCode(code),
      classifyErrorCategory(message, type),
      ErrorSeverity.MEDIUM,
      { operation, context: { originalCode: code, originalType: type } }
    );
  }

  // Primitive values
  const message =
    error instanceof Error
      ? error.message
      : typeof error === 'string'
        ? error
        : 'Unknown error';
  return new SpreadsheetError(
    message,
    SpreadsheetErrorCode.UNKNOWN_ERROR,
    ErrorCategory.SYSTEM,
    ErrorSeverity.MEDIUM,
    { operation }
  );
}

/**
 * Classify an Error into appropriate SpreadsheetError
 */
function classifyError(error: Error, operation?: string): SpreadsheetError {
  const message = error.message.toLowerCase();

  // Validation errors
  if (message.includes('invalid range') || message.includes('invalid cell')) {
    return new SpreadsheetValidationError(error.message, 'range', operation);
  }

  // Permission errors
  if (
    message.includes('permission') ||
    message.includes('denied') ||
    message.includes('protected')
  ) {
    return new SpreadsheetPermissionError(error.message, operation);
  }

  // Network errors
  if (
    message.includes('network') ||
    message.includes('timeout') ||
    message.includes('connection')
  ) {
    return new SpreadsheetNetworkError(error.message, operation);
  }

  // Default to operation error
  return new SpreadsheetOperationError(operation ?? 'unknown operation', error);
}

/**
 * Map string error codes to SpreadsheetErrorCode enum
 */
function mapErrorCode(code?: string): SpreadsheetErrorCode {
  if (!code) {
    return SpreadsheetErrorCode.UNKNOWN_ERROR;
  }

  const codeMap: Record<string, SpreadsheetErrorCode> = {
    INVALID_RANGE: SpreadsheetErrorCode.INVALID_RANGE,
    PERMISSION_DENIED: SpreadsheetErrorCode.PERMISSION_DENIED,
    TIMEOUT: SpreadsheetErrorCode.TIMEOUT,
    NETWORK_ERROR: SpreadsheetErrorCode.NETWORK_ERROR,
    // Add more mappings as needed
  };

  return codeMap[code] ?? SpreadsheetErrorCode.UNKNOWN_ERROR;
}

/**
 * Classify error category based on message and type
 */
function classifyErrorCategory(message: string, type?: string): ErrorCategory {
  const msg = message.toLowerCase();

  if (
    type === 'validation' ||
    msg.includes('invalid') ||
    msg.includes('format')
  ) {
    return ErrorCategory.VALIDATION;
  }

  if (
    type === 'permission' ||
    msg.includes('permission') ||
    msg.includes('denied')
  ) {
    return ErrorCategory.PERMISSION;
  }

  if (
    type === 'network' ||
    msg.includes('network') ||
    msg.includes('timeout')
  ) {
    return ErrorCategory.NETWORK;
  }

  if (msg.includes('data') || msg.includes('formula')) {
    return ErrorCategory.DATA;
  }

  return ErrorCategory.SYSTEM;
}

/**
 * Log error with appropriate level based on severity
 */
export function logError(error: SpreadsheetError): void {
  const info = error.getDetailedInfo();

  switch (error.severity) {
    case ErrorSeverity.FATAL:
      console.error('[SPREADSHEET FATAL]', info);
      break;
    case ErrorSeverity.HIGH:
      console.error('[SPREADSHEET ERROR]', info);
      break;
    case ErrorSeverity.MEDIUM:
      console.warn('[SPREADSHEET WARNING]', info);
      break;
    case ErrorSeverity.LOW:
      console.info('[SPREADSHEET INFO]', info);
      break;
  }
}

/**
 * Helper to display error to user or log it nicely
 */
function formatErrorForUser(error: unknown): string {
  // Ensure error is a SpreadsheetError or can be treated as one for formatting
  const spreadsheetError =
    error instanceof SpreadsheetError ? error : normalizeError(error);

  const { category, operation } = spreadsheetError;
  let message = spreadsheetError.message;

  // Add operation context if available
  if (operation) {
    message = `${operation}: ${message}`;
  }

  // Category-specific formatting
  switch (category) {
    case ErrorCategory.VALIDATION:
      return `Validation Error: ${message}`;
    case ErrorCategory.PERMISSION:
      return `Permission Error: ${message}`;
    case ErrorCategory.NETWORK:
      return `Network Error: ${message}. Please check your connection and try again.`;
    case ErrorCategory.DATA:
      return `Data Error: ${message}`;
    case ErrorCategory.SYSTEM:
      return `System Error: ${message}`;
    default:
      return message;
  }
}

// ============================================================================
// RETRY AND RECOVERY LOGIC
// ============================================================================

/**
 * Check if an error is non-retryable (transient errors should be retried, non-transient should not)
 */
function isNonRetryableError(error: unknown): boolean {
  // SpreadsheetError instances have built-in retryability
  if (error instanceof SpreadsheetError) {
    return !error.isRetryable;
  }

  // 1. Well-known error classes - always non-retryable
  if (
    error instanceof SpreadsheetValidationError ||
    error instanceof SpreadsheetPermissionError
  ) {
    return true;
  }

  // 2. Error codes/types from Facade API or custom error objects
  if (error && typeof error === 'object') {
    const err = error as Record<string, unknown>;

    // Check for specific error codes/types that indicate permanent failures
    const errorCode = err.code as string | undefined;
    const errorType = err.type as string | undefined;

    // Known non-retryable Facade API error codes
    if (
      errorCode &&
      (NON_RETRYABLE_ERROR_CODES as Set<string>).has(errorCode)
    ) {
      return true;
    }

    // Known non-retryable error types
    if (errorType && NON_RETRYABLE_ERROR_TYPES.has(errorType)) {
      return true;
    }
  }

  // 3. HTTP-style status codes
  if (error && typeof error === 'object') {
    const err = error as Record<string, unknown>;
    const status = err.status as number | undefined;
    const statusCode = err.statusCode as number | undefined;

    // Use whichever status property is available
    const httpStatus = status ?? statusCode;

    if (typeof httpStatus === 'number') {
      // Client errors (400-499) are typically non-retryable
      if (httpStatus >= HTTP_STATUS.BAD_REQUEST && httpStatus < 500) {
        // Specific client errors that might be retryable in some contexts
        return !(POTENTIALLY_RETRYABLE_CLIENT_ERRORS as Set<number>).has(
          httpStatus
        );
      }

      // Some server errors are non-retryable (permanent server issues)
      // RFC 7231: 5xx status codes that indicate permanent server-side problems
      if (
        httpStatus >= HTTP_STATUS.INTERNAL_SERVER_ERROR &&
        (NON_RETRYABLE_SERVER_ERRORS as Set<number>).has(httpStatus)
      ) {
        return true;
      }
    }
  }

  // 4. Error properties (nonRetryable flag, etc.)
  if (error && typeof error === 'object' && 'nonRetryable' in error) {
    return Boolean((error as { nonRetryable: unknown }).nonRetryable);
  }

  // 5. Message substring matching (last resort, conservative patterns)
  // Only use this for very specific, unambiguous error messages
  if (error instanceof Error) {
    const message = error.message.toLowerCase();

    // Very specific validation patterns (avoid generic words)
    const validationPatterns = VALIDATION_ERROR_PATTERNS;

    if (validationPatterns.some((pattern) => message.includes(pattern))) {
      return true;
    }

    // Specific permission patterns
    const permissionPatterns = PERMISSION_ERROR_PATTERNS;

    if (permissionPatterns.some((pattern) => message.includes(pattern))) {
      return true;
    }
  }

  // Default: assume error is retryable (transient network/server issues)
  return false;
}

/**
 * Safely execute a Facade API operation with enhanced error handling and retry logic
 */
export async function safeSpreadsheetOperation<T>(
  operation: () => Promise<T> | T,
  operationName: string,
  fallback?: T,
  maxRetries: number = 1
): Promise<T> {
  let lastError: unknown;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const result = await operation();

      // Log successful retry if this wasn't the first attempt
      if (attempt > 0 && import.meta.env.DEV) {
        console.log(
          `[safeSpreadsheetOperation] ${operationName} succeeded on attempt ${attempt + 1}`
        );
      }

      return result;
    } catch (error) {
      lastError = error;

      // Check if this is a non-retryable error
      if (isNonRetryableError(error)) {
        const spreadsheetError = normalizeError(error, operationName);
        logError(spreadsheetError);
        throw spreadsheetError;
      }

      // Log the error with attempt information
      console.error(
        `[safeSpreadsheetOperation] Failed to ${operationName} (attempt ${attempt + 1}/${maxRetries + 1}):`,
        error
      );

      // If this isn't the last attempt, wait before retrying with exponential backoff
      if (attempt < maxRetries) {
        const backoffMs = EXPONENTIAL_BACKOFF_BASE_MS * 2 ** attempt; // Exponential backoff
        await new Promise((resolve) => setTimeout(resolve, backoffMs));
      }
    }
  }

  // All attempts failed
  if (fallback !== undefined) {
    console.warn(
      `[safeSpreadsheetOperation] Using fallback value for ${operationName} after ${maxRetries + 1} attempts`
    );
    return fallback;
  }

  // Convert to SpreadsheetError
  const spreadsheetError = normalizeError(lastError, operationName);
  logError(spreadsheetError);
  throw spreadsheetError;
}

/**
 * Safely execute a synchronous Facade API operation with error handling
 */
export function safeSpreadsheetOperationSync<T>(
  operation: () => T,
  operationName: string,
  fallback?: T
): T {
  try {
    return operation();
  } catch (error) {
    const spreadsheetError = normalizeError(error, operationName);
    logError(spreadsheetError);

    if (fallback !== undefined) {
      console.warn(
        `[safeSpreadsheetOperationSync] Using fallback value for ${operationName}`
      );
      return fallback;
    }

    throw spreadsheetError;
  }
}
