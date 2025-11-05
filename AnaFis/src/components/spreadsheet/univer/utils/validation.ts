// validation.ts - Cell and range reference validation utilities
import { SpreadsheetValidationError } from '../index';

// Compiled regex patterns for better performance
const CELL_REF_PATTERN = /^[A-Z]+\d+$/;
const RANGE_REF_PATTERN = /^[A-Z]+\d+(:[A-Z]+\d+)?$/;

// Enhanced caching with LRU-like behavior
const validationCache = new Map<string, boolean>();
const MAX_CACHE_SIZE = 1000;

/**
 * Validate cell reference format with enhanced caching
 * @example isValidCellRef("A1") // true
 * @example isValidCellRef("XY123") // true
 * @example isValidCellRef("A1:B2") // false (use isValidRangeRef)
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
 * @example isValidRangeRef("A1:B2") // true
 * @example isValidRangeRef("A1") // true (single cell is valid range)
 * @example isValidRangeRef("A1:") // false
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
 * @throws {SpreadsheetValidationError} If range reference is invalid
 */
export function normalizeRangeRef(rangeRef: string): string {
  if (!isValidRangeRef(rangeRef)) {
    throw new SpreadsheetValidationError(`Invalid range reference: ${rangeRef}`, 'rangeRef');
  }
  return rangeRef.toUpperCase();
}

/**
 * Validate and normalize cell reference
 * @throws {SpreadsheetValidationError} If cell reference is invalid
 */
export function normalizeCellRef(cellRef: string): string {
  if (!isValidCellRef(cellRef)) {
    throw new SpreadsheetValidationError(`Invalid cell reference: ${cellRef}`, 'cellRef');
  }
  return cellRef.toUpperCase();
}

/**
 * Clear validation cache (useful for testing or memory management)
 */
export function clearValidationCache(): void {
  validationCache.clear();
}
