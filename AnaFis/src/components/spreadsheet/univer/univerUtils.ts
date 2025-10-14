// univerUtils.ts - Univer-specific utility functions
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
  let colIndex = 0;
  for (let i = 0; i < col.length; i++) {
    colIndex = colIndex * 26 + (col.charCodeAt(i) - 65 + 1);
  }
  colIndex -= 1;

  return { row, col: colIndex };
}