/**
 * useImport hook - Extracted business logic for import operations
 *
 * This hook encapsulates all the business logic for importing data into spreadsheets,
 * including state management, file selection, and API calls.
 *
 * Supports both internal state (useState) and external state (from SidebarStateManager).
 */

import { useCallback, useState } from 'react';
import type {
  FileMetadata,
  ImportOptions,
  ImportService,
} from '@/core/types/import';
import type {
  ImportMode,
  ImportSidebarState,
} from '@/tabs/spreadsheet/managers/SidebarStateManager';
import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';

interface UseImportOptions {
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  importService: ImportService;
  // Optional external state (from SidebarStateManager)
  externalState?: ImportSidebarState | undefined;
  externalActions?:
    | {
        setMode: (mode: ImportMode) => void;
        setTargetRange: (targetRange: string) => void;
        setLibraryDataRange: (libraryDataRange: string) => void;
        setLibraryUncertaintyRange: (libraryUncertaintyRange: string) => void;
      }
    | undefined;
}

export function useImport({
  spreadsheetRef,
  importService,
  externalState,
  externalActions,
}: UseImportOptions) {
  // Internal state (used when external state is not provided)
  const [internalImportMode, setInternalImportMode] =
    useState<ImportMode>('file');
  const [internalTargetRange, setInternalTargetRange] = useState<string>('A1');
  const [internalLibraryDataRange, setInternalLibraryDataRange] =
    useState<string>('A1');
  const [internalLibraryUncertaintyRange, setInternalLibraryUncertaintyRange] =
    useState<string>('B1');

  // Use external state if provided, otherwise use internal state
  const importMode = externalState?.mode ?? internalImportMode;
  const targetRange = externalState?.targetRange ?? internalTargetRange;
  const libraryDataRange =
    externalState?.libraryDataRange ?? internalLibraryDataRange;
  const libraryUncertaintyRange =
    externalState?.libraryUncertaintyRange ?? internalLibraryUncertaintyRange;

  // Determine which setters to use
  const setImportMode = externalActions?.setMode ?? setInternalImportMode;
  const setTargetRange =
    externalActions?.setTargetRange ?? setInternalTargetRange;
  const setLibraryDataRange =
    externalActions?.setLibraryDataRange ?? setInternalLibraryDataRange;
  const setLibraryUncertaintyRange =
    externalActions?.setLibraryUncertaintyRange ??
    setInternalLibraryUncertaintyRange;

  // Processing state (always internal - transient UI state)
  const [isImporting, setIsImporting] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<{
    filePath: string;
    detectedFormat: string;
  } | null>(null);
  const [fileMetadata, setFileMetadata] = useState<FileMetadata | null>(null);

  // Select file for import
  const selectFile = useCallback(async (): Promise<boolean> => {
    try {
      const result = await importService.selectFile();
      if (result) {
        setSelectedFile(result);
        // Get file metadata
        const metadata = await importService.getFileMetadata(result.filePath);
        setFileMetadata(metadata);
        return true;
      }
      return false;
    } catch (error) {
      setError(
        `File selection failed: ${error instanceof Error ? error.message : String(error)}`
      );
      return false;
    }
  }, [importService]);

  // Import the selected file
  const importFile = useCallback(
    async (options: ImportOptions): Promise<void> => {
      if (!selectedFile || !spreadsheetRef.current) {
        setError('No file selected or spreadsheet not initialized');
        return;
      }

      setError(null);
      setSuccess(null);
      setIsImporting(true);

      try {
        const result = await importService.importFile(
          selectedFile.filePath,
          options,
          spreadsheetRef
        );
        if (result.ok) {
          setSuccess(result.value.message ?? 'Import completed successfully');
          // Clear selection on success
          setSelectedFile(null);
          setFileMetadata(null);
        } else {
          setError(result.error.message);
        }
      } catch (error) {
        setError(
          `Import failed: ${error instanceof Error ? error.message : String(error)}`
        );
      } finally {
        setIsImporting(false);
      }
    },
    [selectedFile, spreadsheetRef, importService]
  );

  // Clear results
  const clearResult = useCallback(() => {
    setError(null);
    setSuccess(null);
  }, []);

  return {
    // State
    importMode,
    setImportMode,
    targetRange,
    setTargetRange,
    libraryDataRange,
    setLibraryDataRange,
    libraryUncertaintyRange,
    setLibraryUncertaintyRange,
    isImporting,
    error,
    success,
    selectedFile,
    fileMetadata,

    // Actions
    selectFile,
    importFile,
    clearResult,
  };
}
