import React, { useState, useCallback, useEffect } from 'react';
import {
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
  SelectChangeEvent,
  Chip,
  Tooltip,
} from '@mui/material';
import FileUploadIcon from '@mui/icons-material/FileUpload';
import InfoIcon from '@mui/icons-material/Info';

import { sidebarStyles } from '@/utils/sidebarStyles';
import SidebarCard from '../SidebarCard';
import { anafisColors } from '@/themes';
import { RangeValidationWarning } from './RangeValidationWarning';
import { useImportValidation } from './useImportValidation';

import type {
  ImportFormat,
  ImportTargetMode,
  AnaFisImportMode,
  ImportOptions,
  FileMetadata,
  ImportResult,
  IImportService,
} from '@/types/import';
import type { SpreadsheetRef as SpreadsheetInterface } from '@/components/spreadsheet/SpreadsheetInterface';

interface FileImportPanelProps {
  importService: IImportService;
  spreadsheetRef: React.RefObject<SpreadsheetInterface | null>;
  targetRange: string;
  setTargetRange: (range: string) => void;
  onInputFocus: (field: 'targetRange') => void;
  onInputBlur: () => void;
  focusedInput: 'targetRange' | 'libraryDataRange' | 'libraryUncertaintyRange' | null;
}

/**
 * File Import Panel Component
 * Handles all file import functionality including file selection, format detection, and import options
 */
export const FileImportPanel: React.FC<FileImportPanelProps> = ({
  importService,
  spreadsheetRef,
  targetRange,
  setTargetRange,
  onInputFocus,
  onInputBlur,
  focusedInput,
}) => {
  const { validateFileRange } = useImportValidation();

  // File selection state
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [detectedFormat, setDetectedFormat] = useState<ImportFormat | null>(null);
  const [fileMetadata, setFileMetadata] = useState<FileMetadata | null>(null);

  // Import options state
  const [importFormat, setImportFormat] = useState<ImportFormat>('csv');
  const [targetMode, setTargetMode] = useState<ImportTargetMode>('newSheet');
  const [skipRows, setSkipRows] = useState<number>(0);
  const [customDelimiter, setCustomDelimiter] = useState<string>('|');
  const [anaFisMode, setAnaFisMode] = useState<AnaFisImportMode>('append');

  // Range validation state
  const [rangeValidation, setRangeValidation] = useState<ImportResult['rangeValidation'] | null>(null);

  // UI state
  const [isImporting, setIsImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Handle file selection
  const handleSelectFile = useCallback(async () => {
    try {
      const result = await importService.selectFile();
      if (!result) {
        return;
      }

      const { filePath, detectedFormat } = result;
      setSelectedFile(filePath);
      setError(null);
      setSuccess(null);

      // Get file metadata from backend
      try {
        const metadata = await importService.getFileMetadata(filePath);
        setFileMetadata(metadata);
        setDetectedFormat(detectedFormat);
        setImportFormat(detectedFormat);
      } catch (metadataErr) {
        console.error('Failed to get file metadata:', metadataErr);
        setFileMetadata(null);
        setError('Failed to read file metadata');
      }
    } catch (err) {
      console.error('File selection error:', err);
      setError('Failed to select file');
    }
  }, [importService]);

  // Refetch file metadata with new delimiter (for TXT files)
  const refetchMetadataWithDelimiter = useCallback(
    async (filePath: string, delimiter: string) => {
      if (importFormat !== 'txt' || !filePath) {
        return;
      }

      try {
        const metadata = await importService.getFileMetadata(filePath, delimiter);
        setFileMetadata(metadata);
      } catch (metadataErr) {
        console.error('Failed to refetch file metadata with new delimiter:', metadataErr);
        setError('Failed to update file metadata with new delimiter');
      }
    },
    [importFormat, importService]
  );

  // Handle import
  const handleImport = useCallback(async () => {
    if (!selectedFile) {
      setError('Please select a file first');
      return;
    }

    setError(null);
    setSuccess(null);
    setIsImporting(true);

    try {
      // Create import options based on format
      const options: ImportOptions = {
        format: importFormat,
        ...(importFormat !== 'anafispread' && {
          skipRows,
          ...(importFormat === 'txt' && { delimiter: customDelimiter }),
          encoding: 'utf8' as const,
          targetMode,
          ...(targetMode === 'currentRange' && { targetRange }),
        }),
        ...(importFormat === 'anafispread' && { anaFisMode }),
      };

      // Import the selected file
      const result = await importService.importFile(selectedFile, options, spreadsheetRef);

      if (result.success) {
        setSuccess(result.message ?? 'Import completed successfully');
      } else {
        setError(result.error ?? 'Import failed');
      }
    } catch (err) {
      setError(`Import failed: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsImporting(false);
    }
  }, [selectedFile, importFormat, skipRows, customDelimiter, targetMode, targetRange, anaFisMode, importService, spreadsheetRef]);

  // Validate range when target range or mode changes
  useEffect(() => {
    const validation = validateFileRange(targetRange, fileMetadata, targetMode);
    setRangeValidation(validation);
  }, [validateFileRange, targetRange, fileMetadata, targetMode]);

  // Clear range validation when switching away from currentRange mode
  useEffect(() => {
    if (targetMode !== 'currentRange') {
      setRangeValidation(null);
    }
  }, [targetMode]);

  // Get supported formats for tooltip
  const supportedFormats = importService.getSupportedFormats();

  return (
    <>
      {/* Step 1: File Selection */}
      <SidebarCard title="1. Select File" defaultExpanded={true}>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            marginBottom: 16,
          }}
        >
          <Button
            fullWidth
            variant="outlined"
            startIcon={<FileUploadIcon />}
            onClick={() => void handleSelectFile()}
            sx={{
              borderColor: anafisColors.spreadsheet,
              color: anafisColors.spreadsheet,
              '&:hover': {
                borderColor: anafisColors.spreadsheet,
                bgcolor: 'rgba(33, 150, 243, 0.1)',
              },
            }}
          >
            {selectedFile ? 'Change File' : 'Select File'}
          </Button>

          <Tooltip
            title={
              <div>
                <Typography variant="body2" sx={{ fontWeight: 'bold', mb: 1 }}>
                  Supported Formats:
                </Typography>
                {supportedFormats.map(({ format, description, extensions }: {
                  format: ImportFormat;
                  description: string;
                  extensions: string[];
                }) => (
                  <Typography
                    key={format}
                    variant="body2"
                    sx={{ fontSize: 11, mb: 0.5 }}
                  >
                    <strong>.{extensions.join(', .')}</strong> - {description}
                  </Typography>
                ))}
              </div>
            }
            arrow
            placement="left"
          >
            <IconButton size="small" sx={{ color: anafisColors.spreadsheet }}>
              <InfoIcon fontSize="small" />
            </IconButton>
          </Tooltip>
        </div>

        {/* Selected File Display */}
        {selectedFile && (
          <div
            style={{
              backgroundColor: 'rgba(76, 175, 80, 0.1)',
              borderRadius: 8,
              padding: 12,
              border: '1px solid rgba(76, 175, 80, 0.3)',
            }}
          >
            <Typography
              sx={{
                color: '#81c784',
                fontWeight: 'bold',
                fontSize: 12,
                mb: 1,
              }}
            >
              Selected File:
            </Typography>
            <Typography
              sx={{
                color: 'rgba(255, 255, 255, 0.9)',
                fontSize: 11,
                mb: 1,
                fontFamily: 'monospace',
              }}
            >
              {selectedFile.split('/').pop()}
            </Typography>

            <div
              style={{
                display: 'flex',
                gap: 8,
                alignItems: 'center',
                flexWrap: 'wrap',
              }}
            >
              <Chip
                label={`Format: ${detectedFormat?.toUpperCase()}`}
                size="small"
                sx={{
                  bgcolor: 'rgba(33, 150, 243, 0.2)',
                  color: anafisColors.spreadsheet,
                  fontSize: 10,
                }}
              />
              {fileMetadata && (
                <>
                  {fileMetadata.size && (
                    <Chip
                      label={`Size: ${(fileMetadata.size / 1024).toFixed(1)}KB`}
                      size="small"
                      sx={{
                        bgcolor: 'rgba(33, 150, 243, 0.2)',
                        color: anafisColors.spreadsheet,
                        fontSize: 10,
                      }}
                    />
                  )}
                  {fileMetadata.rowCount && (
                    <Chip
                      label={`Rows: ${fileMetadata.rowCount}`}
                      size="small"
                      sx={{
                        bgcolor: 'rgba(255, 152, 0, 0.2)',
                        color: '#ffb74d',
                        fontSize: 10,
                      }}
                    />
                  )}
                  {fileMetadata.columnCount && (
                    <Chip
                      label={`Cols: ${fileMetadata.columnCount}`}
                      size="small"
                      sx={{
                        bgcolor: 'rgba(255, 152, 0, 0.2)',
                        color: '#ffb74d',
                        fontSize: 10,
                      }}
                    />
                  )}
                </>
              )}
            </div>
          </div>
        )}
      </SidebarCard>

      {/* Step 2: Format Override */}
      {selectedFile && (
        <SidebarCard title="2. Confirm Format" defaultExpanded={true}>
          <FormControl fullWidth sx={{ mb: 2 }}>
            <FormLabel
              sx={{
                color: anafisColors.spreadsheet,
                mb: 1,
                fontWeight: 'bold',
                fontSize: 11,
                textTransform: 'uppercase',
                '&.Mui-focused': { color: anafisColors.spreadsheet },
                '&.Mui-active': { color: anafisColors.spreadsheet },
              }}
            >
              Detected: {detectedFormat?.toUpperCase()}
              {detectedFormat !== importFormat && ' (Modified)'}
            </FormLabel>
            <Select
              value={importFormat}
              onChange={(e: SelectChangeEvent) =>
                setImportFormat(e.target.value as ImportFormat)
              }
              sx={{
                bgcolor: 'rgba(33, 150, 243, 0.05)',
                borderRadius: '6px',
                '& .MuiOutlinedInput-notchedOutline': {
                  borderColor: 'rgba(33, 150, 243, 0.2)',
                },
                '&:hover .MuiOutlinedInput-notchedOutline': {
                  borderColor: 'rgba(33, 150, 243, 0.4)',
                },
                '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
                  borderColor: anafisColors.spreadsheet,
                },
                '& .MuiSelect-select': { color: 'white' },
                '& .MuiSvgIcon-root': { color: anafisColors.spreadsheet },
              }}
            >
              <MenuItem value="anafispread">
                🎯 AnaFis Spreadsheet (.anafispread) - Lossless
              </MenuItem>
              <MenuItem value="csv">📊 CSV (Comma-separated values)</MenuItem>
              <MenuItem value="tsv">📊 TSV (Tab-separated values)</MenuItem>
              <MenuItem value="txt">📊 TXT (Custom delimiter)</MenuItem>
              <MenuItem value="parquet">📊 Parquet (.parquet)</MenuItem>
            </Select>
          </FormControl>
        </SidebarCard>
      )}

      {/* Step 3: Import Options */}
      {selectedFile && (
        <>
          {/* AnaFisSpread Options */}
          {importFormat === 'anafispread' && (
            <SidebarCard title="3. AnaFisSpread Options" defaultExpanded={true}>
              <FormControl component="fieldset" fullWidth>
                <FormLabel
                  sx={{
                    color: anafisColors.spreadsheet,
                    mb: 1,
                    fontWeight: 'bold',
                    fontSize: 11,
                    textTransform: 'uppercase',
                    '&.Mui-focused': { color: anafisColors.spreadsheet },
                    '&.Mui-active': { color: anafisColors.spreadsheet },
                  }}
                >
                  Workbook Handling
                </FormLabel>
                <RadioGroup
                  value={anaFisMode}
                  onChange={(e) => setAnaFisMode(e.target.value as AnaFisImportMode)}
                >
                  <FormControlLabel
                    value="append"
                    control={
                      <Radio
                        sx={{
                          color: anafisColors.spreadsheet,
                          '&.Mui-checked': {
                            color: anafisColors.spreadsheet,
                          },
                        }}
                      />
                    }
                    label="Append Sheets"
                    sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                  />
                  <FormControlLabel
                    value="replace"
                    control={
                      <Radio
                        sx={{
                          color: anafisColors.spreadsheet,
                          '&.Mui-checked': {
                            color: anafisColors.spreadsheet,
                          },
                        }}
                      />
                    }
                    label="Replace Workbook"
                    sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                  />
                </RadioGroup>
              </FormControl>
            </SidebarCard>
          )}

          {/* Simple Format Options */}
          {importFormat !== 'anafispread' && (
            <SidebarCard title="3. Import Options" defaultExpanded={true}>
              {/* Skip Rows */}
              <TextField
                fullWidth
                type="number"
                label="Skip Rows"
                value={skipRows}
                onChange={(e) =>
                  setSkipRows(Math.max(0, parseInt(e.target.value) || 0))
                }
                helperText="Number of rows to skip from the beginning"
                sx={{
                  mb: 2,
                  '& .MuiInputLabel-root': {
                    color: anafisColors.spreadsheet,
                    '&.Mui-focused': { color: anafisColors.spreadsheet },
                  },
                  '& .MuiOutlinedInput-root': {
                    bgcolor: 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '6px',
                    '& fieldset': {
                      borderColor: 'rgba(33, 150, 243, 0.2)',
                    },
                    '&:hover fieldset': {
                      borderColor: 'rgba(33, 150, 243, 0.4)',
                    },
                    '&.Mui-focused fieldset': {
                      borderColor: anafisColors.spreadsheet,
                    },
                    '& input': { color: 'white' },
                  },
                  '& .MuiFormHelperText-root': {
                    color: 'rgba(33, 150, 243, 0.6)',
                    fontSize: 11,
                  },
                }}
              />

              {/* Custom Delimiter for TXT */}
              {importFormat === 'txt' && (
                <TextField
                  fullWidth
                  label="Custom Delimiter"
                  value={customDelimiter}
                  onChange={(e) => {
                    setCustomDelimiter(e.target.value);
                    if (selectedFile) {
                      void refetchMetadataWithDelimiter(selectedFile, e.target.value);
                    }
                  }}
                  placeholder="Enter delimiter (e.g., |, ;, tab)"
                  helperText="CSV uses comma, TSV uses tab - only TXT allows custom delimiters"
                  sx={{
                    mb: 2,
                    '& .MuiInputLabel-root': {
                      color: anafisColors.spreadsheet,
                      '&.Mui-focused': { color: anafisColors.spreadsheet },
                    },
                    '& .MuiOutlinedInput-root': {
                      bgcolor: 'rgba(33, 150, 243, 0.05)',
                      borderRadius: '6px',
                      '& fieldset': {
                        borderColor: 'rgba(33, 150, 243, 0.2)',
                      },
                      '&:hover fieldset': {
                        borderColor: 'rgba(33, 150, 243, 0.4)',
                      },
                      '&.Mui-focused fieldset': {
                        borderColor: anafisColors.spreadsheet,
                      },
                      '& input': {
                        color: 'white',
                        fontFamily: 'monospace',
                      },
                    },
                    '& .MuiFormHelperText-root': {
                      color: 'rgba(33, 150, 243, 0.6)',
                      fontSize: 11,
                    },
                  }}
                />
              )}

              {/* Target Location */}
              <FormControl component="fieldset" fullWidth sx={{ mb: 2 }}>
                <FormLabel
                  sx={{
                    color: anafisColors.spreadsheet,
                    mb: 1,
                    fontWeight: 'bold',
                    fontSize: 11,
                    textTransform: 'uppercase',
                    '&.Mui-focused': { color: anafisColors.spreadsheet },
                    '&.Mui-active': { color: anafisColors.spreadsheet },
                  }}
                >
                  Import Location
                </FormLabel>
                <RadioGroup
                  value={targetMode}
                  onChange={(e) => setTargetMode(e.target.value as ImportTargetMode)}
                >
                  <FormControlLabel
                    value="newSheet"
                    control={
                      <Radio
                        sx={{
                          color: anafisColors.spreadsheet,
                          '&.Mui-checked': { color: anafisColors.spreadsheet },
                        }}
                      />
                    }
                    label="New Sheet"
                    sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                  />
                  <FormControlLabel
                    value="currentRange"
                    control={
                      <Radio
                        sx={{
                          color: anafisColors.spreadsheet,
                          '&.Mui-checked': { color: anafisColors.spreadsheet },
                        }}
                      />
                    }
                    label="Current Sheet at Range"
                    sx={{ color: 'rgba(255, 255, 255, 0.9)' }}
                  />
                </RadioGroup>

                {targetMode === 'currentRange' && (
                  <>
                    <TextField
                      fullWidth
                      size="small"
                      value={targetRange}
                      onChange={(e) => setTargetRange(e.target.value)}
                      onFocus={() => onInputFocus('targetRange')}
                      onBlur={onInputBlur}
                      placeholder="A1"
                      helperText="Click to select range in spreadsheet"
                      sx={{
                        mt: 1,
                        '& .MuiOutlinedInput-root': {
                          bgcolor:
                            focusedInput === 'targetRange'
                              ? 'rgba(33, 150, 243, 0.1)'
                              : 'rgba(33, 150, 243, 0.05)',
                          borderRadius: '6px',
                          '& fieldset': {
                            borderColor:
                              focusedInput === 'targetRange'
                                ? anafisColors.spreadsheet
                                : 'rgba(33, 150, 243, 0.2)',
                          },
                          '&:hover fieldset': {
                            borderColor: 'rgba(33, 150, 243, 0.4)',
                          },
                          '&.Mui-focused fieldset': {
                            borderColor: anafisColors.spreadsheet,
                          },
                          '& input': {
                            color: 'white',
                            fontFamily: 'monospace',
                            fontSize: 13,
                          },
                        },
                        '& .MuiFormHelperText-root': {
                          color: 'rgba(33, 150, 243, 0.6)',
                          fontSize: 11,
                        },
                      }}
                    />

                    {/* Range Validation Display */}
                    {rangeValidation && <RangeValidationWarning validation={rangeValidation} />}
                  </>
                )}
              </FormControl>
            </SidebarCard>
          )}

          {/* Step 4: Import Actions */}
          <SidebarCard title="4. Import Data" defaultExpanded={true}>
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

            <Button
              fullWidth
              startIcon={
                isImporting ? (
                  <CircularProgress size={20} sx={{ color: 'white' }} />
                ) : (
                  <FileUploadIcon />
                )
              }
              onClick={() => void handleImport()}
              disabled={isImporting}
              sx={sidebarStyles.button.primary}
            >
              {isImporting ? 'Importing...' : 'Import Data'}
            </Button>
          </SidebarCard>
        </>
      )}
    </>
  );
};
