import React, { useState, useCallback, useEffect, useMemo } from 'react';
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
  Checkbox,
  CircularProgress,
  Divider,
  SelectChangeEvent
} from '@mui/material';
import CloseIcon from '@mui/icons-material/Close';
import FileDownloadIcon from '@mui/icons-material/FileDownload';
import SaveIcon from '@mui/icons-material/Save';
// invoke and save moved to exportService
import { useSpreadsheetSelection } from '@/hooks/useSpreadsheetSelection';
import { sidebarStyles } from '@/utils/sidebarStyles';
import SidebarCard from './SidebarCard';
import { anafisColors } from '@/themes';
import { spreadsheetEventBus } from '../SpreadsheetEventBus';
import {
  ExportSidebarProps,
  ExportFormat,
  ExportRangeMode,
  JsonFormat,
  ExportOptions
} from '@/types/export';
import { ExportService } from '../univer/exportService';
import { FUniver } from '@univerjs/core/facade';

type FocusedInputType = 'customRange' | 'dataRange' | 'uncertaintyRange' | null;

const ExportSidebar = React.memo<ExportSidebarProps>(({
  open,
  onClose,
  univerRef,
  onSelectionChange,
  exportFormat,
  setExportFormat,
  rangeMode,
  setRangeMode,
  customRange,
  setCustomRange,
  jsonFormat,
  setJsonFormat,
  prettyPrint,
  setPrettyPrint,
  customDelimiter,
  setCustomDelimiter,
  getTrackedBounds,
}) => {
  // State
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Export mode: 'file' or 'library'
  const [exportMode, setExportMode] = useState<'file' | 'library'>('file');

  // Headers option
  const [includeHeaders, setIncludeHeaders] = useState<boolean>(false);

  // New lossless export options
  const [includeFormulas, setIncludeFormulas] = useState<boolean>(false);
  const [includeFormatting, setIncludeFormatting] = useState<boolean>(false);
  const [includeMetadata, setIncludeMetadata] = useState<boolean>(false);
  const [useLosslessExtraction, setUseLosslessExtraction] = useState<boolean>(false);

  // Data Library export state
  const [libraryName, setLibraryName] = useState('');
  const [libraryDescription, setLibraryDescription] = useState('');
  const [libraryTags, setLibraryTags] = useState('');
  const [libraryUnit, setLibraryUnit] = useState('');
  const [dataRange, setDataRange] = useState('A:A');
  const [uncertaintyRange, setUncertaintyRange] = useState('');



  // Use spreadsheet selection hook
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
      const handler = (window as unknown as Record<string, (cellRef: string) => void>).__exportSelectionHandler;
      if (handler) {
        handler(cellRef);
      }
      // NOTE: Don't call onSelectionChange here - it would create an infinite loop
      // since onSelectionChange emits to the event bus, which triggers this handler again
    });

    return unsubscribe;
  }, [open]);

  // File extension and filter utilities moved to exportService

  // Create export service instance
  const exportService = useMemo(() => new ExportService(), []);

  // Data Library export logic moved to exportService

  // Handle export to Data Library
  const handleExportToLibrary = useCallback(async () => {
    setError(null);
    setSuccess(null);
    setIsExporting(true);

    try {
      // Get the univerAPI from the ref
      const univerAPI = univerRef?.current?.getRawAPI?.();
      if (!univerAPI) {
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
      }, univerAPI as ReturnType<typeof FUniver.newAPI>);

      if (result.success) {
        setSuccess(result.message ?? 'Successfully saved to Data Library');

        // Reset form
        setLibraryName('');
        setLibraryDescription('');
        setLibraryTags('');
        setLibraryUnit('');
        setDataRange('A:A');
        setUncertaintyRange('');
      } else {
        setError(result.error ?? 'Failed to save to Data Library');
      }
    } catch (err) {
      setError(`Failed to save to Data Library: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsExporting(false);
    }
  }, [libraryName, libraryDescription, libraryTags, libraryUnit, dataRange, uncertaintyRange, exportService, univerRef]);

  // Handle export action
  const handleExport = useCallback(async () => {
    setError(null);
    setSuccess(null);
    setIsExporting(true);

    try {
      // Create the discriminated union ExportOptions based on rangeMode
      const baseOptions = {
        exportFormat,
        includeHeaders,
        losslessExtraction: useLosslessExtraction,
        includeFormulas,
        includeFormatting,
        includeMetadata,
        jsonFormat,
        prettyPrint,
        delimiter: customDelimiter,
        encoding: 'utf8' as const,
        trackedBounds: getTrackedBounds?.() ?? null,
      };

      let options: ExportOptions;
      switch (rangeMode) {
        case 'sheet':
          options = {
            ...baseOptions,
            rangeMode: 'sheet',
          };
          break;
        case 'all':
          options = {
            ...baseOptions,
            rangeMode: 'all',
          };
          break;
        case 'custom':
          options = {
            ...baseOptions,
            rangeMode: 'custom',
            customRange,
          };
          break;
      }

      const univerAPI = univerRef?.current?.getRawAPI?.();
      if (!univerAPI) {
        setError('Spreadsheet API not available');
        setIsExporting(false);
        return;
      }

      // Type guard to validate API shape instead of unsafe assertion
      const isValidUniverAPI = (api: unknown): api is ReturnType<typeof FUniver.newAPI> => {
        return (
          api !== null &&
          typeof api === 'object' &&
          'getActiveWorkbook' in api &&
          typeof (api as Record<string, unknown>).getActiveWorkbook === 'function'
        );
      };

      if (!isValidUniverAPI(univerAPI)) {
        setError('Spreadsheet API is not in the expected format');
        setIsExporting(false);
        return;
      }

      const result = await exportService.exportWithDialog(options, univerAPI);

      if (result.success) {
        setSuccess(result.message ?? 'Export completed successfully');
      } else {
        setError(result.error ?? 'Export failed');
      }
    } catch (err) {
      setError(`Export failed: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setIsExporting(false);
    }
  }, [exportFormat, rangeMode, customRange, includeHeaders, useLosslessExtraction, includeFormulas, includeFormatting, includeMetadata, jsonFormat, prettyPrint, customDelimiter, getTrackedBounds, exportService, univerRef]);

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
                control={<Radio sx={{ color: '#64b5f6', '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                label="Export to File"
                sx={{ color: 'rgba(255, 255, 255, 0.9)', flex: 1 }}
              />
              <FormControlLabel
                value="library"
                control={<Radio sx={{ color: '#64b5f6', '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
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
              <FormLabel component="legend" sx={{ color: '#64b5f6', mb: 1, fontWeight: 'bold', fontSize: 11, textTransform: 'uppercase', letterSpacing: 0.5, '&.Mui-focused': { color: '#2196f3' } }}>
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
                  '&.Mui-focused .MuiOutlinedInput-notchedOutline': { borderColor: '#2196f3' },
                  '& .MuiSelect-select': { color: 'white' },
                  '& .MuiSvgIcon-root': { color: '#64b5f6' }
                }}
              >
                <MenuItem value="csv">CSV (Comma-separated values)</MenuItem>
                <MenuItem value="tsv">TSV (Tab-separated values)</MenuItem>
                <MenuItem value="txt">TXT (Custom delimiter)</MenuItem>
                <MenuItem value="json">JSON (JavaScript Object Notation)</MenuItem>
                <MenuItem value="xlsx">Excel (.xlsx)</MenuItem>
                <MenuItem value="anafispread">AnaFis Spreadsheet (.anafispread)</MenuItem>
                <MenuItem value="parquet">Parquet (.parquet)</MenuItem>
                <MenuItem value="tex">LaTeX (.tex)</MenuItem>
                <MenuItem value="html">HTML (.html)</MenuItem>
                <MenuItem value="markdown">Markdown (.md)</MenuItem>
              </Select>

              {/* Format-specific info */}
              <Typography sx={{ color: 'rgba(255, 152, 0, 0.8)', fontSize: 11, mt: 1, display: 'flex', alignItems: 'center', gap: 0.5 }}>
                {(exportFormat === 'xlsx' || exportFormat === 'anafispread')
                  ? 'Supports formulas, formatting, and metadata preservation'
                  : 'Formulas will be evaluated to their calculated values'
                }
              </Typography>
            </FormControl>

            {/* Export Range */}
            <FormControl component="fieldset" fullWidth sx={{ mb: 3 }}>
              <FormLabel component="legend" sx={{ color: '#64b5f6', mb: 1, fontWeight: 'bold', fontSize: 11, textTransform: 'uppercase', letterSpacing: 0.5, '&.Mui-focused': { color: '#2196f3' } }}>
                2. Choose Data Range
              </FormLabel>
              <RadioGroup
                value={rangeMode}
                onChange={(e) => setRangeMode(e.target.value as ExportRangeMode)}
              >
                <FormControlLabel
                  value="sheet"
                  control={<Radio sx={{ color: '#64b5f6', '&.Mui-checked': { color: '#2196f3' } }} />}
                  label="Current Sheet"
                  sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                />
                <FormControlLabel
                  value="all"
                  control={<Radio sx={{ color: '#64b5f6', '&.Mui-checked': { color: '#2196f3' } }} />}
                  label="All Sheets"
                  sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                  disabled={!(exportFormat === 'xlsx' || exportFormat === 'anafispread' || exportFormat === 'json')}
                />
                <FormControlLabel
                  value="custom"
                  control={<Radio sx={{ color: '#64b5f6', '&.Mui-checked': { color: '#2196f3' } }} />}
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
                      '& fieldset': { borderColor: focusedInput === 'customRange' ? '#2196f3' : 'rgba(33, 150, 243, 0.2)' },
                      '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                      '&.Mui-focused fieldset': { borderColor: '#2196f3' },
                      '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                    }
                  }}
                />
              )}
            </FormControl>

            <Divider sx={{ my: 2, bgcolor: 'rgba(33, 150, 243, 0.2)' }} />

            {/* Lossless Export Options */}
            {(exportFormat === 'xlsx' || exportFormat === 'anafispread' || exportFormat === 'json') && (
              <Box sx={{ mb: 3 }}>
                <FormLabel sx={{ color: '#64b5f6', mb: 1, fontWeight: 'bold', fontSize: 11, textTransform: 'uppercase', letterSpacing: 0.5 }}>
                  3. Advanced Options
                </FormLabel>

                <FormControlLabel
                  control={
                    <Checkbox
                      checked={useLosslessExtraction}
                      onChange={(e) => {
                        setUseLosslessExtraction(e.target.checked);
                        if (e.target.checked) {
                          // Enable advanced options by default for lossless
                          if (exportFormat === 'xlsx' || exportFormat === 'anafispread') {
                            setIncludeFormulas(true);
                            setIncludeFormatting(true);
                          }
                          if (exportFormat === 'anafispread') {
                            setIncludeMetadata(true);
                          }
                        }
                      }}
                      sx={{ color: '#64b5f6', '&.Mui-checked': { color: '#2196f3' } }}
                    />
                  }
                  label="Advanced Data Extraction"
                  sx={{ color: 'rgba(255, 255, 255, 0.9)', mb: 1, display: 'block' }}
                />

                {useLosslessExtraction && (
                  <Box sx={{ ml: 4, mt: 1 }}>
                    <FormControlLabel
                      control={
                        <Checkbox
                          checked={includeFormulas}
                          onChange={(e) => setIncludeFormulas(e.target.checked)}
                          sx={{ color: '#64b5f6', '&.Mui-checked': { color: '#2196f3' } }}
                        />
                      }
                      label="Include Formulas"
                      sx={{ color: 'rgba(255, 255, 255, 0.8)', display: 'block' }}
                    />

                    <FormControlLabel
                      control={
                        <Checkbox
                          checked={includeFormatting}
                          onChange={(e) => setIncludeFormatting(e.target.checked)}
                          sx={{ color: '#64b5f6', '&.Mui-checked': { color: '#2196f3' } }}
                        />
                      }
                      label="Include Cell Formatting"
                      sx={{ color: 'rgba(255, 255, 255, 0.8)', display: 'block' }}
                    />

                    {exportFormat === 'anafispread' && (
                      <FormControlLabel
                        control={
                          <Checkbox
                            checked={includeMetadata}
                            onChange={(e) => setIncludeMetadata(e.target.checked)}
                            sx={{ color: '#64b5f6', '&.Mui-checked': { color: '#2196f3' } }}
                          />
                        }
                        label="Include Cell Metadata"
                        sx={{ color: 'rgba(255, 255, 255, 0.8)', display: 'block' }}
                      />
                    )}
                  </Box>
                )}

                <Typography sx={{
                  color: 'rgba(100, 181, 246, 0.7)',
                  fontSize: 11,
                  mt: 1,
                  fontStyle: 'italic'
                }}>
                  {useLosslessExtraction
                    ? 'Advanced extraction preserves complete spreadsheet structure'
                    : 'Standard extraction exports cell values only'
                  }
                </Typography>
              </Box>
            )}

            {/* Format-specific options */}
            <Box sx={{ mb: 3 }}>
              <FormLabel sx={{ color: '#64b5f6', mb: 1, fontWeight: 'bold', fontSize: 11, textTransform: 'uppercase', letterSpacing: 0.5, '&.Mui-focused': { color: '#2196f3' } }}>
                {(exportFormat === 'xlsx' || exportFormat === 'anafispread' || exportFormat === 'json') ? '4. Format Options' : '3. Format Options'}
              </FormLabel>

              {/* TXT delimiter option */}
              {exportFormat === 'txt' && (
                <Box sx={{ mt: 2 }}>
                  <FormLabel sx={{ color: '#64b5f6', mb: 1, fontSize: 13, '&.Mui-focused': { color: '#2196f3' } }}>
                    Custom Delimiter
                  </FormLabel>
                  <TextField
                    fullWidth
                    size="small"
                    value={customDelimiter}
                    onChange={(e) => setCustomDelimiter(e.target.value)}
                    placeholder="Enter delimiter character"
                    helperText="Examples: | (pipe), ; (semicolon), tab, space"
                    sx={{
                      mt: 0.5,
                      '& .MuiOutlinedInput-root': {
                        bgcolor: 'rgba(33, 150, 243, 0.05)',
                        borderRadius: '6px',
                        '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                        '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                        '&.Mui-focused fieldset': { borderColor: '#2196f3' },
                        '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                      },
                      '& .MuiFormHelperText-root': { color: 'rgba(33, 150, 243, 0.6)', fontSize: 11 }
                    }}
                  />
                </Box>
              )}

              {/* JSON format options */}
              {exportFormat === 'json' && (
                <Box sx={{ mt: 2 }}>
                  <FormControl fullWidth sx={{ mb: 2 }}>
                    <FormLabel sx={{ color: '#64b5f6', mb: 1, fontSize: 13, '&.Mui-focused': { color: '#2196f3' } }}>
                      JSON Structure
                    </FormLabel>
                    <Select
                      value={jsonFormat}
                      onChange={(e: SelectChangeEvent) => setJsonFormat(e.target.value as JsonFormat)}
                      sx={{
                        bgcolor: 'rgba(33, 150, 243, 0.05)',
                        borderRadius: '6px',
                        '& .MuiOutlinedInput-notchedOutline': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                        '&:hover .MuiOutlinedInput-notchedOutline': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                        '&.Mui-focused .MuiOutlinedInput-notchedOutline': { borderColor: '#2196f3' },
                        '& .MuiSelect-select': { color: 'white' },
                        '& .MuiSvgIcon-root': { color: '#64b5f6' }
                      }}
                    >
                      <MenuItem value="array">2D Array Format</MenuItem>
                      <MenuItem value="object">Column-based Object</MenuItem>
                      <MenuItem value="records">Record Array Format</MenuItem>
                    </Select>
                  </FormControl>

                  {/* Headers option - only for object and records format */}
                  {(jsonFormat === 'object' || jsonFormat === 'records') && (
                    <FormControlLabel
                      control={
                        <Checkbox
                          checked={includeHeaders}
                          onChange={(e) => setIncludeHeaders(e.target.checked)}
                          sx={{ color: '#64b5f6', '&.Mui-checked': { color: '#2196f3' } }}
                        />
                      }
                      label="Use first row as column headers"
                      sx={{ color: 'rgba(255, 255, 255, 0.9)', mb: 2 }}
                    />
                  )}

                  <FormControlLabel
                    control={
                      <Checkbox
                        checked={prettyPrint}
                        onChange={(e) => setPrettyPrint(e.target.checked)}
                        sx={{ color: '#64b5f6', '&.Mui-checked': { color: '#2196f3' } }}
                      />
                    }
                    label="Format JSON with indentation"
                    sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                  />
                </Box>
              )}
            </Box>
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
                '& .MuiInputLabel-root': { color: '#64b5f6', '&.Mui-focused': { color: '#2196f3' } },
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: '#2196f3' },
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
                '& .MuiInputLabel-root': { color: '#64b5f6', '&.Mui-focused': { color: '#2196f3' } },
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: '#2196f3' },
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
                '& .MuiInputLabel-root': { color: '#64b5f6', '&.Mui-focused': { color: '#2196f3' } },
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: '#2196f3' },
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
                '& .MuiInputLabel-root': { color: '#64b5f6', '&.Mui-focused': { color: '#2196f3' } },
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: '#2196f3' },
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
                  '& .MuiInputLabel-root': { color: '#64b5f6', '&.Mui-focused': { color: '#2196f3' } },
                  '& .MuiOutlinedInput-root': {
                    bgcolor: focusedInput === 'dataRange' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '6px',
                    '& fieldset': { borderColor: focusedInput === 'dataRange' ? '#2196f3' : 'rgba(33, 150, 243, 0.2)' },
                    '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                    '&.Mui-focused fieldset': { borderColor: '#2196f3' },
                    '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                  },
                  '& .MuiFormHelperText-root': { color: '#888', fontSize: '0.75rem' }
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
                  '& .MuiInputLabel-root': { color: '#64b5f6', '&.Mui-focused': { color: '#2196f3' } },
                  '& .MuiOutlinedInput-root': {
                    bgcolor: focusedInput === 'uncertaintyRange' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '6px',
                    '& fieldset': { borderColor: focusedInput === 'uncertaintyRange' ? '#2196f3' : 'rgba(33, 150, 243, 0.2)' },
                    '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                    '&.Mui-focused fieldset': { borderColor: '#2196f3' },
                    '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                  },
                  '& .MuiFormHelperText-root': { color: '#888', fontSize: '0.75rem' }
                }}
                helperText="Optional uncertainty values (same format as data range)"
              />
            </Box>

            <Typography sx={{
              color: '#64b5f6',
              fontSize: 12,
              mb: 2,
              bgcolor: 'rgba(33, 150, 243, 0.1)',
              p: 1.5,
              borderRadius: 1,
              border: '1px solid rgba(33, 150, 243, 0.3)'
            }}>
              <strong>Range Examples:</strong> A:A (entire column A), 1:1 (entire row 1), A1:A100 (specific range)
            </Typography>

            <Typography sx={{ color: '#888', fontSize: '0.75rem', mb: 2 }}>
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
