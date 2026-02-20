// validation.ts - Cell and range reference validation utilities
import { SpreadsheetValidationError } from '@/tabs/spreadsheet/univer';

// Compiled regex patterns for better performance
const RANGE_REF_PATTERN = /^[A-Z]+\d+(:[A-Z]+\d+)?$/;

// Enhanced caching with LRU-like behavior
const validationCache = new Map<string, boolean>();
const MAX_CACHE_SIZE = 1000;

/**
 * Validate range reference format with caching
 * @example isValidRangeRef("A1:B2") // true
 * @example isValidRangeRef("A1") // true (single cell is valid range)
 * @example isValidRangeRef("A1:") // false
 */
function isValidRangeRef(rangeRef: string): boolean {
  const cacheKey = `range:${rangeRef}`;

  if (validationCache.has(cacheKey)) {
    const value = validationCache.get(cacheKey) ?? false;
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
    throw new SpreadsheetValidationError(
      `Invalid range reference: ${rangeRef}`,
      'rangeRef'
    );
  }
  return rangeRef.toUpperCase();
}
