/**
 * Generic range utility functions - implementation agnostic.
 *
 * These utilities work with A1 notation and are independent of the underlying
 * spreadsheet library (Univer, AG Grid, etc.). They can be used by any adapter.
 *
 * Coordinate System Convention:
 * - A1 notation uses 1-based indexing (A1 = first cell, row 1, column A)
 * - Internal coordinates use 0-based indexing for array operations
 */

import { CellReference } from './CellReference';

/**
 * Extract the starting cell from a range reference.
 *
 * @example
 * extractStartCell("A1:C10") // Returns "A1"
 * extractStartCell("B5")     // Returns "B5"
 *
 * @param rangeRef - Range in A1 notation (e.g., "A1:C10" or "B5")
 * @returns Starting cell reference (e.g., "A1")
 */
export function extractStartCell(rangeRef: string): string {
  return rangeRef.includes(':') ? rangeRef.split(':')[0]! : rangeRef;
}

/**
 * Extract the ending cell from a range reference.
 *
 * @example
 * extractEndCell("A1:C10") // Returns "C10"
 * extractEndCell("B5")     // Returns "B5"
 *
 * @param rangeRef - Range in A1 notation (e.g., "A1:C10" or "B5")
 * @returns Ending cell reference (e.g., "C10")
 */
export function extractEndCell(rangeRef: string): string {
  if (!rangeRef.includes(':')) {return rangeRef;}
  const parts = rangeRef.split(':');
  return parts[1] ?? rangeRef;
}

/**
 * Check if a range reference is a single cell or a multi-cell range.
 *
 * @example
 * isSingleCell("A1")     // Returns true
 * isSingleCell("A1:C10") // Returns false
 *
 * @param rangeRef - Range in A1 notation
 * @returns true if the range represents a single cell, false otherwise
 */
export function isSingleCell(rangeRef: string): boolean {
  return !rangeRef.includes(':');
}

/**
 * Get the bounds of a range in terms of row and column counts.
 *
 * @example
 * getRangeRowCount("A1:C10")    // Returns 10
 * getRangeColumnCount("A1:C10") // Returns 3
 *
 * @param rangeRef - Range in A1 notation
 * @returns Number of rows or columns in the range
 */
export function getRangeRowCount(rangeRef: string): number {
  if (isSingleCell(rangeRef)) {return 1;}

  const [startCell, endCell] = rangeRef.split(':');
  if (!startCell || !endCell) {return 1;}

  // Extract row numbers (assuming format like A1, B10, etc.)
  const startRowMatch = startCell.match(/\d+/);
  const endRowMatch = endCell.match(/\d+/);

  if (!startRowMatch || !endRowMatch) {return 1;}

  const startRow = parseInt(startRowMatch[0], 10);
  const endRow = parseInt(endRowMatch[0], 10);

  return Math.abs(endRow - startRow) + 1;
}

export function getRangeColumnCount(rangeRef: string): number {
  if (isSingleCell(rangeRef)) {return 1;}

  const [startCell, endCell] = rangeRef.split(':');
  if (!startCell || !endCell) {return 1;}

  // Extract column letters (assuming format like A1, B10, etc.)
  const startColMatch = startCell.match(/^[A-Z]+/i);
  const endColMatch = endCell.match(/^[A-Z]+/i);

  if (!startColMatch || !endColMatch) {return 1;}

  const startCol = CellReference.letterToColumn(startColMatch[0].toUpperCase());
  const endCol = CellReference.letterToColumn(endColMatch[0].toUpperCase());

  return Math.abs(endCol - startCol) + 1;
}

export function boundsToA1StartCell(bounds: { startCol: number; startRow: number }): string {
  // Convert 0-based bounds to 1-based for A1 notation
  const colLetter = CellReference.columnToLetter(bounds.startCol);
  const rowNumber = bounds.startRow + 1;

  return `${colLetter}${rowNumber}`;
}

export function boundsToA1Range(bounds: { startCol: number; startRow: number; endCol: number; endRow: number }): string {
  const startCell = boundsToA1StartCell(bounds);

  // Convert end column and row to 1-based
  const endColLetter = CellReference.columnToLetter(bounds.endCol);
  const endRowNumber = bounds.endRow + 1;

  return `${startCell}:${endColLetter}${endRowNumber}`;
}