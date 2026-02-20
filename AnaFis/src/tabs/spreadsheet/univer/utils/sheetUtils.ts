/**
 * Utility functions for spreadsheet operations
 */

/**
 * Safely retrieves a sheet name with fallback logic
 * @param sheet The sheet object to get name from
 * @param sheets Array of all sheets (for fallback naming)
 * @returns The sheet name or a fallback name
 */
export function getSheetNameSafely(
  sheet: { getSheetName: () => string },
  sheets: { getSheetName: () => string }[]
): string {
  try {
    // Use official Facade API to get sheet name
    return sheet.getSheetName();
  } catch {
    // Fallback to indexed name if API call fails
    return `Sheet ${sheets.indexOf(sheet) + 1}`;
  }
}
