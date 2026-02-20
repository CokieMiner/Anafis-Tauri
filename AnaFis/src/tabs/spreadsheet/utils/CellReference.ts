/**
 * CellReference - Unified utility class for cell reference conversions and parsing
 *
 * This class provides a single source of truth for all cell reference operations,
 * ensuring consistency across the entire spreadsheet codebase. It handles:
 * - Column letter ↔ number conversions (Excel-style: A=0, B=1, ..., Z=25, AA=26)
 * - Cell reference parsing (A1 notation)
 * - Range parsing and formatting
 * - Bounds calculations
 *
 * All operations use 0-based indexing internally for consistency with arrays and most
 * programming contexts, while supporting conversion to/from 1-based systems as needed.
 *
 * Architecture Note: This is a generic utility that works independently of any
 * specific spreadsheet library (Univer, AG Grid, etc.). Library-specific extensions
 * should be handled in separate adapter utilities.
 */

export interface CellCoordinates {
  /** 0-based row index */
  row: number;
  /** 0-based column index */
  col: number;
}

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

export interface ParsedRange {
  /** Start cell coordinates */
  start: CellCoordinates;
  /** End cell coordinates */
  end: CellCoordinates;
  /** Pre-computed bounds for performance */
  bounds: RangeBounds;
}

// biome-ignore lint/complexity/noStaticOnlyClass: Utility namespace for cell references
export class CellReference {
  // Excel-style constants
  private static readonly ALPHABET_SIZE = 26;
  private static readonly ASCII_UPPERCASE_A = 65;

  // Common validation patterns
  private static readonly CELL_PATTERN = /^[A-Z]+[1-9]\d*$/;
  private static readonly COLUMN_PATTERN = /^[A-Z]+$/;
  private static readonly RANGE_PATTERN = /^([A-Z]+[1-9]\d*):([A-Z]+[1-9]\d*)$/;

  /**
   * Converts a 0-based column index to Excel column letter notation
   *
   * @param columnIndex - 0-based column index (0 = A, 1 = B, 25 = Z, 26 = AA)
   * @returns Column letter string
   * @throws Error if columnIndex is negative
   *
   * @example
   * CellReference.columnToLetter(0) // "A"
   * CellReference.columnToLetter(25) // "Z"
   * CellReference.columnToLetter(26) // "AA"
   */
  static columnToLetter(columnIndex: number): string {
    if (columnIndex < 0) {
      throw new Error(`Column index cannot be negative: ${columnIndex}`);
    }

    if (columnIndex === 0) {
      return 'A';
    }

    let result = '';
    let index = columnIndex;

    while (index >= 0) {
      const remainder = index % CellReference.ALPHABET_SIZE;
      result =
        String.fromCharCode(remainder + CellReference.ASCII_UPPERCASE_A) +
        result;
      index = Math.floor(index / CellReference.ALPHABET_SIZE) - 1;
    }

    return result;
  }

  /**
   * Converts Excel column letter notation to 0-based column index
   *
   * @param columnLetter - Column letter string (case-insensitive)
   * @returns 0-based column index
   * @throws Error if columnLetter is invalid
   *
   * @example
   * CellReference.letterToColumn("A") // 0
   * CellReference.letterToColumn("Z") // 25
   * CellReference.letterToColumn("AA") // 26
   */
  static letterToColumn(columnLetter: string): number {
    if (!columnLetter || typeof columnLetter !== 'string') {
      throw new Error('Column letter must be a non-empty string');
    }

    const normalized = columnLetter.toUpperCase();
    if (!CellReference.COLUMN_PATTERN.test(normalized)) {
      throw new Error(`Invalid column letter format: ${columnLetter}`);
    }

    let result = 0;
    for (let i = 0; i < normalized.length; i++) {
      result =
        result * CellReference.ALPHABET_SIZE +
        (normalized.charCodeAt(i) - CellReference.ASCII_UPPERCASE_A + 1);
    }

    return result - 1; // Convert to 0-based
  }

  /**
   * Parses A1 notation cell reference to 0-based coordinates
   *
   * @param cellRef - Cell reference in A1 notation (e.g., "A1", "B10", "AA100")
   * @returns Cell coordinates object
   * @throws Error if cellRef is invalid
   *
   * @example
   * CellReference.parseCell("A1") // { row: 0, col: 0 }
   * CellReference.parseCell("B2") // { row: 1, col: 1 }
   * CellReference.parseCell("AA10") // { row: 9, col: 26 }
   */
  static parseCell(cellRef: string): CellCoordinates {
    if (!cellRef || typeof cellRef !== 'string') {
      throw new Error('Cell reference must be a non-empty string');
    }

    const match = cellRef.match(/^([A-Z]+)([1-9]\d*)$/);
    if (!match) {
      throw new Error(
        `Invalid cell reference format: ${cellRef}. Expected A1 notation like "A1" or "B10".`
      );
    }

    const colStr = match[1] ?? 'A';
    const rowStr = match[2] ?? '1';
    const col = CellReference.letterToColumn(colStr);
    const row = parseInt(rowStr, 10) - 1; // Convert to 0-based

    if (row < 0) {
      throw new Error(`Invalid row number in cell reference: ${cellRef}`);
    }

    return { row, col };
  }

  /**
   * Parses A1 notation range reference to structured bounds
   *
   * @param rangeRef - Range reference in A1 notation (e.g., "A1:B2", "C5:C5")
   * @returns Parsed range with coordinates and bounds
   * @throws Error if rangeRef is invalid
   *
   * @example
   * CellReference.parseRange("A1:B2") // { start: {row:0,col:0}, end: {row:1,col:1}, bounds: {...} }
   * CellReference.parseRange("C5") // Single cell range
   */
  static parseRange(rangeRef: string): ParsedRange {
    if (!rangeRef || typeof rangeRef !== 'string') {
      throw new Error('Range reference must be a non-empty string');
    }

    // Handle single cell (no colon)
    if (!rangeRef.includes(':')) {
      const coords = CellReference.parseCell(rangeRef);
      return {
        start: coords,
        end: coords,
        bounds: {
          startCol: coords.col,
          startRow: coords.row,
          endCol: coords.col,
          endRow: coords.row,
        },
      };
    }

    // Handle range (with colon)
    const match = rangeRef.match(CellReference.RANGE_PATTERN);
    if (!match) {
      throw new Error(
        `Invalid range reference format: ${rangeRef}. Expected "A1:B2" or "C5:C5".`
      );
    }

    const startCell = match[1] ?? 'A1';
    const endCell = match[2] ?? 'A1';
    const start = CellReference.parseCell(startCell);
    const end = CellReference.parseCell(endCell);

    return {
      start,
      end,
      bounds: {
        startCol: Math.min(start.col, end.col),
        startRow: Math.min(start.row, end.row),
        endCol: Math.max(start.col, end.col),
        endRow: Math.max(start.row, end.row),
      },
    };
  }

  /**
   * Formats 0-based coordinates to A1 notation cell reference
   *
   * @param row - 0-based row index
   * @param col - 0-based column index
   * @returns Cell reference in A1 notation
   * @throws Error if coordinates are invalid
   *
   * @example
   * CellReference.formatCell(0, 0) // "A1"
   * CellReference.formatCell(9, 26) // "AA10"
   */
  static formatCell(row: number, col: number): string {
    if (!Number.isInteger(row) || row < 0) {
      throw new Error(`Row index must be a non-negative integer: ${row}`);
    }
    if (!Number.isInteger(col) || col < 0) {
      throw new Error(`Column index must be a non-negative integer: ${col}`);
    }

    const colLetter = CellReference.columnToLetter(col);
    const rowNumber = row + 1; // Convert to 1-based
    return `${colLetter}${rowNumber}`;
  }

  /**
   * Formats range bounds to A1 notation range reference
   *
   * @param bounds - Range bounds with 0-based coordinates
   * @returns Range reference in A1 notation
   * @throws Error if bounds are invalid
   *
   * @example
   * CellReference.formatRange({startCol:0,startRow:0,endCol:1,endRow:1}) // "A1:B2"
   */
  static formatRange(bounds: RangeBounds): string {
    const startCell = CellReference.formatCell(
      bounds.startRow,
      bounds.startCol
    );
    const endCell = CellReference.formatCell(bounds.endRow, bounds.endCol);
    return `${startCell}:${endCell}`;
  }

  /**
   * Checks if a cell reference is valid A1 notation
   *
   * @param cellRef - String to validate
   * @returns true if valid cell reference
   */
  static isValidCell(cellRef: string): boolean {
    return CellReference.CELL_PATTERN.test(cellRef);
  }

  /**
   * Checks if a range reference is valid A1 notation
   *
   * @param rangeRef - String to validate
   * @returns true if valid range reference
   */
  static isValidRange(rangeRef: string): boolean {
    if (!rangeRef.includes(':')) {
      return CellReference.isValidCell(rangeRef);
    }
    return CellReference.RANGE_PATTERN.test(rangeRef);
  }

  /**
   * Gets the bounds of a range as a simple object
   *
   * @param rangeRef - Range reference in A1 notation
   * @returns Range bounds
   */
  static getRangeBounds(rangeRef: string): RangeBounds {
    return CellReference.parseRange(rangeRef).bounds;
  }
}
