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
  return rangeRef.includes(':')
    ? (rangeRef.split(':')[0] ?? rangeRef)
    : rangeRef;
}
