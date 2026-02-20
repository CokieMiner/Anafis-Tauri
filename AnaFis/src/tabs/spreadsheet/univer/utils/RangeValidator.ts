// RangeValidator.ts - Centralized range validation utilities
import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import {
  parseRange,
  type RangeBounds,
} from '@/tabs/spreadsheet/univer/utils/cellUtils';
import { normalizeRangeRef } from '@/tabs/spreadsheet/univer/utils/validation';
import { SpreadsheetValidationError } from './errors';

/**
 * Centralized range validation utilities
 * Provides comprehensive validation for spreadsheet ranges with fail-fast error handling
 */
// biome-ignore lint/complexity/noStaticOnlyClass: Used as a namespace for range operations
export class RangeValidator {
  /**
   * Validate range format (A1 notation)
   * @throws {Error} If range format is invalid
   */
  static validateFormat(range: string): void {
    if (!range || typeof range !== 'string') {
      throw new SpreadsheetValidationError(
        'Range reference must be a non-empty string',
        'range',
        'validateFormat',
        { value: range, type: typeof range }
      );
    }

    try {
      normalizeRangeRef(range.trim());
    } catch (error) {
      const message =
        error instanceof Error ? error.message : 'Invalid range format';
      throw new SpreadsheetValidationError(
        `Invalid range format "${range}": ${message}`,
        'range',
        'validateFormat',
        { range, originalError: message }
      );
    }
  }

  /**
   * Validate range is within sheet bounds
   * @throws {Error} If range exceeds sheet bounds
   */
  static validateBounds(range: string, maxRows: number, maxCols: number): void {
    RangeValidator.validateFormat(range);

    const bounds = parseRange(range);
    if (!bounds) {
      throw new SpreadsheetValidationError(
        `Could not parse range bounds for "${range}"`,
        'range',
        'validateBounds',
        { range, maxRows, maxCols }
      );
    }

    if (bounds.startRow < 0 || bounds.startCol < 0) {
      throw new SpreadsheetValidationError(
        `Range "${range}" contains negative coordinates`,
        'range',
        'validateBounds',
        { range, bounds, maxRows, maxCols }
      );
    }

    if (bounds.endRow >= maxRows || bounds.endCol >= maxCols) {
      throw new SpreadsheetValidationError(
        `Range "${range}" exceeds sheet bounds (${maxRows}×${maxCols})`,
        'range',
        'validateBounds',
        { range, bounds, maxRows, maxCols }
      );
    }
  }

  /**
   * Validate two ranges don't overlap
   * @throws {Error} If ranges overlap
   */
  static validateNoOverlap(range1: string, range2: string): void {
    RangeValidator.validateFormat(range1);
    RangeValidator.validateFormat(range2);

    const bounds1 = parseRange(range1);
    const bounds2 = parseRange(range2);

    if (!bounds1 || !bounds2) {
      throw new SpreadsheetValidationError(
        'Could not parse range bounds for overlap check',
        'ranges',
        'validateNoOverlap',
        { range1, range2, bounds1: !!bounds1, bounds2: !!bounds2 }
      );
    }

    if (RangeValidator.rangesIntersect(bounds1, bounds2)) {
      throw new SpreadsheetValidationError(
        `Ranges "${range1}" and "${range2}" overlap`,
        'ranges',
        'validateNoOverlap',
        { range1, range2, bounds1, bounds2 }
      );
    }
  }

  /**
   * Validate range exists and is accessible in spreadsheet
   * @throws {Error} If range is not accessible
   */
  static async validateAccessible(
    range: string,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<void> {
    RangeValidator.validateFormat(range);

    try {
      await spreadsheetAPI.getRange(range);
    } catch (error) {
      throw new SpreadsheetValidationError(
        `Range "${range}" is not accessible: ${String(error)}`,
        'range',
        'validateAccessible',
        { range, error: String(error) }
      );
    }
  }

  /**
   * Validate range is writable (can read and write data)
   * @throws {Error} If range is not writable
   */
  static async validateWritable(
    range: string,
    spreadsheetAPI: SpreadsheetRef
  ): Promise<void> {
    await RangeValidator.validateAccessible(range, spreadsheetAPI);

    try {
      const testData = await spreadsheetAPI.getRange(range);
      if (!Array.isArray(testData) || testData.length === 0) {
        throw new SpreadsheetValidationError(
          `Range "${range}" appears to be empty or inaccessible`,
          'range',
          'validateWritable',
          {
            range,
            dataLength: Array.isArray(testData) ? testData.length : 'not array',
          }
        );
      }
    } catch (error) {
      throw new SpreadsheetValidationError(
        `Range "${range}" is not writable: ${String(error)}`,
        'range',
        'validateWritable',
        { range, error: String(error) }
      );
    }
  }

  /**
   * Validate multiple ranges don't overlap with each other
   * @throws {Error} If any ranges overlap
   */
  static validateNoOverlaps(ranges: string[]): void {
    for (let i = 0; i < ranges.length; i++) {
      for (let j = i + 1; j < ranges.length; j++) {
        const range1 = ranges[i];
        const range2 = ranges[j];
        if (!range1 || !range2) continue;
        RangeValidator.validateNoOverlap(range1, range2);
      }
    }
  }

  /**
   * Validate ranges don't overlap with a set of reference ranges
   * @throws {Error} If any range overlaps with reference ranges
   */
  static validateNoOverlapWithReferences(
    ranges: string[],
    referenceRanges: string[]
  ): void {
    for (const range of ranges) {
      for (const refRange of referenceRanges) {
        RangeValidator.validateNoOverlap(range, refRange);
      }
    }
  }

  /**
   * Check if two range bounds intersect
   * @private
   */
  private static rangesIntersect(
    bounds1: RangeBounds,
    bounds2: RangeBounds
  ): boolean {
    return !(
      bounds1.endRow < bounds2.startRow ||
      bounds1.startRow > bounds2.endRow ||
      bounds1.endCol < bounds2.startCol ||
      bounds1.startCol > bounds2.endCol
    );
  }
}
