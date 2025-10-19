import React, { useState, useCallback, useEffect } from 'react';
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
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { useSpreadsheetSelection } from '../../hooks/useSpreadsheetSelection';
import { sidebarStyles } from '../../utils/sidebarStyles';
import SidebarCard from '../ui/SidebarCard';
import { anafisColors } from '../../themes';
import { spreadsheetEventBus } from './SpreadsheetEventBus';
import {
  ExportSidebarProps,
  ExportFormat,
  ExportRangeMode,
  ExportConfig,
  JsonFormat
} from '../../types/export';

type FocusedInputType = 'customRange' | 'dataRange' | 'uncertaintyRange' | null;

const ExportSidebar: React.FC<ExportSidebarProps> = ({
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
}) => {
  // State
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Export mode: 'file' or 'library'
  const [exportMode, setExportMode] = useState<'file' | 'library'>('file');

  // Headers option
  const [includeHeaders, setIncludeHeaders] = useState<boolean>(false);

  // Data Library export state
  const [libraryName, setLibraryName] = useState('');
  const [libraryDescription, setLibraryDescription] = useState('');
  const [libraryTags, setLibraryTags] = useState('');
  const [libraryUnit, setLibraryUnit] = useState('');
  const [dataRange, setDataRange] = useState('A:A');
  const [uncertaintyRange, setUncertaintyRange] = useState('');



  // Use spreadsheet selection hook
  const { focusedInput, handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange,
    updateField: (inputType, selection) => {
      if (inputType === 'customRange') {
        setCustomRange(selection);
      } else if (inputType === 'dataRange') {
        setDataRange(selection);
      } else if (inputType === 'uncertaintyRange') {
        setUncertaintyRange(selection);
      }
    },
    sidebarDataAttribute: 'data-export-sidebar',
    handlerName: '__exportSelectionHandler',
  });

  // Subscribe to spreadsheet selection events via event bus
  useEffect(() => {
    if (!open) return;

    const unsubscribe = spreadsheetEventBus.on('selection-change', (cellRef) => {
      // Call the window handler that the hook is listening to
      const handler = (window as any).__exportSelectionHandler;
      if (handler) {
        handler(cellRef);
      }
      // NOTE: Don't call onSelectionChange here - it would create an infinite loop
      // since onSelectionChange emits to the event bus, which triggers this handler again
    });

    return unsubscribe;
  }, [open]);

  // Get file extension for current format
  const getFileExtension = useCallback((): string => {
    switch (exportFormat) {
      case 'csv': return 'csv';
      case 'tsv': return 'tsv';
      case 'txt': return 'txt';
      case 'json': return 'json';
      case 'parquet': return 'parquet';
      case 'tex': return 'tex';
      case 'html': return 'html';
      case 'markdown': return 'md';
      default: return 'txt';
    }
  }, [exportFormat]);

  // Get file filter name
  const getFilterName = useCallback((): string => {
    switch (exportFormat) {
      case 'csv': return 'CSV File';
      case 'tsv': return 'TSV File';
      case 'txt': return 'Text File';
      case 'json': return 'JSON File';
      case 'parquet': return 'Parquet File';
      case 'tex': return 'LaTeX File';
      case 'html': return 'HTML File';
      case 'markdown': return 'Markdown File';
      default: return 'File';
    }
  }, [exportFormat]);

  // Extract data from spreadsheet based on range mode
  const extractData = useCallback(async (): Promise<unknown[][] | null> => {
    if (!univerRef?.current) {
      console.error('[Export] Spreadsheet reference not available');
      setError('Spreadsheet reference not available');
      return null;
    }

    try {
      console.log('[Export] Range mode:', rangeMode, 'Format:', exportFormat);

      // Single sheet extraction only
      let range: string;

      if (rangeMode === 'custom') {
        if (!customRange) {
          setError('Please specify a custom range');
          return null;
        }
        range = customRange;
      } else {
        // Default to 'sheet' - get used range of active sheet
        console.log('[Export] Getting used range for sheet...');
        range = await univerRef.current.getUsedRange();
        console.log('[Export] Used range:', range);
      }

      console.log('[Export] Final range:', range);
      console.log('[Export] About to call getRange...');
      const rawData = await univerRef.current.getRange(range);
      console.log('[Export] Raw data:', rawData);

      // Filter out completely empty rows and trim empty cells at the end
      const filteredData = filterEmptyData(rawData);
      console.log('[Export] Filtered data:', filteredData);

      if (filteredData.length === 0) {
        console.error('[Export] No data after filtering');
        setError('No data found in the selected range');
        return null;
      }

      return filteredData;
    } catch (err) {
      setError(`Failed to extract data: ${err}`);
      return null;
    }
    // univerRef is a ref and doesn't change, so it's safe to omit from dependencies
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rangeMode, customRange, exportFormat]);

  // Filter out empty rows and trim empty columns
  const filterEmptyData = (data: unknown[][]): unknown[][] => {
    if (!data || data.length === 0) return [];

    // Find the last non-empty row
    let lastNonEmptyRow = -1;
    for (let i = data.length - 1; i >= 0; i--) {
      if (data[i].some(cell => cell !== null && cell !== undefined && cell !== '')) {
        lastNonEmptyRow = i;
        break;
      }
    }

    if (lastNonEmptyRow === -1) {
      return []; // All rows are empty
    }

    // Trim to last non-empty row
    const trimmedRows = data.slice(0, lastNonEmptyRow + 1);

    // Find the last non-empty column across all rows
    let lastNonEmptyCol = -1;
    for (const row of trimmedRows) {
      for (let j = row.length - 1; j >= 0; j--) {
        const cell = row[j];
        if (cell !== null && cell !== undefined && cell !== '') {
          lastNonEmptyCol = Math.max(lastNonEmptyCol, j);
          break;
        }
      }
    }

    if (lastNonEmptyCol === -1) {
      return []; // All columns are empty
    }

    // Trim each row to last non-empty column
    return trimmedRows.map(row => row.slice(0, lastNonEmptyCol + 1));
  };

  // Extract range data for Data Library export
  const extractRangeData = useCallback(async (range: string): Promise<number[]> => {
    if (!univerRef?.current) {
      throw new Error('Spreadsheet reference not available');
    }

    try {
      // Use Univer's API to get the range data
      const rangeData = await univerRef.current.getRange(range);
      const values: number[] = [];

      // Flatten the range data and extract numeric values
      for (const row of rangeData) {
        for (const cell of row) {
          if (cell !== null && cell !== undefined && cell !== '') {
            const num = typeof cell === 'number' ? cell : parseFloat(String(cell));
            if (!isNaN(num)) {
              values.push(num);
            }
          }
        }
      }

      return values;
    } catch (err) {
      throw new Error(`Failed to extract range ${range}: ${err}`);
    }
  }, [univerRef]);

  // Handle export to Data Library
  const handleExportToLibrary = useCallback(async () => {
    setError(null);
    setSuccess(null);
    setIsExporting(true);

    try {
      // Validate inputs
      if (!libraryName.trim()) {
        setError('Please enter a name for the data sequence');
        return;
      }

      if (!dataRange.trim()) {
        setError('Please specify a data range');
        return;
      }

      // Extract range data
      const dataValues = await extractRangeData(dataRange);
      if (dataValues.length === 0) {
        setError(`No valid numeric data found in range ${dataRange}`);
        return;
      }

      // Extract uncertainty data if specified
      let uncertainties: number[] | undefined;
      if (uncertaintyRange.trim()) {
        uncertainties = await extractRangeData(uncertaintyRange);
        if (uncertainties.length !== dataValues.length) {
          setError(`Uncertainty range (${uncertainties.length} values) must have the same length as data range (${dataValues.length} values)`);
          return;
        }
      }

      // Parse tags
      const tags = libraryTags
        .split(',')
        .map(tag => tag.trim())
        .filter(tag => tag.length > 0);

      // Build save request
      const request = {
        name: libraryName.trim(),
        description: libraryDescription.trim(),
        tags,
        unit: libraryUnit.trim(),
        source: `Range: ${dataRange}${uncertaintyRange ? ` (uncertainty: ${uncertaintyRange})` : ''}`,
        data: dataValues,
        uncertainties: uncertainties && uncertainties.length > 0 ? uncertainties : null,
        is_pinned: false,
      };

      // Save to Data Library
      await invoke<string>('save_sequence', { request });

      setSuccess(`Successfully saved '${libraryName}' to Data Library (${dataValues.length} data points)`);

      // Reset form
      setLibraryName('');
      setLibraryDescription('');
      setLibraryTags('');
      setLibraryUnit('');
      setDataRange('A:A');
      setUncertaintyRange('');

    } catch (err) {
      setError(`Failed to save to Data Library: ${err}`);
    } finally {
      setIsExporting(false);
    }
  }, [libraryName, libraryDescription, libraryTags, libraryUnit, dataRange, uncertaintyRange, extractRangeData]);

  // Handle export action
  const handleExport = useCallback(async () => {
    setError(null);
    setSuccess(null);
    setIsExporting(true);

    try {
      // Extract data
      const data = await extractData();
      if (!data) {
        setError('No data to export');
        return;
      }

      // Handle single-sheet data only
      const exportData = data as unknown[][];
      const rowCount = exportData.length;
      const colCount = exportData[0]?.length || 0;
      const previewText = `${rowCount} rows × ${colCount} columns`;

      // Validate export data
      if (exportData.length === 0) {
        setError('No data found to export');
        return;
      }

      // Show save dialog
      const filePath = await save({
        defaultPath: `export.${getFileExtension()}`,
        filters: [{
          name: getFilterName(),
          extensions: [getFileExtension()]
        }]
      });

      if (!filePath) {
        // User cancelled
        setIsExporting(false);
        return;
      }

      // Validate file path
      if (!filePath.trim()) {
        setError('Invalid file path');
        return;
      }

      // Build export config
      const config: ExportConfig = {
        range: rangeMode === 'custom' ? customRange : rangeMode,
        format: exportFormat,
        options: {
          includeHeaders,
          delimiter: exportFormat === 'txt' ? customDelimiter : undefined,
          jsonFormat,
          prettyPrint,
          encoding: 'utf8',
          lineEnding: 'lf',
          quoteChar: '"',
          compress: false,
        }
      };

      // Call appropriate backend command based on format
      if (exportFormat === 'csv' || exportFormat === 'tsv' || exportFormat === 'txt') {
        await invoke('export_to_text', { data: exportData, filePath, config });
      } else if (exportFormat === 'json') {
        await invoke('export_to_json', { data: exportData, filePath, config });
      } else if (exportFormat === 'html') {
        await invoke('export_to_html', { data: exportData, filePath, config });
      } else if (exportFormat === 'markdown') {
        await invoke('export_to_markdown', { data: exportData, filePath, config });
      } else if (exportFormat === 'tex') {
        await invoke('export_to_latex', { data: exportData, filePath, config });
      } else if (exportFormat === 'parquet') {
        await invoke('export_to_parquet', { data: exportData, filePath, config });
      } else {
        setError(`Export format '${exportFormat}' not supported`);
        return;
      }

      setSuccess(`Successfully exported ${previewText} to ${filePath}`);
    } catch (err) {
      // Provide more specific error messages based on error type
      const errorMessage = err as string;
      if (errorMessage.includes('permission') || errorMessage.includes('access')) {
        setError('Permission denied: Cannot write to the selected file. Please check file permissions and try a different location.');
      } else if (errorMessage.includes('disk') || errorMessage.includes('space')) {
        setError('Insufficient disk space to save the file.');
      } else if (errorMessage.includes('format') || errorMessage.includes('invalid')) {
        setError(`Data format error: ${errorMessage}`);
      } else if (errorMessage.includes('encoding')) {
        setError(`Text encoding error: ${errorMessage}`);
      } else {
        setError(`Export failed: ${errorMessage || 'Unknown error occurred'}`);
      }
    } finally {
      setIsExporting(false);
    }
  }, [exportFormat, rangeMode, customRange, jsonFormat, prettyPrint, customDelimiter, includeHeaders, extractData, getFileExtension, getFilterName]);

  if (!open) return null;

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
      <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', gap: 2, p: 1.5 }}>
        {/* Export Mode */}
        <SidebarCard title="Export Destination" defaultExpanded={true}>
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
          <SidebarCard title="File Export Settings" defaultExpanded={true}>
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
                <MenuItem value="parquet">Parquet (.parquet)</MenuItem>
                <MenuItem value="tex">LaTeX (.tex)</MenuItem>
                <MenuItem value="html">HTML (.html)</MenuItem>
                <MenuItem value="markdown">Markdown (.md)</MenuItem>
              </Select>

              {/* Format-specific info */}
              <Typography sx={{ color: 'rgba(255, 152, 0, 0.8)', fontSize: 11, mt: 1, display: 'flex', alignItems: 'center', gap: 0.5 }}>
                ℹ️ Formulas will be evaluated - only values exported
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
                  label="Active Sheet"
                  sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
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

            {/* Format-specific options */}
            <Box sx={{ mb: 3 }}>
              <FormLabel sx={{ color: '#64b5f6', mb: 1, fontWeight: 'bold', fontSize: 11, textTransform: 'uppercase', letterSpacing: 0.5, '&.Mui-focused': { color: '#2196f3' } }}>
                3. Format Options
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
                    placeholder="e.g., | or ; or tab"
                    helperText="Common: | (pipe), ; (semicolon), tab, space"
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
                      <MenuItem value="array">Array - [[1,2],[3,4]]</MenuItem>
                      <MenuItem value="object">Object - {'{col1:[1,3], col2:[2,4]}'}</MenuItem>
                      <MenuItem value="records">Records - [{'{col1:1}'},{'{col1:3}'}]</MenuItem>
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
                      label="First row contains headers"
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
                    label="Pretty Print (indented)"
                    sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                  />
                </Box>
              )}
            </Box>
          </SidebarCard>
        )}

        {/* Data Library Export Settings */}
        {exportMode === 'library' && (
          <SidebarCard title="Data Library Settings" defaultExpanded={true}>
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
              placeholder="experiment, measurement, physics"
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
              placeholder="m, kg, s, V, A, etc."
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
                helperText="Column: A:A, C:C | Range: A1:A100, C5:C50"
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
                helperText="Optional - same format as data range"
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
              💡 <strong>Tip:</strong> A:A = entire column A | 1:1 = entire row 1 | A1:A100 = cells A1 to A100
            </Typography>

            <Typography sx={{ color: '#888', fontSize: '0.75rem', mb: 2 }}>
              Only numeric values from the specified ranges will be exported.
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
              variant="contained"
              startIcon={isExporting ? <CircularProgress size={20} sx={{ color: 'white' }} /> : <FileDownloadIcon />}
              onClick={handleExport}
              disabled={isExporting || (rangeMode === 'custom' && !customRange)}
              sx={{
                bgcolor: '#2196f3',
                '&:hover': { bgcolor: '#1976d2' },
                '&:disabled': { bgcolor: '#555' }
              }}
            >
              {isExporting ? 'Exporting...' : 'Export to File'}
            </Button>
          ) : (
            <Button
              fullWidth
              variant="contained"
              startIcon={isExporting ? <CircularProgress size={20} sx={{ color: 'white' }} /> : <SaveIcon />}
              onClick={handleExportToLibrary}
              disabled={isExporting || !libraryName.trim() || !dataRange.trim()}
              sx={{
                bgcolor: '#2196f3',
                '&:hover': { bgcolor: '#1976d2' },
                '&:disabled': { bgcolor: '#555' }
              }}
            >
              {isExporting ? 'Saving...' : 'Save to Library'}
            </Button>
          )}
        </SidebarCard>
      </Box>
    </Box>
  );
};

export default ExportSidebar;
