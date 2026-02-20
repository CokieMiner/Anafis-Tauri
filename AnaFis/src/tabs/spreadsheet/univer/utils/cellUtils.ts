/**
 * Utility functions for cell reference parsing and conversion
 */

import {
  CellReference,
  type RangeBounds,
} from '@/tabs/spreadsheet/utils/CellReference';

// Re-export for backward compatibility
export type { RangeBounds };

/**
 * Converts a column number to letter notation (0 = A, 1 = B, etc.)
 * @param col Zero-based column index
 * @returns Column letter(s) like 'A', 'B', 'AA', etc.
 */
export function columnToLetter(col: number): string {
  return CellReference.columnToLetter(col);
}

/**
 * Parses A1 notation and returns 0-indexed row/col coordinates
 * @param cellRef Cell reference like 'A1', 'B2', 'AA10'
 * @returns Object with row and col properties, or null if invalid
 */
export function parseCellRef(
  cellRef: string
): { row: number; col: number } | null {
  try {
    return CellReference.parseCell(cellRef);
  } catch {
    return null;
  }
}

/**
 * Parse range reference (A1 notation) to 0-based bounds.
 *
 * @example
 * parseRange("A1:C10")
 * // Returns { startCol: 0, startRow: 0, endCol: 2, endRow: 9 }
 *
 * parseRange("B5")
 * // Returns { startCol: 1, startRow: 4, endCol: 1, endRow: 4 }
 *
 * @param rangeRef - Range in A1 notation (e.g., "A1:C10" or "B5")
 * @returns RangeBounds with 0-based coordinates, or null if invalid
 */
export function parseRange(rangeRef: string): RangeBounds | null {
  try {
    return CellReference.getRangeBounds(rangeRef);
  } catch {
    return null;
  }
}

/**
 * Converts a column letter to number (A = 0, B = 1, etc.)
 * @param letter Column letter(s) like 'A', 'B', 'AA', etc.
 * @returns Zero-based column index
 */
export function letterToColumn(letter: string): number {
  return CellReference.letterToColumn(letter);
}
