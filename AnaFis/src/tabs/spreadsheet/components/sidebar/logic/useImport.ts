/**
 * useImport hook - Extracted business logic for import operations
 *
 * This hook encapsulates all the business logic for importing data into spreadsheets,
 * including state management, file selection, and API calls.
 */

import { useCallback, useState } from 'react';
import type {
  FileMetadata,
  ImportOptions,
  ImportService,
} from '@/core/types/import';
import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';

interface UseImportOptions {
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  importService: ImportService;
}

export function useImport({ spreadsheetRef, importService }: UseImportOptions) {
  // Import state
  const [isImporting, setIsImporting] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<{
    filePath: string;
    detectedFormat: string;
  } | null>(null);
  const [fileMetadata, setFileMetadata] = useState<FileMetadata | null>(null);

  // Import mode: 'file' or 'library'
  const [importMode, setImportMode] = useState<'file' | 'library'>('file');

  // Range state (shared between components)
  const [targetRange, setTargetRange] = useState<string>('A1');
  const [libraryDataRange, setLibraryDataRange] = useState<string>('A1');
  const [libraryUncertaintyRange, setLibraryUncertaintyRange] =
    useState<string>('B1');

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

  // Clear file selection
  const clearSelection = useCallback(() => {
    setSelectedFile(null);
    setFileMetadata(null);
    setError(null);
    setSuccess(null);
  }, []);

  // Get supported formats
  const getSupportedFormats = useCallback(() => {
    return importService.getSupportedFormats();
  }, [importService]);

  return {
    // State
    isImporting,
    error,
    success,
    selectedFile,
    fileMetadata,
    importMode,
    setImportMode,
    targetRange,
    setTargetRange,
    libraryDataRange,
    setLibraryDataRange,
    libraryUncertaintyRange,
    setLibraryUncertaintyRange,

    // Actions
    selectFile,
    importFile,
    clearResult,
    clearSelection,
    getSupportedFormats,
  };
}
