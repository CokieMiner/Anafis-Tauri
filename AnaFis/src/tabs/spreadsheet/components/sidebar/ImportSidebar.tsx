import CloseIcon from '@mui/icons-material/Close';
import {
  Box,
  FormControl,
  FormControlLabel,
  IconButton,
  Radio,
  RadioGroup,
  Typography,
} from '@mui/material';
import React, { useEffect } from 'react';
import type { ImportSidebarProps } from '@/core/types/import';
import { FileImportPanel } from '@/tabs/spreadsheet/components/sidebar/ImportSidebarComponents/FileImportPanel';
import { LibraryImportPanel } from '@/tabs/spreadsheet/components/sidebar/ImportSidebarComponents/LibraryImportPanel';
import { useImport } from '@/tabs/spreadsheet/components/sidebar/logic/useImport';
import SidebarCard from '@/tabs/spreadsheet/components/sidebar/SidebarCard';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';
import { sidebarStyles } from '@/tabs/spreadsheet/components/sidebar/utils/sidebarStyles';
import { useSpreadsheetSelection } from '@/tabs/spreadsheet/managers/useSpreadsheetSelection';

type FocusedInputType =
  | 'targetRange'
  | 'libraryDataRange'
  | 'libraryUncertaintyRange'
  | null;

/**
 * Import Sidebar Container Component
 * Manages mode switching between file import and library import
 */
const ImportSidebar = React.memo<ImportSidebarProps>(
  ({ open, onClose, spreadsheetRef, onSelectionChange, importService }) => {
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
    const { focusedInput, handleInputFocus, handleInputBlur } =
      useSpreadsheetSelection<FocusedInputType>({
        onSelectionChange: onSelectionChange ?? (() => {}),
        updateField: React.useCallback(
          (inputType, selection) => {
            switch (inputType) {
              case 'targetRange':
                setTargetRange(selection);
                break;
              case 'libraryDataRange':
                setLibraryDataRange(selection);
                break;
              case 'libraryUncertaintyRange':
                setLibraryUncertaintyRange(selection);
                break;
            }
          },
          [setTargetRange, setLibraryDataRange, setLibraryUncertaintyRange]
        ),
        sidebarDataAttribute: 'data-import-sidebar',
      });

    // Subscribe to spreadsheet selection events
    useEffect(() => {
      if (!open) {
        return;
      }

      // No longer needed - selection is handled via context in the hook
      return;
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
          <Typography sx={sidebarStyles.text.header}>Import Data</Typography>
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
                onChange={(e) =>
                  setImportMode(e.target.value as 'file' | 'library')
                }
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
  }
);

ImportSidebar.displayName = 'ImportSidebar';

export default ImportSidebar;
