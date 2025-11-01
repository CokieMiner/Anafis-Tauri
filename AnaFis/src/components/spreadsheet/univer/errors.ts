// errors.ts - Error handling utilities for Facade API operations
export class UniverError extends Error {
  constructor(message: string, public readonly code: string, public readonly originalError?: Error) {
    super(message);
    this.name = 'UniverError';
  }
}

export class UniverValidationError extends UniverError {
  constructor(message: string, public readonly field: string) {
    super(message, 'VALIDATION_ERROR');
    this.name = 'UniverValidationError';
  }
}

export class UniverOperationError extends UniverError {
  constructor(operation: string, originalError?: Error) {
    super(`Failed to ${operation}`, 'OPERATION_FAILED', originalError);
    this.name = 'UniverOperationError';
  }
}

/**
 * Handle Univer errors with consistent logging and error types
 */
export function handleUniverError(operation: string, error: unknown): never {
  const message = `Failed to ${operation}`;
  console.error(message, error);

  if (error instanceof Error) {
    throw new UniverOperationError(operation, error);
  }

  throw new UniverError(message, 'UNIVER_UNKNOWN_ERROR');
}

/**
 * Check if an error is non-retryable (transient errors should be retried, non-transient should not)
 *
 * Classification hierarchy (most specific to least specific):
 * 1. Well-known error classes (UniverValidationError, etc.)
 * 2. Error codes/types from Facade API or custom error objects
 * 3. HTTP-style status codes (400-499 client errors, specific 500+ server errors)
 * 4. Error properties (nonRetryable flag, etc.)
 * 5. Message substring matching (last resort, conservative patterns)
 */
function isNonRetryableError(error: unknown): boolean {
  // 1. Well-known error classes - always non-retryable
  if (error instanceof UniverValidationError) {
    return true; // Validation errors are permanent and shouldn't be retried
  }

  // 2. Error codes/types from Facade API or custom error objects
  if (error && typeof error === 'object') {
    const err = error as Record<string, unknown>;

    // Check for specific error codes/types that indicate permanent failures
    const errorCode = err.code as string | undefined;
    const errorType = err.type as string | undefined;

    // Known non-retryable Facade API error codes
    const nonRetryableCodes = new Set([
      'VALIDATION_ERROR',
      'INVALID_RANGE',
      'INVALID_CELL_REF',
      'PERMISSION_DENIED',
      'UNAUTHORIZED',
      'FORBIDDEN',
      'BAD_REQUEST',
      'MALFORMED_INPUT'
    ]);

    if (errorCode && nonRetryableCodes.has(errorCode)) {
      return true;
    }

    // Known non-retryable error types
    const nonRetryableTypes = new Set([
      'validation',
      'permission',
      'authorization',
      'authentication',
      'bad_request',
      'malformed'
    ]);

    if (errorType && nonRetryableTypes.has(errorType)) {
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
      if (httpStatus >= 400 && httpStatus < 500) {
        // Specific client errors that might be retryable in some contexts
        const potentiallyRetryableClientErrors = new Set([408, 429]); // Request Timeout, Too Many Requests
        return !potentiallyRetryableClientErrors.has(httpStatus);
      }

      // Some server errors are non-retryable (permanent server issues)
      // RFC 7231: 5xx status codes that indicate permanent server-side problems
      const nonRetryableServerErrors = new Set([501, 505, 506, 507, 508, 510, 511]);
      if (httpStatus >= 500 && nonRetryableServerErrors.has(httpStatus)) {
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
    const validationPatterns = [
      'invalid cell reference',
      'invalid range reference',
      'malformed cell reference',
      'malformed range reference'
    ];

    if (validationPatterns.some(pattern => message.includes(pattern))) {
      return true;
    }

    // Specific permission patterns
    const permissionPatterns = [
      'access denied',
      'insufficient permissions',
      'not authorized'
    ];

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
export async function safeUniverOperation<T>(
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
        console.log(`[safeUniverOperation] ${operationName} succeeded on attempt ${attempt + 1}`);
      }

      return result;
    } catch (error) {
      lastError = error;

      // Check if this is a non-retryable error
      if (isNonRetryableError(error)) {
        console.error(`[safeUniverOperation] Non-retryable error in ${operationName}, rethrowing immediately:`, error);
        throw error;
      }

      // Log the error with attempt information
      console.error(`[safeUniverOperation] Failed to ${operationName} (attempt ${attempt + 1}/${maxRetries + 1}):`, error);

      // If this isn't the last attempt, wait before retrying with exponential backoff
      if (attempt < maxRetries) {
        const backoffMs = 100 * Math.pow(2, attempt); // Exponential backoff
        await new Promise(resolve => setTimeout(resolve, backoffMs));
      }
    }
  }

  // All attempts failed
  if (fallback !== undefined) {
    console.warn(`[safeUniverOperation] Using fallback value for ${operationName} after ${maxRetries + 1} attempts`);
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

  throw new UniverOperationError(operationName, errorToThrow);
}

// Compiled regex patterns for better performance
const CELL_REF_PATTERN = /^[A-Z]+\d+$/;
const RANGE_REF_PATTERN = /^[A-Z]+\d+(:[A-Z]+\d+)?$/;

// Enhanced caching with LRU-like behavior
const validationCache = new Map<string, boolean>();
const MAX_CACHE_SIZE = 1000;

/**
 * Validate cell reference format with enhanced caching
 */
export function isValidCellRef(cellRef: string): boolean {
  const cacheKey = `cell:${cellRef}`;

  // Check cache first
  if (validationCache.has(cacheKey)) {
    // Move to end (LRU behavior)
    const value = validationCache.get(cacheKey)!;
    validationCache.delete(cacheKey);
    validationCache.set(cacheKey, value);
    return value;
  }

  const isValid = CELL_REF_PATTERN.test(cellRef);

  // Cache management with LRU eviction
  if (validationCache.size >= MAX_CACHE_SIZE) {
    // Remove oldest entry (first in Map)
    const firstKey = validationCache.keys().next().value;
    if (firstKey) {
      validationCache.delete(firstKey);
    }
  }

  validationCache.set(cacheKey, isValid);
  return isValid;
}

/**
 * Validate range reference format with caching
 */
export function isValidRangeRef(rangeRef: string): boolean {
  const cacheKey = `range:${rangeRef}`;

  if (validationCache.has(cacheKey)) {
    const value = validationCache.get(cacheKey)!;
    validationCache.delete(cacheKey);
    validationCache.set(cacheKey, value);
    return value;
  }

  const isValid = RANGE_REF_PATTERN.test(rangeRef);

  if (validationCache.size >= MAX_CACHE_SIZE) {
    const firstKey = validationCache.keys().next().value;
    if (firstKey) {
      validationCache.delete(firstKey);
    }
  }

  validationCache.set(cacheKey, isValid);
  return isValid;
}

/**
 * Normalize range reference with validation
 */
export function normalizeRangeRef(rangeRef: string): string {
  if (!isValidRangeRef(rangeRef)) {
    throw new UniverValidationError(`Invalid range reference: ${rangeRef}`, 'rangeRef');
  }
  return rangeRef.toUpperCase();
}

/**
 * Validate and normalize cell reference
 */
export function normalizeCellRef(cellRef: string): string {
  if (!isValidCellRef(cellRef)) {
    throw new UniverValidationError(`Invalid cell reference: ${cellRef}`, 'cellRef');
  }
  return cellRef.toUpperCase();
}

/**
 * Clear validation cache (useful for testing or memory management)
 */
export function clearValidationCache(): void {
  validationCache.clear();
}

/**
 * Performance monitoring utility for Facade API operations
 */
export function withPerformanceMonitoring<T extends (...args: unknown[]) => unknown>(
  fn: T,
  operationName: string
): T {
  return ((...args: Parameters<T>) => {
    const startTime = performance.now();

    try {
      const result = fn(...args);

      // Handle both sync and async functions
      if (result instanceof Promise) {
        return result
          .then((res: Awaited<ReturnType<T>>) => {
            const endTime = performance.now();
            if (process.env.NODE_ENV === 'development') {
              console.log(`[Performance] ${operationName} completed in ${(endTime - startTime).toFixed(2)}ms`);
            }
            return res;
          })
          .catch((err: unknown) => {
            const endTime = performance.now();
            if (process.env.NODE_ENV === 'development') {
              console.error(`[Performance] ${operationName} failed after ${(endTime - startTime).toFixed(2)}ms:`, err);
            }
            throw err;
          });
      } else {
        const endTime = performance.now();
        if (process.env.NODE_ENV === 'development') {
          console.log(`[Performance] ${operationName} took ${(endTime - startTime).toFixed(2)}ms`);
        }
        return result;
      }
    } catch (error) {
      const endTime = performance.now();
      if (process.env.NODE_ENV === 'development') {
        console.error(`[Performance] ${operationName} failed after ${(endTime - startTime).toFixed(2)}ms:`, error);
      }
      throw error;
    }
  }) as T;
}