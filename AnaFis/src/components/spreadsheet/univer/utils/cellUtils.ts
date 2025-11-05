/**
 * Utility functions for cell reference parsing and conversion
 */

import { EXCEL_ALPHABET_SIZE, ASCII_UPPERCASE_A, EXCEL_COLUMN_OFFSET, ERROR_MESSAGES } from './constants';

/**
 * Converts a column number to letter notation (0 = A, 1 = B, etc.)
 * @param col Zero-based column index
 * @returns Column letter(s) like 'A', 'B', 'AA', etc.
 */
export function columnToLetter(col: number): string {
  let temp, letter = '';
  let column = col;
  while (column >= 0) {
    temp = column % EXCEL_ALPHABET_SIZE;
    letter = String.fromCharCode(temp + ASCII_UPPERCASE_A) + letter;
    column = Math.floor(column / EXCEL_ALPHABET_SIZE) - 1;
  }
  return letter;
}

/**
 * Parses A1 notation and returns 0-indexed row/col coordinates
 * @param cellRef Cell reference like 'A1', 'B2', 'AA10'
 * @returns Object with row and col properties, or null if invalid
 */
export function parseCellRef(cellRef: string): { row: number; col: number } | null {
  const match = cellRef.match(/^([A-Z]+)(\d+)$/);
  if (!match) { return null; }

  const colStr = match[1]!;
  const rowStr = match[2]!;

  // Convert column letters to number (A=0, B=1, etc.)
  let col = 0;
  for (let i = 0; i < colStr.length; i++) {
    col = col * EXCEL_ALPHABET_SIZE + (colStr.charCodeAt(i) - ASCII_UPPERCASE_A + EXCEL_COLUMN_OFFSET);
  }
  col -= EXCEL_COLUMN_OFFSET; // Convert to 0-indexed

  const row = parseInt(rowStr, 10) - EXCEL_COLUMN_OFFSET; // Convert to 0-indexed

  return { row, col };
}

/**
 * Converts a column letter to number (A = 0, B = 1, etc.)
 * @param letter Column letter(s) like 'A', 'B', 'AA', etc.
 * @returns Zero-based column index
 */
export function letterToColumn(letter: string): number {
  if (!letter || typeof letter !== 'string') {
    throw new TypeError(ERROR_MESSAGES.INPUT_MUST_BE_NON_EMPTY_STRING);
  }

  const normalizedLetter = letter.toUpperCase();
  if (!/^[A-Z]+$/.test(normalizedLetter)) {
    throw new TypeError(ERROR_MESSAGES.INVALID_COLUMN_LETTER_FORMAT(letter));
  }

  let column = 0;
  for (let i = 0; i < normalizedLetter.length; i++) {
    column = column * EXCEL_ALPHABET_SIZE + (normalizedLetter.charCodeAt(i) - ASCII_UPPERCASE_A + EXCEL_COLUMN_OFFSET);
  }
  return column - EXCEL_COLUMN_OFFSET;
}

/**
 * Parses range or single cell reference and returns all coordinates
 * @param cellRef Cell reference like 'A1' or range like 'A1:C3'
 * @returns Array of coordinate objects for bounds calculation
 */
export function parseRangeOrCell(cellRef: string): { row: number; col: number }[] {
  if (cellRef.includes(':')) {
    // Handle range like "A1:C3"
    const parts = cellRef.split(':');
    const startRef = parts[0];
    const endRef = parts[1];

    if (!startRef || !endRef) {
      return [];
    }

    const startCoords = parseCellRef(startRef);
    const endCoords = parseCellRef(endRef);

    if (!startCoords || !endCoords) {
      return [];
    }

    // Return both start and end coordinates for bounds calculation
    return [startCoords, endCoords];
  } else {
    // Handle single cell like "A1"
    const coords = parseCellRef(cellRef);
    return coords ? [coords] : [];
  }
}