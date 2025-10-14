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
  | 'xlsx'          // Excel 2007+ format (future)
  | 'anafispread';  // AnaFis spreadsheet format (future)

/**
 * Export range mode
 */
export type ExportRangeMode = 
  | 'selection'     // Current spreadsheet selection
  | 'sheet'         // Active sheet only
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
 * Options for configuring exports
 */
export interface ExportOptions {
  // General options
  includeHeaders?: boolean;        // Include header row (default: true)
  includeFormulas?: boolean;       // Include formulas vs evaluated values (default: false)
  includeFormatting?: boolean;     // Include formatting metadata (default: false)
  includeMetadata?: boolean;       // Include metadata (default: false)
  includeUncertainties?: boolean;  // Include uncertainties (default: false)
  
  // Text format options (CSV, TSV, TXT)
  delimiter?: string;              // Delimiter character (default: ',' for CSV, '\t' for TSV)
  encoding?: 'utf8' | 'latin1' | 'utf16';  // Character encoding (default: utf8)
  lineEnding?: 'lf' | 'crlf';      // Line ending style (default: crlf)
  quoteChar?: string;              // Quote character (default: '"')
  
  // JSON options
  jsonFormat?: JsonFormat;         // JSON format type (default: 'records')
  prettyPrint?: boolean;           // Pretty print JSON (default: true)
  
  // Compression options
  compress?: boolean;              // Compress output file (gzip) (default: false)
}

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
  includeHeaders: boolean;
  setIncludeHeaders: (include: boolean) => void;
  jsonFormat: JsonFormat;
  setJsonFormat: (format: JsonFormat) => void;
  prettyPrint: boolean;
  setPrettyPrint: (pretty: boolean) => void;
  customDelimiter: string;
  setCustomDelimiter: (delimiter: string) => void;
  
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
