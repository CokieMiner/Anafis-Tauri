// RangeValidator.ts - Centralized range validation utilities
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { parseRange, type RangeBounds } from '@/tabs/spreadsheet/univer/utils/cellUtils';
import { normalizeRangeRef } from '@/tabs/spreadsheet/univer/utils/validation';

/**
 * Centralized range validation utilities
 * Provides comprehensive validation for spreadsheet ranges with fail-fast error handling
 */
export class RangeValidator {
  /**
   * Validate range format (A1 notation)
   * @throws {Error} If range format is invalid
   */
  static validateFormat(range: string): void {
    if (!range || typeof range !== 'string') {
      throw new Error('Range reference must be a non-empty string');
    }

    try {
      normalizeRangeRef(range.trim());
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Invalid range format';
      throw new Error(`Invalid range format "${range}": ${message}`);
    }
  }

  /**
   * Validate range is within sheet bounds
   * @throws {Error} If range exceeds sheet bounds
   */
  static validateBounds(range: string, maxRows: number, maxCols: number): void {
    this.validateFormat(range);

    const bounds = parseRange(range);
    if (!bounds) {
      throw new Error(`Could not parse range bounds for "${range}"`);
    }

    if (bounds.startRow < 0 || bounds.startCol < 0) {
      throw new Error(`Range "${range}" contains negative coordinates`);
    }

    if (bounds.endRow >= maxRows || bounds.endCol >= maxCols) {
      throw new Error(`Range "${range}" exceeds sheet bounds (${maxRows}×${maxCols})`);
    }
  }

  /**
   * Validate two ranges don't overlap
   * @throws {Error} If ranges overlap
   */
  static validateNoOverlap(range1: string, range2: string): void {
    this.validateFormat(range1);
    this.validateFormat(range2);

    const bounds1 = parseRange(range1);
    const bounds2 = parseRange(range2);

    if (!bounds1 || !bounds2) {
      throw new Error('Could not parse range bounds for overlap check');
    }

    if (this.rangesIntersect(bounds1, bounds2)) {
      throw new Error(`Ranges "${range1}" and "${range2}" overlap`);
    }
  }

  /**
   * Validate range exists and is accessible in spreadsheet
   * @throws {Error} If range is not accessible
   */
  static async validateAccessible(range: string, spreadsheetAPI: SpreadsheetRef): Promise<void> {
    this.validateFormat(range);

    try {
      await spreadsheetAPI.getRange(range);
    } catch (error) {
      throw new Error(`Range "${range}" is not accessible: ${String(error)}`);
    }
  }

  /**
   * Validate range is writable (can read and write data)
   * @throws {Error} If range is not writable
   */
  static async validateWritable(range: string, spreadsheetAPI: SpreadsheetRef): Promise<void> {
    await this.validateAccessible(range, spreadsheetAPI);

    try {
      const testData = await spreadsheetAPI.getRange(range);
      if (!Array.isArray(testData) || testData.length === 0) {
        throw new Error(`Range "${range}" appears to be empty or inaccessible`);
      }
    } catch (error) {
      throw new Error(`Range "${range}" is not writable: ${String(error)}`);
    }
  }

  /**
   * Validate multiple ranges don't overlap with each other
   * @throws {Error} If any ranges overlap
   */
  static validateNoOverlaps(ranges: string[]): void {
    for (let i = 0; i < ranges.length; i++) {
      for (let j = i + 1; j < ranges.length; j++) {
        const range1 = ranges[i]!;
        const range2 = ranges[j]!;
        this.validateNoOverlap(range1, range2);
      }
    }
  }

  /**
   * Validate ranges don't overlap with a set of reference ranges
   * @throws {Error} If any range overlaps with reference ranges
   */
  static validateNoOverlapWithReferences(ranges: string[], referenceRanges: string[]): void {
    for (const range of ranges) {
      for (const refRange of referenceRanges) {
        this.validateNoOverlap(range, refRange);
      }
    }
  }

  /**
   * Check if two range bounds intersect
   * @private
   */
  private static rangesIntersect(bounds1: RangeBounds, bounds2: RangeBounds): boolean {
    return !(bounds1.endRow < bounds2.startRow ||
             bounds1.startRow > bounds2.endRow ||
             bounds1.endCol < bounds2.startCol ||
             bounds1.startCol > bounds2.endCol);
  }
}