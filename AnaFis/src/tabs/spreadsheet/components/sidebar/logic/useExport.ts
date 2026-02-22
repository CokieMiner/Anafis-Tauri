/**
 * useExport hook - Extracted business logic for export operations
 *
 * This hook encapsulates all the business logic for exporting spreadsheet data,
 * including state management, validation, and API calls.
 *
 * Supports both internal state (useState) and external state (from SidebarStateManager).
 */

import { useCallback, useState } from 'react';
import type {
  ExportFormat as CoreExportFormat,
  ExportRangeMode as CoreExportRangeMode,
  ExportOptions,
  ExportService,
} from '@/core/types/export';
import type {
  ExportFormat,
  ExportMode,
  ExportRangeMode,
  ExportSidebarState,
} from '@/tabs/spreadsheet/managers/SidebarStateManager';
import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';

interface UseExportOptions {
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  exportService: ExportService;
  onSelectionChange?: (selection: string) => void;
  // Optional external state (from SidebarStateManager)
  externalState?: ExportSidebarState | undefined;
  externalActions?:
    | {
        setFormat: (format: ExportFormat) => void;
        setRangeMode: (rangeMode: ExportRangeMode) => void;
        setCustomRange: (customRange: string) => void;
        setCustomDelimiter: (customDelimiter: string) => void;
        setMode: (mode: ExportMode) => void;
        setLibraryName: (libraryName: string) => void;
        setLibraryDescription: (libraryDescription: string) => void;
        setLibraryTags: (libraryTags: string) => void;
        setLibraryUnit: (libraryUnit: string) => void;
        setDataRange: (dataRange: string) => void;
        setUncertaintyRange: (uncertaintyRange: string) => void;
      }
    | undefined;
}

export function useExport({
  spreadsheetRef,
  exportService,
  onSelectionChange,
  externalState,
  externalActions,
}: UseExportOptions) {
  // Internal state (used when external state is not provided)
  const [internalExportFormat, setInternalExportFormat] =
    useState<ExportFormat>('anafispread');
  const [internalRangeMode, setInternalRangeMode] =
    useState<ExportRangeMode>('sheet');
  const [internalCustomRange, setInternalCustomRange] = useState<string>('');
  const [internalCustomDelimiter, setInternalCustomDelimiter] =
    useState<string>('|');
  const [internalExportMode, setInternalExportMode] =
    useState<ExportMode>('file');
  const [internalLibraryName, setInternalLibraryName] = useState('');
  const [internalLibraryDescription, setInternalLibraryDescription] =
    useState('');
  const [internalLibraryTags, setInternalLibraryTags] = useState('');
  const [internalLibraryUnit, setInternalLibraryUnit] = useState('');
  const [internalDataRange, setInternalDataRange] = useState('A:A');
  const [internalUncertaintyRange, setInternalUncertaintyRange] = useState('');

  // Use external state if provided, otherwise use internal state
  const exportFormat = externalState?.format ?? internalExportFormat;
  const rangeMode = externalState?.rangeMode ?? internalRangeMode;
  const customRange = externalState?.customRange ?? internalCustomRange;
  const customDelimiter =
    externalState?.customDelimiter ?? internalCustomDelimiter;
  const exportMode = externalState?.mode ?? internalExportMode;
  const libraryName = externalState?.libraryName ?? internalLibraryName;
  const libraryDescription =
    externalState?.libraryDescription ?? internalLibraryDescription;
  const libraryTags = externalState?.libraryTags ?? internalLibraryTags;
  const libraryUnit = externalState?.libraryUnit ?? internalLibraryUnit;
  const dataRange = externalState?.dataRange ?? internalDataRange;
  const uncertaintyRange =
    externalState?.uncertaintyRange ?? internalUncertaintyRange;

  // Determine which setters to use
  const setExportFormat = externalActions?.setFormat ?? setInternalExportFormat;
  const setRangeMode = externalActions?.setRangeMode ?? setInternalRangeMode;
  const setCustomRange =
    externalActions?.setCustomRange ?? setInternalCustomRange;
  const setCustomDelimiter =
    externalActions?.setCustomDelimiter ?? setInternalCustomDelimiter;
  const setExportMode = externalActions?.setMode ?? setInternalExportMode;
  const setLibraryName =
    externalActions?.setLibraryName ?? setInternalLibraryName;
  const setLibraryDescription =
    externalActions?.setLibraryDescription ?? setInternalLibraryDescription;
  const setLibraryTags =
    externalActions?.setLibraryTags ?? setInternalLibraryTags;
  const setLibraryUnit =
    externalActions?.setLibraryUnit ?? setInternalLibraryUnit;
  const setDataRange = externalActions?.setDataRange ?? setInternalDataRange;
  const setUncertaintyRange =
    externalActions?.setUncertaintyRange ?? setInternalUncertaintyRange;

  // Processing state (always internal - transient UI state)
  const [isExporting, setIsExporting] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Handle export with dialog
  const handleExport = useCallback(async (): Promise<void> => {
    setError(null);
    setSuccess(null);
    setIsExporting(true);

    try {
      const options: ExportOptions = {
        format: exportFormat as CoreExportFormat,
        rangeMode: rangeMode as CoreExportRangeMode,
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

      const result = await exportService.exportWithDialog(
        options,
        spreadsheetAPI
      );

      if (result.ok) {
        setSuccess(result.value.message ?? 'Export completed successfully');
      } else {
        setError(result.error.message);
      }
    } catch (err) {
      setError(
        `Export failed: ${err instanceof Error ? err.message : 'Unknown error'}`
      );
    } finally {
      setIsExporting(false);
    }
  }, [
    exportFormat,
    rangeMode,
    customRange,
    customDelimiter,
    exportService,
    spreadsheetRef,
  ]);

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

      const result = await exportService.exportToDataLibrary(
        {
          libraryName,
          libraryDescription,
          libraryTags,
          libraryUnit,
          dataRange,
          uncertaintyRange,
        },
        spreadsheetAPI
      );

      if (result.ok) {
        setSuccess(
          result.value.message ?? 'Successfully saved to Data Library'
        );
      } else {
        setError(result.error.message);
      }
    } catch (err) {
      setError(
        `Failed to save to Data Library: ${err instanceof Error ? err.message : String(err)}`
      );
    } finally {
      setIsExporting(false);
    }
  }, [
    libraryName,
    libraryDescription,
    libraryTags,
    libraryUnit,
    dataRange,
    uncertaintyRange,
    exportService,
    spreadsheetRef,
  ]);

  // Clear export result
  const clearResult = useCallback(() => {
    setError(null);
    setSuccess(null);
  }, []);

  // Handle selection change for custom range
  const handleSelectionChange = useCallback(
    (selection: string) => {
      if (rangeMode === 'custom') {
        setCustomRange(selection);
      }
      onSelectionChange?.(selection);
    },
    [rangeMode, onSelectionChange, setCustomRange]
  );

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
