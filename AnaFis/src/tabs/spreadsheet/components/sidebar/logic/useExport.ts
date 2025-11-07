/**
 * useExport hook - Extracted business logic for export operations
 *
 * This hook encapsulates all the business logic for exporting spreadsheet data,
 * including state management, validation, and API calls.
 */

import { useState, useCallback } from 'react';
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { ExportFormat, ExportRangeMode, ExportService, ExportOptions } from '@/core/types/export';

interface UseExportOptions {
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  exportService: ExportService;
  onSelectionChange?: (selection: string) => void;
}

export function useExport({
  spreadsheetRef,
  exportService,
  onSelectionChange,
}: UseExportOptions) {
  // Export state
  const [exportFormat, setExportFormat] = useState<ExportFormat>('anafispread');
  const [rangeMode, setRangeMode] = useState<ExportRangeMode>('sheet');
  const [customRange, setCustomRange] = useState<string>('');
  const [customDelimiter, setCustomDelimiter] = useState<string>('|');
  const [isExporting, setIsExporting] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Export mode: 'file' or 'library'
  const [exportMode, setExportMode] = useState<'file' | 'library'>('file');

  // Data Library export state
  const [libraryName, setLibraryName] = useState('');
  const [libraryDescription, setLibraryDescription] = useState('');
  const [libraryTags, setLibraryTags] = useState('');
  const [libraryUnit, setLibraryUnit] = useState('');
  const [dataRange, setDataRange] = useState('A:A');
  const [uncertaintyRange, setUncertaintyRange] = useState('');

  // Handle export with dialog
  const handleExport = useCallback(async (): Promise<void> => {
    setError(null);
    setSuccess(null);
    setIsExporting(true);

    try {
      const options: ExportOptions = {
        format: exportFormat,
        rangeMode,
        ...(rangeMode === 'custom' && { customRange }),
        ...(exportFormat === 'txt' && { delimiter: customDelimiter }),
        encoding: 'utf8' as const,
      };

      const spreadsheetAPI = spreadsheetRef.current;
      if (!spreadsheetAPI) {
        setError('Spreadsheet API not available');
        setIsExporting(false);
        return;
      }

      const result = await exportService.exportWithDialog(options, spreadsheetAPI);

      if (result.ok) {
        setSuccess(result.value.message ?? 'Export completed successfully');
      } else {
        setError(result.error.message);
      }
    } catch (err) {
      setError(`Export failed: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setIsExporting(false);
    }
  }, [exportFormat, rangeMode, customRange, customDelimiter, exportService, spreadsheetRef]);

  // Handle export to data library
  const handleExportToLibrary = useCallback(async (): Promise<void> => {
    setError(null);
    setSuccess(null);
    setIsExporting(true);

    try {
      const spreadsheetAPI = spreadsheetRef.current;
      if (!spreadsheetAPI) {
        setError('Spreadsheet API not available');
        setIsExporting(false);
        return;
      }

      const result = await exportService.exportToDataLibrary({
        libraryName,
        libraryDescription,
        libraryTags,
        libraryUnit,
        dataRange,
        uncertaintyRange,
      }, spreadsheetAPI);

      if (result.ok) {
        setSuccess(result.value.message ?? 'Successfully saved to Data Library');
      } else {
        setError(result.error.message);
      }
    } catch (err) {
      setError(`Failed to save to Data Library: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsExporting(false);
    }
  }, [libraryName, libraryDescription, libraryTags, libraryUnit, dataRange, uncertaintyRange, exportService, spreadsheetRef]);

  // Clear export result
  const clearResult = useCallback(() => {
    setError(null);
    setSuccess(null);
  }, []);

  // Handle selection change for custom range
  const handleSelectionChange = useCallback((selection: string) => {
    if (rangeMode === 'custom') {
      setCustomRange(selection);
    }
    onSelectionChange?.(selection);
  }, [rangeMode, onSelectionChange]);

  return {
    // State
    exportFormat,
    setExportFormat,
    rangeMode,
    setRangeMode,
    customRange,
    setCustomRange,
    customDelimiter,
    setCustomDelimiter,
    isExporting,
    error,
    success,
    exportMode,
    setExportMode,
    libraryName,
    setLibraryName,
    libraryDescription,
    setLibraryDescription,
    libraryTags,
    setLibraryTags,
    libraryUnit,
    setLibraryUnit,
    dataRange,
    setDataRange,
    uncertaintyRange,
    setUncertaintyRange,

    // Actions
    handleExport,
    handleExportToLibrary,
    clearResult,
    handleSelectionChange,
  };
}