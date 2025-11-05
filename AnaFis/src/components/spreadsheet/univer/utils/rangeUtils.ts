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

// Constants for column conversion
const ALPHABET_SIZE = 26;
const CHAR_CODE_A = 65;

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
 * @returns True if single cell, false if range
 */
export function isSingleCell(rangeRef: string): boolean {
  return !rangeRef.includes(':');
}

/**
 * Convert column letter to 0-based column index.
 * 
 * @example
 * letterToColumn("A")  // Returns 0
 * letterToColumn("Z")  // Returns 25
 * letterToColumn("AA") // Returns 26
 * 
 * @param letter - Column letter(s) (e.g., "A", "Z", "AA")
 * @returns 0-based column index
 */
export function letterToColumn(letter: string): number {
  let col = 0;
  for (let i = 0; i < letter.length; i++) {
    col = col * ALPHABET_SIZE + (letter.charCodeAt(i) - CHAR_CODE_A + 1);
  }
  return col - 1;
}

/**
 * Convert 0-based column index to column letter.
 * 
 * @example
 * columnToLetter(0)  // Returns "A"
 * columnToLetter(25) // Returns "Z"
 * columnToLetter(26) // Returns "AA"
 * 
 * @param col - 0-based column index
 * @returns Column letter(s)
 */
export function columnToLetter(col: number): string {
  let letter = '';
  let index = col;
  
  while (index >= 0) {
    letter = String.fromCharCode((index % ALPHABET_SIZE) + CHAR_CODE_A) + letter;
    index = Math.floor(index / ALPHABET_SIZE) - 1;
  }
  
  return letter;
}

/**
 * Parse cell reference (A1 notation) to 0-based coordinates.
 * 
 * @example
 * parseCellRef("A1")  // Returns { row: 0, col: 0 }
 * parseCellRef("C10") // Returns { row: 9, col: 2 }
 * 
 * @param cellRef - Cell reference in A1 notation (e.g., "A1", "C10")
 * @returns Object with 0-based row and column indices, or null if invalid
 */
export function parseCellRef(cellRef: string): { row: number; col: number } | null {
  const match = cellRef.match(/^([A-Z]+)(\d+)$/);
  if (!match) {return null;}

  const colStr = match[1]!;
  const rowStr = match[2]!;

  const col = letterToColumn(colStr);
  const row = parseInt(rowStr, 10) - 1; // Convert to 0-based

  return { row, col };
}

/**
 * Range bounds with 0-based coordinates.
 */
export interface RangeBounds {
  /** 0-based starting column index */
  startCol: number;
  /** 0-based starting row index */
  startRow: number;
  /** 0-based ending column index */
  endCol: number;
  /** 0-based ending row index */
  endRow: number;
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
  // Handle single cell
  if (!rangeRef.includes(':')) {
    const coords = parseCellRef(rangeRef);
    if (!coords) {return null;}
    
    return {
      startCol: coords.col,
      startRow: coords.row,
      endCol: coords.col,
      endRow: coords.row
    };
  }

  // Handle range
  const [startCell, endCell] = rangeRef.split(':');
  if (!startCell || !endCell) {return null;}

  const startCoords = parseCellRef(startCell);
  const endCoords = parseCellRef(endCell);

  if (!startCoords || !endCoords) {return null;}

  return {
    startCol: startCoords.col,
    startRow: startCoords.row,
    endCol: endCoords.col,
    endRow: endCoords.row
  };
}

/**
 * Convert 0-based bounds to A1 notation starting cell (1-based).
 * 
 * @example
 * boundsToA1StartCell({ startCol: 0, startRow: 0, ... }) // Returns "A1"
 * boundsToA1StartCell({ startCol: 2, startRow: 5, ... }) // Returns "C6"
 * 
 * @param bounds - Range bounds with 0-based coordinates
 * @returns A1 notation cell reference (1-based)
 */
export function boundsToA1StartCell(bounds: RangeBounds): string {
  return `${columnToLetter(bounds.startCol)}${bounds.startRow + 1}`;
}

/**
 * Convert 0-based bounds to full A1 notation range (1-based).
 * 
 * @example
 * boundsToA1Range({ startCol: 0, startRow: 0, endCol: 2, endRow: 9 }) 
 * // Returns "A1:C10"
 * 
 * @param bounds - Range bounds with 0-based coordinates
 * @returns Full A1 notation range (1-based)
 */
export function boundsToA1Range(bounds: RangeBounds): string {
  const startCell = boundsToA1StartCell(bounds);
  
  // If single cell, return just the cell
  if (bounds.startCol === bounds.endCol && bounds.startRow === bounds.endRow) {
    return startCell;
  }
  
  const endCell = `${columnToLetter(bounds.endCol)}${bounds.endRow + 1}`;
  return `${startCell}:${endCell}`;
}

/**
 * Check if two ranges intersect/overlap.
 * 
 * @param range1 - First range bounds
 * @param range2 - Second range bounds
 * @returns True if ranges overlap, false otherwise
 */
export function rangesIntersect(range1: RangeBounds, range2: RangeBounds): boolean {
  return !(
    range1.endCol < range2.startCol ||
    range2.endCol < range1.startCol ||
    range1.endRow < range2.startRow ||
    range2.endRow < range1.startRow
  );
}

/**
 * Get the number of rows in a range.
 * 
 * @param bounds - Range bounds
 * @returns Number of rows
 */
export function getRangeRowCount(bounds: RangeBounds): number {
  return bounds.endRow - bounds.startRow + 1;
}

/**
 * Get the number of columns in a range.
 * 
 * @param bounds - Range bounds
 * @returns Number of columns
 */
export function getRangeColumnCount(bounds: RangeBounds): number {
  return bounds.endCol - bounds.startCol + 1;
}
