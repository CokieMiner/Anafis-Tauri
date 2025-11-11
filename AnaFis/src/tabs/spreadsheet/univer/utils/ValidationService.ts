// ValidationService.ts - Consolidated validation service with caching
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
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
export class ValidationService {
  private static cache: ValidationCache = {};
  private static readonly CACHE_TTL = 5000; // 5 seconds

  /**
   * Generate a cache key for the validation request
   */
  private static generateCacheKey(
    variables: Variable[],
    outputValueRange: string,
    outputUncertaintyRange: string
  ): string {
    const varKey = variables.map(v =>
      `${v.name}:${v.valueRange}:${v.uncertaintyRange}:${v.confidence}`
    ).join('|');
    return `${varKey}::${outputValueRange}::${outputUncertaintyRange}`;
  }

  /**
   * Check if cached result is still valid
   */
  private static isCacheValid(timestamp: number): boolean {
    return Date.now() - timestamp < this.CACHE_TTL;
  }

  /**
   * Clear expired cache entries
   */
  private static clearExpiredCache(): void {
    const keys = Object.keys(this.cache);
    keys.forEach(key => {
      if (!this.isCacheValid(this.cache[key]!.timestamp)) {
        delete this.cache[key];
      }
    });
  }

  /**
   * Validate uncertainty propagation setup with caching
   */
  static async validateUncertaintySetup(
    variables: Variable[],
    outputValueRange: string,
    outputUncertaintyRange: string,
    spreadsheetRef: SpreadsheetRef
  ): Promise<ValidationResult> {
    // Clear expired cache entries
    this.clearExpiredCache();

    // Generate cache key
    const cacheKey = this.generateCacheKey(variables, outputValueRange, outputUncertaintyRange);

    // Check cache first
    const cached = this.cache[cacheKey];
    if (cached && this.isCacheValid(cached.timestamp)) {
      return cached.result;
    }

    // Perform validation
    const result = await this.performValidation(variables, outputValueRange, outputUncertaintyRange, spreadsheetRef);

    // Cache the result
    this.cache[cacheKey] = {
      result,
      timestamp: Date.now()
    };

    return result;
  }

  /**
   * Perform the actual validation logic using the unified pipeline
   */
  private static async performValidation(
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
      errors: pipelineResult.errors.map(e => e.message),
      warnings: pipelineResult.warnings.map(w => w.message)
    };
  }

  /**
   * Clear the validation cache (useful for testing or when spreadsheet data changes)
   */
  static clearCache(): void {
    this.cache = {};
  }

  /**
   * Get cache statistics
   */
  static getCacheStats(): { size: number; hits: number; misses: number } {
    return {
      size: Object.keys(this.cache).length,
      hits: 0, // Would need to track this separately
      misses: 0
    };
  }
}