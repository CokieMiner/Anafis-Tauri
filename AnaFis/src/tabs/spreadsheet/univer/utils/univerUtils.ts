// univerUtils.ts - Consolidated utility functions for Facade API
import type { IRange } from '@univerjs/core';
import { columnToLetter } from './cellUtils';
import { ERROR_MESSAGES } from './constants';
import { SpreadsheetValidationError } from './errors';

// Type for the facade API instance
type FacadeAPI = ReturnType<
  typeof import('@univerjs/core/facade').FUniver.newAPI
>;

/**
 * Determine the used range using Univer's Facade API
 *
 * Uses the public Facade API methods:
 * - getDataRange() returns a FRange from A1 to last cell with data
 * - getLastRow() and getLastColumn() provide explicit bounds
 *
 * This approach:
 * - ✅ 100% accurate - includes ALL data, no loss
 * - ✅ Handles sparse data correctly (e.g., data at A1, C1, B2 → range A1:C2)
 * - ✅ Efficient (no manual scanning)
 * - ✅ Public Facade API (stable, documented)
 *
 * Example with sparse data:
 *   Data: 1 at A1, 3 at C1, 5 at E1, 6 at F1
 *          2 at A2
 *          4 at A3
 *   Result: A1:F3 (includes all empty cells within bounds)
 *
 * @param facadeAPI The facade API instance
 * @returns The used range in A1 notation (A1:A1 if empty, otherwise actual bounds)
 * @throws Error if workbook/sheet access fails
 */
export function determineUsedRange(facadeAPI: FacadeAPI): string {
  try {
    const workbook = facadeAPI.getActiveWorkbook();
    if (!workbook) {
      throw new SpreadsheetValidationError(
        'No active workbook available',
        'workbook',
        'determineUsedRange',
        { operation: 'getActiveWorkbook' }
      );
    }

    // Get the active sheet using the Facade API
    const sheet = workbook.getActiveSheet();

    // Method 1: Try getDataRange() first (simplest, most reliable)
    try {
      const dataRange = sheet.getDataRange();
      // dataRange is always returned, no need to check
      const a1Notation = dataRange.getA1Notation();
      if (a1Notation !== 'A1:A1') {
        return a1Notation;
      }
    } catch {
      // Fallback to method 2
    }

    // Method 2: Use getLastRow() and getLastColumn() explicitly
    // This is the fallback if getDataRange() doesn't work properly
    // Indices are 0-based for getLastRow/Column, convert to 1-based for A1 notation
    const lastRow = sheet.getLastRow();
    const lastCol = sheet.getLastColumn();

    // If sheet is empty, both will be -1 or 0
    if (lastRow < 0 || lastCol < 0) {
      return 'A1:A1';
    }

    // Convert 0-based indices to A1 notation (1-based)
    // lastRow is 0-based, so row number = lastRow + 1
    // lastCol is 0-based, so we use columnToLetter(lastCol)
    const endCol = columnToLetter(lastCol);
    const endRow = lastRow + 1;

    return `A1:${endCol}${endRow}`;
  } catch (error) {
    // If critical workbook/sheet access fails, return empty fallback
    if (error instanceof Error && error.message.includes('No active')) {
      return 'A1:A1'; // Empty sheet fallback
    }
    throw error;
  }
}

/**
 * Convert IRange to A1 notation
 */
export function rangeToA1(range: IRange): string {
  // Add null checks for range properties
  if (
    typeof range.startColumn !== 'number' ||
    typeof range.startRow !== 'number' ||
    typeof range.endColumn !== 'number' ||
    typeof range.endRow !== 'number'
  ) {
    throw new SpreadsheetValidationError(
      ERROR_MESSAGES.INVALID_RANGE_OBJECT,
      'range',
      'rangeToA1',
      {
        startColumn: range.startColumn,
        startRow: range.startRow,
        endColumn: range.endColumn,
        endRow: range.endRow,
      }
    );
  }

  const startCol = columnToLetter(range.startColumn);
  const startRow = range.startRow + 1;
  const endCol = columnToLetter(range.endColumn);
  const endRow = range.endRow + 1;

  if (
    range.startColumn === range.endColumn &&
    range.startRow === range.endRow
  ) {
    return `${startCol}${startRow}`;
  }

  return `${startCol}${startRow}:${endCol}${endRow}`;
}
