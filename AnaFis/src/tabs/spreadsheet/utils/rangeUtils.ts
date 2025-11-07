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

  // Convert column letters to numbers (A=1, B=2, ..., Z=26, AA=27, etc.)
  const columnToNumber = (col: string): number => {
    let result = 0;
    for (let i = 0; i < col.length; i++) {
      result = result * 26 + (col.charCodeAt(i) - 64);
    }
    return result;
  };

  const startCol = columnToNumber(startColMatch[0].toUpperCase());
  const endCol = columnToNumber(endColMatch[0].toUpperCase());

  return Math.abs(endCol - startCol) + 1;
}

/**
 * Convert bounds to A1 notation start cell.
 *
 * @param bounds - Range bounds with startCol, startRow, endCol, endRow
 * @returns A1 notation cell reference for the start position
 */
export function boundsToA1StartCell(bounds: { startCol: number; startRow: number }): string {
  // Convert column number to letter (1-based to A-based)
  const columnToLetter = (col: number): string => {
    let result = '';
    while (col > 0) {
      col--; // Adjust for 1-based indexing
      result = String.fromCharCode(65 + (col % 26)) + result;
      col = Math.floor(col / 26);
    }
    return result || 'A';
  };

  const colLetter = columnToLetter(bounds.startCol + 1); // Convert to 1-based
  const rowNumber = bounds.startRow + 1; // Convert to 1-based

  return `${colLetter}${rowNumber}`;
}

/**
 * Convert bounds to A1 notation range.
 *
 * @param bounds - Range bounds with startCol, startRow, endCol, endRow
 * @returns A1 notation range reference
 */
export function boundsToA1Range(bounds: { startCol: number; startRow: number; endCol: number; endRow: number }): string {
  const startCell = boundsToA1StartCell(bounds);

  // Convert end column and row to 1-based
  const endColLetter = boundsToA1StartCell({ startCol: bounds.endCol, startRow: 0 }).replace(/\d+$/, '');
  const endRowNumber = bounds.endRow + 1;

  return `${startCell}:${endColLetter}${endRowNumber}`;
}