// Standardized Result type for consistent error handling across operations
// Provides a type-safe alternative to throwing exceptions or using success/error objects

/**
 * Result type representing either success with a value or failure with an error
 */
export type Result<T, E> =
  | { ok: true; value: T }
  | { ok: false; error: E };

/**
 * Create a successful result
 */
export function ok<T, E = Error>(value: T): Result<T, E> {
  return { ok: true, value };
}

/**
 * Create a failed result
 */
export function err<T = never, E = Error>(error: E): Result<T, E> {
  return { ok: false, error };
}

/**
 * Check if a result is successful
 */
export function isOk<T, E>(result: Result<T, E>): result is { ok: true; value: T } {
  return result.ok;
}

/**
 * Check if a result is failed
 */
export function isErr<T, E>(result: Result<T, E>): result is { ok: false; error: E } {
  return !result.ok;
}

/**
 * Get the value from a successful result, or throw the error
 */
export function unwrap<T, E>(result: Result<T, E>): T {
  if (isErr(result)) {
    throw result.error instanceof Error ? result.error : new Error(String(result.error));
  }
  return result.value;
}

/**
 * Get the value from a successful result, or return a default value
 */
export function unwrapOr<T, E>(result: Result<T, E>, defaultValue: T): T {
  return isOk(result) ? result.value : defaultValue;
}

/**
 * Get the value from a successful result, or compute a default value
 */
export function unwrapOrElse<T, E>(result: Result<T, E>, defaultFn: (error: E) => T): T {
  return isOk(result) ? result.value : defaultFn(result.error);
}

/**
 * Transform a successful result's value using a function
 */
export function map<T, U, E>(result: Result<T, E>, fn: (value: T) => U): Result<U, E> {
  return isOk(result) ? ok<U, E>(fn(result.value)) : result;
}

/**
 * Transform a failed result's error using a function
 */
export function mapErr<T, E, F>(result: Result<T, E>, fn: (error: E) => F): Result<T, F> {
  return isErr(result) ? err(fn(result.error)) : result;
}

/**
 * Chain operations on successful results
 */
export function andThen<T, U, E>(result: Result<T, E>, fn: (value: T) => Result<U, E>): Result<U, E> {
  return isOk(result) ? fn(result.value) : result;
}