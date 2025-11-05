// Range validation utilities - implementation-agnostic
// These utilities validate spreadsheet range references without depending on any specific implementation

/**
 * Validates and normalizes a range reference (e.g., "A1:B10" or "Sheet1!A1:B10")
 * @param rangeRef - The range reference to validate
 * @returns The normalized range reference
 * @throws Error if the range reference is invalid
 */
export function normalizeRangeRef(rangeRef: string): string {
  if (!rangeRef || typeof rangeRef !== 'string') {
    throw new Error('Range reference must be a non-empty string');
  }

  const trimmed = rangeRef.trim();
  if (!trimmed) {
    throw new Error('Range reference cannot be empty');
  }

  // Basic validation for range format
  // Supports: A1, A1:B10, Sheet1!A1:B10
  const rangePattern = /^(?:([^!]+)!)?([A-Z]+\d+)(?::([A-Z]+\d+))?$/i;
  
  if (!rangePattern.test(trimmed)) {
    throw new Error(`Invalid range format: ${rangeRef}`);
  }

  return trimmed;
}

/**
 * Validates a cell reference (e.g., "A1" or "Sheet1!A1")
 * @param cellRef - The cell reference to validate
 * @returns The normalized cell reference
 * @throws Error if the cell reference is invalid
 */
export function normalizeCellRef(cellRef: string): string {
  if (!cellRef || typeof cellRef !== 'string') {
    throw new Error('Cell reference must be a non-empty string');
  }

  const trimmed = cellRef.trim();
  if (!trimmed) {
    throw new Error('Cell reference cannot be empty');
  }

  // Basic validation for cell format
  // Supports: A1, Sheet1!A1
  const cellPattern = /^(?:([^!]+)!)?([A-Z]+\d+)$/i;
  
  if (!cellPattern.test(trimmed)) {
    throw new Error(`Invalid cell format: ${cellRef}`);
  }

  return trimmed;
}

/**
 * Checks if a string looks like a valid range reference
 * @param rangeRef - The string to check
 * @returns true if the string looks like a valid range
 */
export function isValidRangeRef(rangeRef: string): boolean {
  try {
    normalizeRangeRef(rangeRef);
    return true;
  } catch {
    return false;
  }
}

/**
 * Checks if a string looks like a valid cell reference
 * @param cellRef - The string to check
 * @returns true if the string looks like a valid cell
 */
export function isValidCellRef(cellRef: string): boolean {
  try {
    normalizeCellRef(cellRef);
    return true;
  } catch {
    return false;
  }
}
