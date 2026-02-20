// Range and cell utilities for spreadsheet operations
// AnaFis - Unit Conversion Sidebar

import { CellReference } from '@/tabs/spreadsheet/utils/CellReference';

// Cached regex patterns for better performance
const RANGE_REGEX = /^([A-Z]+)(\d+):([A-Z]+)(\d+)$/;
const CELL_REGEX = /^[A-Z]+\d+$/;

/**
 * Convert column letter to number (A=0, B=1, ..., Z=25, AA=26, etc.)
 */
const colToNum = (col: string): number => {
  return CellReference.letterToColumn(col);
};

/**
 * Convert column number to letter (0=A, 1=B, ..., 25=Z, 26=AA, etc.)
 */
const numToCol = (num: number): string => {
  return CellReference.columnToLetter(num);
};

/**
 * Parse a range string (e.g., "A1:B10") into an array of cell references
 */
export const parseRange = (rangeRef: string): string[] => {
  const match = rangeRef.match(RANGE_REGEX);
  if (!match?.[1] || !match[2] || !match[3] || !match[4]) {
    throw new Error(`Invalid range format: ${rangeRef}`);
  }

  const startCol = match[1];
  const startRow = parseInt(match[2], 10);
  const endCol = match[3];
  const endRow = parseInt(match[4], 10);

  const startColNum = colToNum(startCol);
  const endColNum = colToNum(endCol);

  const cells: string[] = [];
  for (let row = startRow; row <= endRow; row++) {
    for (let colNum = startColNum; colNum <= endColNum; colNum++) {
      const colStr = numToCol(colNum);
      cells.push(`${colStr}${row}`);
    }
  }
  return cells;
};

/**
 * Get the dimensions (rows x cols) of a range
 */
export const getRangeFormat = (
  rangeRef: string
): { rows: number; cols: number } => {
  const match = rangeRef.match(RANGE_REGEX);
  if (!match?.[1] || !match[2] || !match[3] || !match[4]) {
    throw new Error(`Invalid range format: ${rangeRef}`);
  }

  const startCol = match[1];
  const startRow = parseInt(match[2], 10);
  const endCol = match[3];
  const endRow = parseInt(match[4], 10);

  const rows = endRow - startRow + 1;
  const cols = colToNum(endCol) - colToNum(startCol) + 1;

  return { rows, cols };
};

/**
 * Check if two ranges overlap
 */
export const rangesOverlap = (range1: string, range2: string): boolean => {
  // Parse range1
  const match1 = range1.match(RANGE_REGEX);
  if (!match1?.[1] || !match1[2] || !match1[3] || !match1[4]) {
    throw new Error(`Invalid range format: ${range1}`);
  }
  const startCol1 = colToNum(match1[1]);
  const startRow1 = parseInt(match1[2], 10);
  const endCol1 = colToNum(match1[3]);
  const endRow1 = parseInt(match1[4], 10);

  // Parse range2
  const match2 = range2.match(RANGE_REGEX);
  if (!match2?.[1] || !match2[2] || !match2[3] || !match2[4]) {
    throw new Error(`Invalid range format: ${range2}`);
  }
  const startCol2 = colToNum(match2[1]);
  const startRow2 = parseInt(match2[2], 10);
  const endCol2 = colToNum(match2[3]);
  const endRow2 = parseInt(match2[4], 10);

  // Check for overlap using bounding box intersection
  // Two rectangles overlap if they overlap on both x and y axes
  return !(
    endCol1 < startCol2 ||
    endCol2 < startCol1 ||
    endRow1 < startRow2 ||
    endRow2 < startRow1
  );
};

/**
 * Determine input type from a string value
 */
export const getInputType = (
  input: string
): 'number' | 'cell' | 'range' | 'empty' => {
  if (!input) {
    return 'empty';
  }
  if (input.includes(':')) {
    return 'range';
  }
  if (CELL_REGEX.test(input)) {
    return 'cell';
  }

  // Strict numeric validation: trim input and test against numeric pattern
  const trimmed = input.trim();
  const numericRegex = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$/;
  if (numericRegex.test(trimmed)) {
    // Additional check: ensure the entire string is consumed by Number parsing
    const numValue = Number(trimmed);
    if (!Number.isNaN(numValue) && Number.isFinite(numValue)) {
      return 'number';
    }
  }

  throw new Error(`Invalid input format: ${input}`);
};
