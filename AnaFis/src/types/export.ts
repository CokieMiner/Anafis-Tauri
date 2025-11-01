// Export types - matches Rust backend types

import { SpreadsheetRef } from '../components/spreadsheet/SpreadsheetInterface';

/**
 * Export format types supported by the application
 */
export type ExportFormat = 
  | 'csv'           // Comma-separated values
  | 'tsv'           // Tab-separated values
  | 'txt'           // Custom delimiter text
  | 'json'          // JSON format
  | 'xlsx'          // Excel 2007+ format
  | 'anafispread'   // AnaFis spreadsheet format
  | 'parquet'       // Apache Parquet
  | 'tex'           // LaTeX table
  | 'html'          // HTML table
  | 'markdown';     // Markdown table

/**
 * Export range mode
 */
export type ExportRangeMode = 
  | 'sheet'         // Current sheet only
  | 'all'           // All sheets
  | 'custom';       // Custom range (e.g., 'A1:D20')

/**
 * JSON export format options
 */
export type JsonFormat = 
  | 'array'         // Simple 2D array [[1, 2], [3, 4]]
  | 'object'        // Object with named columns {col1: [1, 3], col2: [2, 4]}
  | 'records';      // Array of objects [{col1: 1, col2: 2}, {col1: 3, col2: 4}]

/**
 * Options for configuring exports - using discriminated unions for type safety
 */
export interface ExportOptionsBase {
  // Basic export configuration
  exportFormat: ExportFormat;

  // General options (required - UI always provides these)
  includeHeaders: boolean;        // Include header row
  losslessExtraction: boolean;    // Use advanced data extraction
  includeFormulas: boolean;       // Include formulas vs evaluated values
  includeFormatting: boolean;     // Include formatting metadata
  includeMetadata: boolean;       // Include metadata

  // Text format options (CSV, TSV, TXT)
  delimiter?: string;              // Delimiter character (default: ',' for CSV, '\t' for TSV)
  encoding: 'utf8' | 'latin1' | 'utf16';  // Character encoding (restricted to supported types)
  lineEnding?: 'lf' | 'crlf' | 'cr';      // Line ending style (default: crlf)
  quoteChar?: string;              // Quote character (default: '"')

  // JSON options
  jsonFormat?: JsonFormat;         // JSON format type (default: 'records')
  prettyPrint?: boolean;           // Pretty print JSON (default: true)

  // Compression options
  compress?: boolean;              // Compress output file (gzip) (default: false)

  // Runtime tracking for efficient export
  trackedBounds: Record<string, { maxRow: number; maxCol: number }> | null;
}

// Discriminated union: customRange is required only when rangeMode === 'custom'
export interface ExportOptionsSheet extends ExportOptionsBase {
  rangeMode: 'sheet';
}

export interface ExportOptionsAll extends ExportOptionsBase {
  rangeMode: 'all';
}

export interface ExportOptionsCustom extends ExportOptionsBase {
  rangeMode: 'custom';
  customRange: string;  // Required only for custom range mode
}

export type ExportOptions = ExportOptionsSheet | ExportOptionsAll | ExportOptionsCustom;

/**
 * Export configuration passed to backend
 */
export interface ExportConfig {
  range: string;                   // Range specification
  format: ExportFormat;            // Export format
  options: ExportOptions;          // Format-specific options
}

/**
 * Props for ExportSidebar component - follows existing sidebar pattern
 */
export interface ExportSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef?: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  
  // Lifted state for persistence across sidebar switches
  exportFormat: ExportFormat;
  setExportFormat: (format: ExportFormat) => void;
  rangeMode: ExportRangeMode;
  setRangeMode: (mode: ExportRangeMode) => void;
  customRange: string;
  setCustomRange: (range: string) => void;
  
  // Export options (lifted for persistence)
  jsonFormat: JsonFormat;
  setJsonFormat: (format: JsonFormat) => void;
  prettyPrint: boolean;
  setPrettyPrint: (pretty: boolean) => void;
  customDelimiter: string;
  setCustomDelimiter: (delimiter: string) => void;

  // Runtime bounds tracking for efficient export (per-sheet bounds)
  getTrackedBounds?: () => Record<string, { maxRow: number; maxCol: number }> | null;
  
  // Data Library export state (optional - for future implementation)
  dataLibraryName?: string;
  setDataLibraryName?: (name: string) => void;
  dataLibraryXColumn?: string;
  setDataLibraryXColumn?: (column: string) => void;
  dataLibraryYColumn?: string;
  setDataLibraryYColumn?: (column: string) => void;
  dataLibraryUncertaintyColumn?: string;
  setDataLibraryUncertaintyColumn?: (column: string) => void;
}

/**
 * Export result from backend
 */
export interface ExportResult {
  success: boolean;
  message?: string;
  filePath?: string;
  error?: string;
}
