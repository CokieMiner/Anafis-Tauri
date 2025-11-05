import React, { useState, useCallback, useEffect } from 'react';
import {
  Box,
  Typography,
  IconButton,
  RadioGroup,
  FormControlLabel,
  Radio,
  FormControl,
} from '@mui/material';
import CloseIcon from '@mui/icons-material/Close';

import { useSpreadsheetSelection } from '@/hooks/useSpreadsheetSelection';
import { sidebarStyles } from '@/utils/sidebarStyles';
import SidebarCard from './SidebarCard';
import { anafisColors } from '@/themes';
import { spreadsheetEventBus } from '../SpreadsheetEventBus';
import { FileImportPanel } from './ImportSidebarComponents/FileImportPanel';
import { LibraryImportPanel } from './ImportSidebarComponents/LibraryImportPanel';

import type { ImportSidebarProps } from '@/types/import';

type FocusedInputType = 'targetRange' | 'libraryDataRange' | 'libraryUncertaintyRange' | null;

/**
 * Import Sidebar Container Component
 * Manages mode switching between file import and library import
 */
const ImportSidebar = React.memo<ImportSidebarProps>(({
  open,
  onClose,
  spreadsheetRef,
  onSelectionChange,
  importService,
}) => {
  // Import mode: 'file' or 'library'
  const [importMode, setImportMode] = useState<'file' | 'library'>('file');

  // Range state (shared between components)
  const [targetRange, setTargetRange] = useState<string>('A1');
  const [libraryDataRange, setLibraryDataRange] = useState<string>('A1');
  const [libraryUncertaintyRange, setLibraryUncertaintyRange] = useState<string>('B1');

  // Spreadsheet selection hook for range inputs
  const updateField = useCallback((inputType: FocusedInputType, selection: string) => {
    if (inputType === 'targetRange') {
      setTargetRange(selection);
    } else if (inputType === 'libraryDataRange') {
      setLibraryDataRange(selection);
    } else if (inputType === 'libraryUncertaintyRange') {
      setLibraryUncertaintyRange(selection);
    }
  }, []);

  const noopSelectionChange = useCallback(() => { }, []);

  const { focusedInput, handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange: onSelectionChange ?? noopSelectionChange,
    updateField,
    sidebarDataAttribute: 'data-import-sidebar',
    handlerName: '__importSelectionHandler',
  });

  // Subscribe to spreadsheet selection events
  useEffect(() => {
    if (!open) {
      return;
    }

    const unsubscribe = spreadsheetEventBus.on('selection-change', (cellRef) => {
      const handler = (window as unknown as Record<string, (cellRef: string) => void>).__importSelectionHandler;
      if (handler) {
        handler(cellRef);
      }
    });

    return unsubscribe;
  }, [open]);

  if (!open) {
    return null;
  }

  return (
    <Box
      data-import-sidebar
      sx={{ ...sidebarStyles.container, px: 1, pt: 2 }}
    >
      {/* ===== HEADER ===== */}
      <Box sx={sidebarStyles.header}>
        <Typography sx={sidebarStyles.text.header}>
          Import Data
        </Typography>
        <IconButton onClick={onClose} sx={sidebarStyles.iconButton}>
          <CloseIcon />
        </IconButton>
      </Box>

      {/* ===== MAIN CONTENT ===== */}
      <Box sx={sidebarStyles.contentWrapper}>
        {/* ===== IMPORT MODE SELECTOR ===== */}
        <SidebarCard title="Import Mode" defaultExpanded={true}>
          <FormControl fullWidth>
            <RadioGroup
              value={importMode}
              onChange={(e) => setImportMode(e.target.value as 'file' | 'library')}
              row
            >
              <FormControlLabel
                value="file"
                control={
                  <Radio
                    sx={{
                      color: anafisColors.spreadsheet,
                      '&.Mui-checked': { color: anafisColors.spreadsheet },
                    }}
                  />
                }
                label="Import from File"
                sx={{ color: 'rgba(255, 255, 255, 0.9)', flex: 1 }}
              />
              <FormControlLabel
                value="library"
                control={
                  <Radio
                    sx={{
                      color: anafisColors.spreadsheet,
                      '&.Mui-checked': { color: anafisColors.spreadsheet },
                    }}
                  />
                }
                label="Import from Data Library"
                sx={{ color: 'rgba(255, 255, 255, 0.9)', flex: 1 }}
              />
            </RadioGroup>
          </FormControl>
        </SidebarCard>

        {/* ===== FILE IMPORT MODE ===== */}
        {importMode === 'file' && (
          <FileImportPanel
            importService={importService}
            spreadsheetRef={spreadsheetRef}
            targetRange={targetRange}
            setTargetRange={setTargetRange}
            onInputFocus={handleInputFocus}
            onInputBlur={handleInputBlur}
            focusedInput={focusedInput}
          />
        )}

        {/* ===== DATA LIBRARY IMPORT MODE ===== */}
        {importMode === 'library' && (
          <LibraryImportPanel
            spreadsheetRef={spreadsheetRef}
            libraryDataRange={libraryDataRange}
            setLibraryDataRange={setLibraryDataRange}
            libraryUncertaintyRange={libraryUncertaintyRange}
            setLibraryUncertaintyRange={setLibraryUncertaintyRange}
            onInputFocus={handleInputFocus}
            onInputBlur={handleInputBlur}
            focusedInput={focusedInput}
          />
        )}
      </Box>
    </Box>
  );
});

ImportSidebar.displayName = 'ImportSidebar';

export default ImportSidebar;
