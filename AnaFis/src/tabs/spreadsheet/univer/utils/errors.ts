// errors.ts - Error classes and handling utilities for spreadsheet operations
import {
  NON_RETRYABLE_ERROR_CODES,
  NON_RETRYABLE_ERROR_TYPES,
  HTTP_STATUS,
  POTENTIALLY_RETRYABLE_CLIENT_ERRORS,
  NON_RETRYABLE_SERVER_ERRORS,
  VALIDATION_ERROR_PATTERNS,
  PERMISSION_ERROR_PATTERNS,
} from './constants';
export class SpreadsheetError extends Error {
  constructor(message: string, public readonly code: string, public readonly originalError?: Error) {
    super(message);
    this.name = 'SpreadsheetError';
  }
}

export class SpreadsheetValidationError extends SpreadsheetError {
  constructor(message: string, public readonly field: string) {
    super(message, 'VALIDATION_ERROR');
    this.name = 'SpreadsheetValidationError';
  }
}

export class SpreadsheetOperationError extends SpreadsheetError {
  constructor(operation: string, originalError?: Error) {
    super(`Failed to ${operation}`, 'OPERATION_FAILED', originalError);
    this.name = 'SpreadsheetOperationError';
  }
}

/**
 * Handle spreadsheet errors with consistent logging and error types
 */
export function handleSpreadsheetError(operation: string, error: unknown): never {
  const message = `Failed to ${operation}`;
  console.error(message, error);

  if (error instanceof Error) {
    throw new SpreadsheetOperationError(operation, error);
  }

  throw new SpreadsheetError(message, 'SPREADSHEET_UNKNOWN_ERROR');
}

/**
 * Check if an error is non-retryable (transient errors should be retried, non-transient should not)
 *
 * Classification hierarchy (most specific to least specific):
 * 1. Well-known error classes
 * 2. Error codes/types from Facade API or custom error objects
 * 3. HTTP-style status codes (400-499 client errors, specific 500+ server errors)
 * 4. Error properties (nonRetryable flag, etc.)
 * 5. Message substring matching (last resort, conservative patterns)
 */
function isNonRetryableError(error: unknown): boolean {
  // 1. Well-known error classes - always non-retryable
  if (error instanceof SpreadsheetValidationError) {
    return true; // Validation errors are permanent and shouldn't be retried
  }

  // 2. Error codes/types from Facade API or custom error objects
  if (error && typeof error === 'object') {
    const err = error as Record<string, unknown>;

    // Check for specific error codes/types that indicate permanent failures
    const errorCode = err.code as string | undefined;
    const errorType = err.type as string | undefined;

    // Known non-retryable Facade API error codes
    if (errorCode && (NON_RETRYABLE_ERROR_CODES as Set<string>).has(errorCode)) {
      return true;
    }

    // Known non-retryable error types
    if (errorType && (NON_RETRYABLE_ERROR_TYPES).has(errorType)) {
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
        return !(POTENTIALLY_RETRYABLE_CLIENT_ERRORS as Set<number>).has(httpStatus);
      }

      // Some server errors are non-retryable (permanent server issues)
      // RFC 7231: 5xx status codes that indicate permanent server-side problems
      if (httpStatus >= HTTP_STATUS.INTERNAL_SERVER_ERROR && (NON_RETRYABLE_SERVER_ERRORS as Set<number>).has(httpStatus)) {
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

    if (validationPatterns.some(pattern => message.includes(pattern))) {
      return true;
    }

    // Specific permission patterns
    const permissionPatterns = PERMISSION_ERROR_PATTERNS;

    if (permissionPatterns.some(pattern => message.includes(pattern))) {
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
      if (attempt > 0 && process.env.NODE_ENV === 'development') {
        console.log(`[safeSpreadsheetOperation] ${operationName} succeeded on attempt ${attempt + 1}`);
      }

      return result;
    } catch (error) {
      lastError = error;

      // Check if this is a non-retryable error
      if (isNonRetryableError(error)) {
        console.error(`[safeSpreadsheetOperation] Non-retryable error in ${operationName}, rethrowing immediately:`, error);
        throw error;
      }

      // Log the error with attempt information
      console.error(`[safeSpreadsheetOperation] Failed to ${operationName} (attempt ${attempt + 1}/${maxRetries + 1}):`, error);

      // If this isn't the last attempt, wait before retrying with exponential backoff
      if (attempt < maxRetries) {
        const backoffMs = 100 * Math.pow(2, attempt); // Exponential backoff
        await new Promise(resolve => setTimeout(resolve, backoffMs));
      }
    }
  }

  // All attempts failed
  if (fallback !== undefined) {
    console.warn(`[safeSpreadsheetOperation] Using fallback value for ${operationName} after ${maxRetries + 1} attempts`);
    return fallback;
  }

  // Safely convert lastError to Error instance
  let errorToThrow: Error;
  if (lastError instanceof Error) {
    errorToThrow = lastError;
  } else if (lastError && typeof lastError === 'object' && 'message' in lastError) {
    // Handle objects with message property (like custom error objects)
    errorToThrow = new Error(String(lastError.message));
  } else {
    // Handle strings, numbers, or other types
    errorToThrow = new Error(String(lastError));
  }

  throw new SpreadsheetOperationError(operationName, errorToThrow);
}

/**
 * Format spreadsheet operation errors for user-friendly display
 * @param err - The error to format
 * @param operation - The operation type ('export', 'import', 'general')
 * @returns User-friendly error message
 */
export function formatSpreadsheetError(
  err: unknown,
  operation: 'export' | 'import' | 'general' = 'general'
): string {
  const msg = err instanceof Error ? err.message : String(err);

  // Common patterns across all operations
  if (msg.includes('Invalid range')) {
    return `Range error: ${msg}`;
  }

  if (msg.includes('permission') || msg.includes('denied')) {
    const action = operation === 'export' ? 'write to' : 'read';
    return `Permission denied: Cannot ${action} file`;
  }

  if (msg.includes('disk') || msg.includes('space')) {
    return 'Insufficient disk space';
  }

  if (msg.includes('timeout')) {
    return `${operation.charAt(0).toUpperCase() + operation.slice(1)} timed out - try smaller range`;
  }

  // Operation-specific patterns
  if (operation === 'import') {
    if (msg.includes('No such file')) {
      return 'File not found';
    }
    if (msg.includes('encoding') || msg.includes('charset')) {
      return `Encoding error: ${msg}`;
    }
  }

  // Default operation-specific message
  const operationName = operation.charAt(0).toUpperCase() + operation.slice(1);
  return `${operationName} failed: ${msg}`;
}
