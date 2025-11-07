// Range and cell utilities for spreadsheet operations
// AnaFis - Unit Conversion Sidebar

// Cached regex patterns for better performance
const RANGE_REGEX = /^([A-Z]+)(\d+):([A-Z]+)(\d+)$/;
const CELL_REGEX = /^[A-Z]+\d+$/;

/**
 * Convert column letter to number (A=1, B=2, ..., Z=26, AA=27, etc.)
 */
export const colToNum = (col: string): number => {
  let result = 0;
  for (let i = 0; i < col.length; i++) {
    result = result * 26 + (col.charCodeAt(i) - 65 + 1);
  }
  return result;
};

/**
 * Convert column number to letter (1=A, 2=B, ..., 26=Z, 27=AA, etc.)
 */
export const numToCol = (num: number): string => {
  let colStr = '';
  let temp = num;
  while (temp > 0) {
    temp--;
    colStr = String.fromCharCode(65 + (temp % 26)) + colStr;
    temp = Math.floor(temp / 26);
  }
  return colStr;
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
  const startRow = parseInt(match[2]);
  const endCol = match[3];
  const endRow = parseInt(match[4]);

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
export const getRangeFormat = (rangeRef: string): { rows: number; cols: number } => {
  const match = rangeRef.match(RANGE_REGEX);
  if (!match?.[1] || !match[2] || !match[3] || !match[4]) {
    throw new Error(`Invalid range format: ${rangeRef}`);
  }

  const startCol = match[1];
  const startRow = parseInt(match[2]);
  const endCol = match[3];
  const endRow = parseInt(match[4]);

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
  const startRow1 = parseInt(match1[2]);
  const endCol1 = colToNum(match1[3]);
  const endRow1 = parseInt(match1[4]);

  // Parse range2
  const match2 = range2.match(RANGE_REGEX);
  if (!match2?.[1] || !match2[2] || !match2[3] || !match2[4]) {
    throw new Error(`Invalid range format: ${range2}`);
  }
  const startCol2 = colToNum(match2[1]);
  const startRow2 = parseInt(match2[2]);
  const endCol2 = colToNum(match2[3]);
  const endRow2 = parseInt(match2[4]);

  // Check for overlap using bounding box intersection
  // Two rectangles overlap if they overlap on both x and y axes
  return !(endCol1 < startCol2 || endCol2 < startCol1 || endRow1 < startRow2 || endRow2 < startRow1);
};

/**
 * Determine input type from a string value
 */
export const getInputType = (input: string): 'number' | 'cell' | 'range' | 'empty' => {
  if (!input) { return 'empty'; }
  if (input.includes(':')) { return 'range'; }
  if (CELL_REGEX.test(input)) { return 'cell'; }

  // Strict numeric validation: trim input and test against numeric pattern
  const trimmed = input.trim();
  const numericRegex = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$/;
  if (numericRegex.test(trimmed)) {
    // Additional check: ensure the entire string is consumed by Number parsing
    const numValue = Number(trimmed);
    if (!isNaN(numValue) && isFinite(numValue)) {
      return 'number';
    }
  }

  throw new Error(`Invalid input format: ${input}`);
};