import React, { useCallback, useEffect } from 'react';
import {
  Box,
  Typography,
  TextField,
  Button,
  IconButton,
  Alert,
  RadioGroup,
  FormControlLabel,
  Radio,
  FormControl,
  FormLabel,
  Select,
  MenuItem,
  CircularProgress,
  Divider,
  SelectChangeEvent
} from '@mui/material';
import CloseIcon from '@mui/icons-material/Close';
import FileDownloadIcon from '@mui/icons-material/FileDownload';
import SaveIcon from '@mui/icons-material/Save';
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { ExportService, ExportFormat, ExportRangeMode } from '@/core/types/export';
import { useSpreadsheetSelection } from '@/tabs/spreadsheet/managers/useSpreadsheetSelection';
import { sidebarStyles } from '@/tabs/spreadsheet/components/sidebar/utils/sidebarStyles';
import SidebarCard from '@/tabs/spreadsheet/components/sidebar/SidebarCard';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';
import { spreadsheetEventBus } from '@/tabs/spreadsheet/managers/SpreadsheetEventBus';
import { useExport } from '@/tabs/spreadsheet/components/sidebar/logic/useExport';

type FocusedInputType = 'customRange' | 'dataRange' | 'uncertaintyRange' | null;

interface ExportSidebarProps {
  open: boolean;
  onClose: () => void;
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  exportService: ExportService;
}

const ExportSidebar = React.memo<ExportSidebarProps>(({
  open,
  onClose,
  spreadsheetRef,
  onSelectionChange,
  exportService,
}) => {
  // Use the export hook - all business logic is now here
  const {
    exportFormat, setExportFormat,
    rangeMode, setRangeMode,
    customRange, setCustomRange,
    customDelimiter, setCustomDelimiter,
    isExporting,
    error,
    success,
    exportMode, setExportMode,
    libraryName, setLibraryName,
    libraryDescription, setLibraryDescription,
    libraryTags, setLibraryTags,
    libraryUnit, setLibraryUnit,
    dataRange, setDataRange,
    uncertaintyRange, setUncertaintyRange,
    handleExport,
    handleExportToLibrary,
  } = useExport({
    spreadsheetRef,
    exportService,
    ...(onSelectionChange && { onSelectionChange }),
  });

  // State
  // All state is now managed by the useExport hook



  // Memoized update field function for better performance
  const updateField = useCallback((inputType: FocusedInputType, selection: string) => {
    if (inputType === 'customRange') {
      setCustomRange(selection);
    } else if (inputType === 'dataRange') {
      setDataRange(selection);
    } else if (inputType === 'uncertaintyRange') {
      setUncertaintyRange(selection);
    }
  }, [setCustomRange, setDataRange, setUncertaintyRange]);

  // Stable no-op function to prevent unnecessary re-renders
  const noopSelectionChange = useCallback(() => {
    // No-op function for when onSelectionChange is not provided
  }, []);

  const { focusedInput, handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange: onSelectionChange ?? noopSelectionChange,
    updateField,
    sidebarDataAttribute: 'data-export-sidebar',
    handlerName: '__exportSelectionHandler',
  });

  // Subscribe to spreadsheet selection events via event bus
  useEffect(() => {
    if (!open) { return; }

    const unsubscribe = spreadsheetEventBus.on('selection-change', (cellRef) => {
      // Call the window handler that the hook is listening to
      const handler = window.__exportSelectionHandler;
      if (handler) {
        handler(cellRef);
      }
      // NOTE: Don't call onSelectionChange here - it would create an infinite loop
      // since onSelectionChange emits to the event bus, which triggers this handler again
    });

    return unsubscribe;
  }, [open]);

  // File extension and filter utilities moved to exportService
  // Export service is now injected via props for better abstraction

  if (!open) { return null; }

  return (
    <Box data-export-sidebar sx={{ ...sidebarStyles.container, px: 1, pt: 2 }}>
      {/* Header */}
      <Box sx={sidebarStyles.header}>
        <Typography sx={sidebarStyles.text.header}>
          Export Data
        </Typography>
        <IconButton
          onClick={onClose}
          sx={sidebarStyles.iconButton}
        >
          <CloseIcon />
        </IconButton>
      </Box>

      {/* Main Content */}
      <Box sx={sidebarStyles.contentWrapper}>
        {/* Export Mode */}
        <SidebarCard title="Export Mode" defaultExpanded={true}>
          <FormControl fullWidth>
            <RadioGroup
              value={exportMode}
              onChange={(e) => setExportMode(e.target.value as 'file' | 'library')}
              row
            >
              <FormControlLabel
                value="file"
                control={<Radio sx={{ color: anafisColors.spreadsheet, '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                label="Export to File"
                sx={{ color: 'rgba(255, 255, 255, 0.9)', flex: 1 }}
              />
              <FormControlLabel
                value="library"
                control={<Radio sx={{ color: anafisColors.spreadsheet, '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                label="Export to Data Library"
                sx={{ color: 'rgba(255, 255, 255, 0.9)', flex: 1 }}
              />
            </RadioGroup>
          </FormControl>
        </SidebarCard>

        {/* File Export Settings */}
        {exportMode === 'file' && (
          <SidebarCard title="Export Configuration" defaultExpanded={true}>
            {/* Export Format */}
            <FormControl fullWidth sx={{ mb: 3 }}>
              <FormLabel component="legend" sx={{ color: anafisColors.spreadsheet, mb: 1, fontWeight: 'bold', fontSize: 11, textTransform: 'uppercase', letterSpacing: 0.5, '&.Mui-focused': { color: anafisColors.spreadsheet } }}>
                1. Select Format
              </FormLabel>
              <Select
                value={exportFormat}
                onChange={(e: SelectChangeEvent) => setExportFormat(e.target.value as ExportFormat)}
                sx={{
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& .MuiOutlinedInput-notchedOutline': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover .MuiOutlinedInput-notchedOutline': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused .MuiOutlinedInput-notchedOutline': { borderColor: anafisColors.spreadsheet },
                  '& .MuiSelect-select': { color: 'white' },
                  '& .MuiSvgIcon-root': { color: anafisColors.spreadsheet }
                }}
              >
                {/* PRIMARY: Lossless native format */}
                <MenuItem value="anafispread">🎯 AnaFis Spreadsheet (.anafispread) - Lossless</MenuItem>

                {/* SIMPLE INTERCHANGE: Data exchange formats */}
                <MenuItem value="csv">📊 CSV (Comma-separated values)</MenuItem>
                <MenuItem value="tsv">📊 TSV (Tab-separated values)</MenuItem>
                <MenuItem value="txt">📊 TXT (Custom delimiter)</MenuItem>
                <MenuItem value="parquet">📊 Parquet (.parquet)</MenuItem>

                {/* READ-ONLY DOCUMENTS: Report formats */}
                <MenuItem value="html">📄 HTML (.html)</MenuItem>
                <MenuItem value="markdown">📄 Markdown (.md)</MenuItem>
                <MenuItem value="tex">📄 LaTeX (.tex)</MenuItem>
              </Select>
            </FormControl>

            {/* Special message for .anafispread - no options needed */}
            {exportFormat === 'anafispread' ? (
              <Box sx={{
                mt: 3,
                p: 2,
                bgcolor: 'rgba(76, 175, 80, 0.1)',
                borderRadius: '8px',
                border: '1px solid rgba(76, 175, 80, 0.3)'
              }}>
                <Typography sx={{ color: '#81c784', fontWeight: 'bold', mb: 1, fontSize: 13 }}>
                  ✓ Lossless Native Format
                </Typography>
                <Typography sx={{ color: 'rgba(255, 255, 255, 0.8)', fontSize: 12, mb: 1 }}>
                  The .anafispread format exports the complete workbook including:
                </Typography>
                <Box component="ul" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 11, pl: 2, m: 0 }}>
                  <li>All sheets, cells, formulas, and values</li>
                  <li>Complete formatting and styles</li>
                  <li>Merged cells and protection rules</li>
                  <li>All metadata and resources</li>
                </Box>
              </Box>
            ) : (
              <>
                {/* Export Range - Hidden for .anafispread */}
                <FormControl component="fieldset" fullWidth sx={{ mb: 3 }}>
                  <FormLabel component="legend" sx={{ color: anafisColors.spreadsheet, mb: 1, fontWeight: 'bold', fontSize: 11, textTransform: 'uppercase', letterSpacing: 0.5, '&.Mui-focused': { color: anafisColors.spreadsheet } }}>
                    2. Choose Data Range
                  </FormLabel>
                  <RadioGroup
                    value={rangeMode}
                    onChange={(e) => setRangeMode(e.target.value as ExportRangeMode)}
                  >
                    <FormControlLabel
                      value="sheet"
                      control={<Radio sx={{ color: anafisColors.spreadsheet, '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                      label="Current Sheet"
                      sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                    />
                    <FormControlLabel
                      value="custom"
                      control={<Radio sx={{ color: anafisColors.spreadsheet, '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                      label="Custom Range"
                      sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                    />
                  </RadioGroup>

                  {rangeMode === 'custom' && (
                    <TextField
                      fullWidth
                      size="small"
                      value={customRange}
                      onChange={(e) => setCustomRange(e.target.value)}
                      onFocus={() => handleInputFocus('customRange')}
                      onBlur={handleInputBlur}
                      placeholder="e.g., A1:D20"
                      sx={{
                        mt: 1,
                        '& .MuiOutlinedInput-root': {
                          bgcolor: focusedInput === 'customRange' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                          borderRadius: '6px',
                          '& fieldset': { borderColor: focusedInput === 'customRange' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                          '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                          '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                          '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                        }
                      }}
                    />
                  )}
                </FormControl>

                <Divider sx={{ my: 2, bgcolor: 'rgba(33, 150, 243, 0.2)' }} />

                {/* Format-specific options - only custom delimiter for TXT */}
                {exportFormat === 'txt' && (
                  <Box sx={{ mb: 3 }}>
                    <FormLabel sx={{ color: anafisColors.spreadsheet, mb: 1, fontWeight: 'bold', fontSize: 11, textTransform: 'uppercase', letterSpacing: 0.5 }}>
                      3. Custom Delimiter
                    </FormLabel>
                    <TextField
                      fullWidth
                      size="small"
                      value={customDelimiter}
                      onChange={(e) => setCustomDelimiter(e.target.value)}
                      placeholder="Enter delimiter character (e.g., |, ;, tab)"
                      helperText="CSV uses comma (,) and TSV uses tab - only TXT allows custom delimiters"
                      sx={{
                        mt: 1,
                        '& .MuiOutlinedInput-root': {
                          bgcolor: 'rgba(33, 150, 243, 0.05)',
                          borderRadius: '6px',
                          '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                          '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                          '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                          '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                        },
                        '& .MuiFormHelperText-root': { color: 'rgba(33, 150, 243, 0.6)', fontSize: 11 }
                      }}
                    />
                  </Box>
                )}
              </>
            )}
          </SidebarCard>
        )}

        {/* Data Library Export Settings */}
        {exportMode === 'library' && (
          <SidebarCard title="Data Library Configuration" defaultExpanded={true}>
            <TextField
              fullWidth
              label="Name"
              value={libraryName}
              onChange={(e) => setLibraryName(e.target.value)}
              required
              sx={{
                mb: 2,
                '& .MuiInputLabel-root': { color: anafisColors.spreadsheet, '&.Mui-focused': { color: anafisColors.spreadsheet } },
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white' }
                }
              }}
            />

            <TextField
              fullWidth
              label="Description"
              value={libraryDescription}
              onChange={(e) => setLibraryDescription(e.target.value)}
              multiline
              rows={2}
              sx={{
                mb: 2,
                '& .MuiInputLabel-root': { color: anafisColors.spreadsheet, '&.Mui-focused': { color: anafisColors.spreadsheet } },
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& textarea': { color: 'white' }
                }
              }}
            />

            <TextField
              fullWidth
              label="Tags (comma-separated)"
              value={libraryTags}
              onChange={(e) => setLibraryTags(e.target.value)}
              placeholder="experimental, measurement, analysis"
              sx={{
                mb: 2,
                '& .MuiInputLabel-root': { color: anafisColors.spreadsheet, '&.Mui-focused': { color: anafisColors.spreadsheet } },
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white' }
                }
              }}
            />

            <TextField
              fullWidth
              label="Unit"
              value={libraryUnit}
              onChange={(e) => setLibraryUnit(e.target.value)}
              placeholder="m, kg, s, V, A"
              sx={{
                mb: 2,
                '& .MuiInputLabel-root': { color: anafisColors.spreadsheet, '&.Mui-focused': { color: anafisColors.spreadsheet } },
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white' }
                }
              }}
            />

            <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
              <TextField
                label="Data Range"
                value={dataRange}
                onChange={(e) => setDataRange(e.target.value.toUpperCase())}
                onFocus={() => handleInputFocus('dataRange')}
                onBlur={handleInputBlur}
                placeholder="A:A or A1:A100"
                required
                sx={{
                  flex: 1,
                  '& .MuiInputLabel-root': { color: anafisColors.spreadsheet, '&.Mui-focused': { color: anafisColors.spreadsheet } },
                  '& .MuiOutlinedInput-root': {
                    bgcolor: focusedInput === 'dataRange' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '6px',
                    '& fieldset': { borderColor: focusedInput === 'dataRange' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                    '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                    '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                    '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                  },
                  '& .MuiFormHelperText-root': { color: 'rgba(33, 150, 243, 0.6)', fontSize: 11 }
                }}
                helperText="Entire column: A:A, C:C | Specific range: A1:A100, C5:C50"
              />

              <TextField
                label="Uncertainty Range"
                value={uncertaintyRange}
                onChange={(e) => setUncertaintyRange(e.target.value.toUpperCase())}
                onFocus={() => handleInputFocus('uncertaintyRange')}
                onBlur={handleInputBlur}
                placeholder="B:B or B1:B100"
                sx={{
                  flex: 1,
                  '& .MuiInputLabel-root': { color: anafisColors.spreadsheet, '&.Mui-focused': { color: anafisColors.spreadsheet } },
                  '& .MuiOutlinedInput-root': {
                    bgcolor: focusedInput === 'uncertaintyRange' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '6px',
                    '& fieldset': { borderColor: focusedInput === 'uncertaintyRange' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                    '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                    '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                    '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                  },
                  '& .MuiFormHelperText-root': { color: 'rgba(33, 150, 243, 0.6)', fontSize: 11 }
                }}
                helperText="Optional uncertainty values (same format as data range)"
              />
            </Box>

            <Typography sx={{
              color: anafisColors.spreadsheet,
              fontSize: 12,
              mb: 2,
              bgcolor: 'rgba(33, 150, 243, 0.1)',
              p: 1.5,
              borderRadius: 1,
              border: '1px solid rgba(33, 150, 243, 0.3)'
            }}>
              <strong>Range Examples:</strong> A:A (entire column A), 1:1 (entire row 1), A1:A100 (specific range)
            </Typography>

            <Typography sx={{ color: 'rgba(255, 255, 255, 0.6)', fontSize: 12, mb: 2 }}>
              Only numeric values from the specified ranges will be saved to the library.
            </Typography>
          </SidebarCard>
        )}

        {/* Export Actions */}
        <SidebarCard title="Export Actions" defaultExpanded={true}>
          {/* Status Messages */}
          {error && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {error}
            </Alert>
          )}

          {success && (
            <Alert severity="success" sx={{ mb: 2 }}>
              {success}
            </Alert>
          )}

          {/* Export Button */}
          {exportMode === 'file' ? (
            <Button
              fullWidth
              startIcon={isExporting ? <CircularProgress size={20} sx={{ color: 'white' }} /> : <FileDownloadIcon />}
              onClick={() => void handleExport()}
              disabled={isExporting || (rangeMode === 'custom' && !customRange)}
              sx={sidebarStyles.button.primary}
            >
              {isExporting ? 'Exporting...' : 'Export File'}
            </Button>
          ) : (
            <Button
              fullWidth
              startIcon={isExporting ? <CircularProgress size={20} sx={{ color: 'white' }} /> : <SaveIcon />}
              onClick={() => void handleExportToLibrary()}
              disabled={isExporting || !libraryName.trim() || !dataRange.trim()}
              sx={sidebarStyles.button.primary}
            >
              {isExporting ? 'Saving...' : 'Save to Data Library'}
            </Button>
          )}
        </SidebarCard>
      </Box>
    </Box>
  );
});

ExportSidebar.displayName = 'ExportSidebar';

export default ExportSidebar;
