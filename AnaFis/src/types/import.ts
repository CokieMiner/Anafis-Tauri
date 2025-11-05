// Import types - streamlined for AnaFis workflow

import { SpreadsheetRef } from '../components/spreadsheet/SpreadsheetInterface';

/**
 * Import format types - streamlined hierarchy
 * 
 * PRIMARY (Lossless): anafispread - native format for full workbook restoration
 * SIMPLE INTERCHANGE: csv, tsv, txt, parquet - for external data import
 * DATA LIBRARY: datalibrary - import saved sequences from data library
 */
export type ImportFormat = 
  | 'anafispread'  // Lossless native format (primary)
  | 'csv' | 'tsv' | 'txt' | 'parquet'  // Simple interchange formats
  | 'datalibrary'; // Import from data library

/**
 * Import target mode for simple formats - where to place the data
 */
export type ImportTargetMode = 
  | 'newSheet'      // Create new sheet (default for simple formats)
  | 'currentRange'; // Import to specified range in current sheet (A1 default)

/**
 * AnaFisSpread import mode - workbook level operations
 */
export type AnaFisImportMode = 
  | 'append'   // Append sheets to current workbook (default)
  | 'replace'; // Replace entire workbook

/**
 * File metadata for import preview
 */
export interface FileMetadata {
  path: string;
  size: number;
  extension: string;
  rowCount?: number;
  columnCount?: number;
}

/**
 * Streamlined import options - context-sensitive configuration
 * 
 * AnaFisSpread: Append sheets (default) OR replace workbook
 * Simple formats: Skip rows, custom delimiter (TXT only), encoding, target location
 * Defaults: New sheet (simple formats), A1 (top left cell), Append (AnaFisSpread)
 */
export interface ImportOptions {
  format: ImportFormat;
  
  // Simple format options (csv, tsv, txt, parquet)
  skipRows?: number;           // Number of rows to skip (default: 0)
  delimiter?: string;          // Custom delimiter for txt format only (csv=comma, tsv=tab fixed)
  encoding?: 'utf8' | 'latin1' | 'utf16';  // Text encoding (default: utf8)
  targetMode?: ImportTargetMode;  // Where to import (default: newSheet for simple formats)
  targetRange?: string;        // Required when targetMode === 'currentRange', no default for specified range
  
  // AnaFisSpread options (anafispread format)
  anaFisMode?: AnaFisImportMode;  // How to handle workbook (default: append)
}

/**
 * Import service interface
 */
export interface IImportService {
  selectFile(): Promise<{ filePath: string; detectedFormat: ImportFormat } | null>;
  importFile(filePath: string, options: ImportOptions, spreadsheetRef: React.RefObject<SpreadsheetRef | null>): Promise<ImportResult>;
  getFileMetadata(filePath: string, delimiter?: string): Promise<FileMetadata | null>;
  getSupportedFormats(): { format: ImportFormat; description: string; extensions: string[] }[];
}

/**
 * Streamlined import sidebar props - file-first workflow
 */
export interface ImportSidebarProps {
  open: boolean;
  onClose: () => void;
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  
  // Dependency injection for import service
  importService: IImportService;
}

/**
 * Import result
 */
export interface ImportResult {
  success: boolean;
  message?: string;
  sheetCount?: number;
  error?: string;
  
  // Range validation information
  fileDimensions?: {
    rows: number;
    columns: number;
  };
  rangeValidation?: {
    isValid: boolean;
    warnings: string[];
    willTruncate: boolean;
    selectedRange?: {
      rows: number;
      columns: number;
    };
  };
}

/**
 * Workbook data structure
 */
export interface WorkbookData {
  id: string;
  name: string;
  appVersion?: string;
  locale?: string;
  styles?: unknown;
  sheets: Record<string, SheetData>;
  sheetOrder?: string[];
}

export interface MergeData {
  startRow: number;
  startColumn: number;
  endRow: number;
  endColumn: number;
}

export interface SheetData {
  id: string;
  name: string;
  cellData?: Record<string, CellData>;
  rowCount?: number;
  columnCount?: number;
  mergeData?: MergeData[];
}

export interface CellData {
  v?: string | number | boolean | null;
  f?: string;
  s?: unknown;
  p?: unknown;
  t?: unknown;
}
