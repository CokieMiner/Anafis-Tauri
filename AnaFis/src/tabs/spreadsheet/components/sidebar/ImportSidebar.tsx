import React, { useCallback, useEffect } from 'react';
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

import { useSpreadsheetSelection } from '@/tabs/spreadsheet/managers/useSpreadsheetSelection';
import { sidebarStyles } from '@/tabs/spreadsheet/components/sidebar/utils/sidebarStyles';
import SidebarCard from '@/tabs/spreadsheet/components/sidebar/SidebarCard';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';
import { spreadsheetEventBus } from '@/tabs/spreadsheet/managers/SpreadsheetEventBus';
import { useImport } from '@/tabs/spreadsheet/components/sidebar/logic/useImport';
import { FileImportPanel } from '@/tabs/spreadsheet/components/sidebar/ImportSidebarComponents/FileImportPanel';
import { LibraryImportPanel } from '@/tabs/spreadsheet/components/sidebar/ImportSidebarComponents/LibraryImportPanel';

import type { ImportSidebarProps } from '@/core/types/import';

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
  // Use the import hook - all business logic is now here
  const {
    importMode,
    setImportMode,
    targetRange,
    setTargetRange,
    libraryDataRange,
    setLibraryDataRange,
    libraryUncertaintyRange,
    setLibraryUncertaintyRange,
  } = useImport({
    spreadsheetRef,
    importService,
  });

  // Spreadsheet selection hook for range inputs
  const updateField = useCallback((inputType: FocusedInputType, selection: string) => {
    if (inputType === 'targetRange') {
      setTargetRange(selection);
    } else if (inputType === 'libraryDataRange') {
      setLibraryDataRange(selection);
    } else if (inputType === 'libraryUncertaintyRange') {
      setLibraryUncertaintyRange(selection);
    }
  }, [setTargetRange, setLibraryDataRange, setLibraryUncertaintyRange]);

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
      const handler = window.__importSelectionHandler;
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
