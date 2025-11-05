// Export types - streamlined for AnaFis workflow

import { SpreadsheetRef } from '../components/spreadsheet/SpreadsheetInterface';

/**
 * Export format types - streamlined hierarchy
 * 
 * PRIMARY (Lossless): anafispread - native format for full workbook preservation
 * SIMPLE INTERCHANGE: csv, tsv, txt, parquet - for external application interaction  
 * READ-ONLY DOCUMENTS: html, markdown, tex - for reports and documentation (no options, just custom delimiter for txt)
 */
export type ExportFormat = 
  | 'anafispread'  // Lossless native format (primary)
  | 'csv' | 'tsv' | 'txt' | 'parquet'  // Simple interchange formats
  | 'html' | 'markdown' | 'tex';  // Read-only document formats

/**
 * Export range mode - simplified to two essential options
 */
export type ExportRangeMode = 'sheet' | 'custom';

/**
 * Streamlined export options - only essential configuration
 * 
 * AnaFisSpread: No options needed (always lossless)
 * Simple formats: Only custom delimiter for TXT, encoding for supported formats
 * Range: Current sheet OR custom user-selected range
 */
export interface ExportOptions {
  format: ExportFormat;
  rangeMode: ExportRangeMode;
  customRange?: string;  // Required only when rangeMode === 'custom'
  delimiter?: string;    // Only for txt format (csv=comma, tsv=tab fixed)
  encoding?: 'utf8' | 'latin1' | 'utf16';  // For supporting formats
  trackedBounds?: Record<string, { maxRow: number; maxCol: number }> | null;  // Performance optimization
}

/**
 * Export result
 */
export interface ExportResult {
  success: boolean;
  message?: string;
  error?: string;
}

/**
 * Streamlined export sidebar props - simplified state management
 */
export interface ExportSidebarProps {
  open: boolean;
  onClose: () => void;
  spreadsheetRef?: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  
  // Dependency injection for export service (abstraction)
  exportService: IExportService;
  
  // Simplified state management - only essential options
  exportFormat: ExportFormat;
  setExportFormat: (format: ExportFormat) => void;
  rangeMode: ExportRangeMode;
  setRangeMode: (mode: ExportRangeMode) => void;
  customRange: string;
  setCustomRange: (range: string) => void;
  customDelimiter: string;
  setCustomDelimiter: (delimiter: string) => void;
}

/**
 * Export service interface
 */
export interface IExportService {
  exportWithDialog(options: ExportOptions, spreadsheetAPI: SpreadsheetRef): Promise<ExportResult>;
  
  exportToDataLibrary(options: {
    libraryName: string;
    libraryDescription: string;
    libraryTags: string;
    libraryUnit: string;
    dataRange: string;
    uncertaintyRange: string;
  }, spreadsheetAPI: SpreadsheetRef): Promise<ExportResult>;
}
