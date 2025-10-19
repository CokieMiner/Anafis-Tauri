// errors.ts - Error handling utilities for Univer operations
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
 * Safely execute a Univer operation with error handling
 */
export async function safeUniverOperation<T>(
  operation: () => Promise<T> | T,
  operationName: string,
  fallback?: T
): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    console.error(`Failed to ${operationName}:`, error);
    if (fallback !== undefined) {
      return fallback;
    }
    throw error;
  }
}

/**
 * Validate cell reference format
 */
export function isValidCellRef(cellRef: string): boolean {
  return /^[A-Z]+\d+$/.test(cellRef);
}

/**
 * Validate range reference format
 */
export function isValidRangeRef(rangeRef: string): boolean {
  return /^[A-Z]+\d+(:[A-Z]+\d+)?$/.test(rangeRef);
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
 * Validate and normalize range reference
 */
export function normalizeRangeRef(rangeRef: string): string {
  if (!isValidRangeRef(rangeRef)) {
    throw new UniverValidationError(`Invalid range reference: ${rangeRef}`, 'rangeRef');
  }
  return rangeRef.toUpperCase();
}