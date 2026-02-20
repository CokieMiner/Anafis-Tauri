// ValidationService.ts - Consolidated validation service with caching
import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { ValidationPipeline, type Variable } from './ValidationPipeline';

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

export interface ValidationCache {
  [key: string]: {
    result: ValidationResult;
    timestamp: number;
  };
}

/**
 * Consolidated validation service that handles all uncertainty propagation validation
 * with caching to avoid redundant validation calls.
 */
let cache: ValidationCache = {};
const CACHE_TTL = 5000; // 5 seconds

/**
 * Generate a cache key for the validation request
 */
function generateCacheKey(
  variables: Variable[],
  outputValueRange: string,
  outputUncertaintyRange: string
): string {
  const varKey = variables
    .map(
      (v) => `${v.name}:${v.valueRange}:${v.uncertaintyRange}:${v.confidence}`
    )
    .join('|');
  return `${varKey}::${outputValueRange}::${outputUncertaintyRange}`;
}

/**
 * Check if cached result is still valid
 */
function isCacheValid(timestamp: number): boolean {
  return Date.now() - timestamp < CACHE_TTL;
}

/**
 * Clear expired cache entries
 */
function clearExpiredCache(): void {
  const keys = Object.keys(cache);
  keys.forEach((key) => {
    const cacheEntry = cache[key];
    if (cacheEntry && !isCacheValid(cacheEntry.timestamp)) {
      delete cache[key];
    }
  });
}

/**
 * Perform the actual validation logic using the unified pipeline
 */
async function performValidation(
  variables: Variable[],
  outputValueRange: string,
  outputUncertaintyRange: string,
  spreadsheetRef: SpreadsheetRef
): Promise<ValidationResult> {
  // Use the unified validation pipeline
  const pipelineResult = await ValidationPipeline.validateUncertaintySetup(
    variables,
    outputValueRange,
    outputUncertaintyRange,
    spreadsheetRef
  );

  // Convert structured errors/warnings to string arrays for backward compatibility
  return {
    isValid: pipelineResult.isValid,
    errors: pipelineResult.errors.map((e) => e.message),
    warnings: pipelineResult.warnings.map((w) => w.message),
  };
}

export const ValidationService = {
  /**
   * Validate uncertainty propagation setup with caching
   */
  async validateUncertaintySetup(
    variables: Variable[],
    outputValueRange: string,
    outputUncertaintyRange: string,
    spreadsheetRef: SpreadsheetRef
  ): Promise<ValidationResult> {
    // Clear expired cache entries
    clearExpiredCache();

    // Generate cache key
    const cacheKey = generateCacheKey(
      variables,
      outputValueRange,
      outputUncertaintyRange
    );

    // Check cache first
    const cached = cache[cacheKey];
    if (cached && isCacheValid(cached.timestamp)) {
      return cached.result;
    }

    // Perform validation
    const result = await performValidation(
      variables,
      outputValueRange,
      outputUncertaintyRange,
      spreadsheetRef
    );

    // Cache the result
    cache[cacheKey] = {
      result,
      timestamp: Date.now(),
    };

    return result;
  },

  /**
   * Clear the validation cache (useful for testing or when spreadsheet data changes)
   */
  clearCache(): void {
    cache = {};
  },

  /**
   * Get cache statistics
   */
  getCacheStats(): { size: number; hits: number; misses: number } {
    return {
      size: Object.keys(cache).length,
      hits: 0, // Would need to track this separately
      misses: 0,
    };
  },
};
