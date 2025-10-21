// univerUtils.ts - Core utility functions for Univer operations
import { IRange } from '@univerjs/core';

export function columnToLetter(column: number): string {
  let temp, letter = '';
  while (column >= 0) {
    temp = column % 26;
    letter = String.fromCharCode(temp + 65) + letter;
    column = Math.floor(column / 26) - 1;
  }
  return letter;
}

export function letterToColumn(letter: string): number {
  let column = 0;
  for (let i = 0; i < letter.length; i++) {
    column = column * 26 + (letter.charCodeAt(i) - 65 + 1);
  }
  return column - 1;
}

export function rangeToA1(range: IRange): string {
  const startCol = columnToLetter(range.startColumn);
  const startRow = range.startRow + 1;
  const endCol = columnToLetter(range.endColumn);
  const endRow = range.endRow + 1;
  
  // If it's a single cell, return just that cell
  if (range.startColumn === range.endColumn && range.startRow === range.endRow) {
    return `${startCol}${startRow}`;
  }
  
  // Otherwise return full range notation
  return `${startCol}${startRow}:${endCol}${endRow}`;
}

export function cellRefToIndices(cellRef: string): { row: number; col: number } | null {
  const match = cellRef.match(/^([A-Z]+)(\d+)$/);
  if (!match) return null;

  const col = match[1];
  const row = parseInt(match[2]) - 1;
  const colIndex = letterToColumn(col);

  return { row, col: colIndex };
}

export function parseRange(rangeRef: string): { startCol: number; startRow: number; endCol: number; endRow: number } | null {
  // Parse range: A1:B10 or A1 (single cell)
  const rangeMatch = rangeRef.match(/^([A-Z]+)(\d+):([A-Z]+)(\d+)$/);
  const singleMatch = rangeRef.match(/^([A-Z]+)(\d+)$/);

  if (rangeMatch) {
    const [, startCol, startRow, endCol, endRow] = rangeMatch.map((v, i) =>
      i === 2 || i === 4 ? parseInt(v) - 1 : v
    ) as [string, string, number, string, number];
    return {
      startCol: letterToColumn(startCol),
      startRow,
      endCol: letterToColumn(endCol),
      endRow
    };
  } else if (singleMatch) {
    const col = singleMatch[1];
    const row = parseInt(singleMatch[2]) - 1;
    const colIndex = letterToColumn(col);
    return {
      startCol: colIndex,
      startRow: row,
      endCol: colIndex,
      endRow: row
    };
  }

  return null;
}
