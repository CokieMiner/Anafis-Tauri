// § 1. Data Source Section — Library/CSV selector with mini preview grid

import { FolderOpen, LibraryBooks, Settings } from '@mui/icons-material';
import {
  Alert,
  Autocomplete,
  type AutocompleteRenderInputParams,
  Box,
  Button,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControl,
  IconButton,
  InputLabel,
  MenuItem,
  Paper,
  type PaperProps,
  Select,
  TextField,
  ToggleButton,
  ToggleButtonGroup,
  Typography,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useCallback, useState } from 'react';
import type { DataSequence } from '@/core/types/dataLibrary';
import {
  type CsvImportSettings,
  type DataSourceMode,
  DEFAULT_CSV_SETTINGS,
  type ImportedData,
} from '../types/fittingTypes';
import { parseCsvText } from '../utils/csvParser';

interface DataSourceSectionProps {
  mode: DataSourceMode;
  importedData: ImportedData | null;
  librarySequences: DataSequence[];
  onModeChange: (mode: DataSourceMode) => void;
  onDataImported: (data: ImportedData | null) => void;
}

const sectionSx = {
  mb: 2,
  p: 1.5,
  borderRadius: 1.5,
  border: '1px solid rgba(148, 163, 184, 0.12)',
  background: 'rgba(255,255,255,0.02)',
  maxHeight: 320,
  overflow: 'auto',
  '&::-webkit-scrollbar': { width: 4 },
  '&::-webkit-scrollbar-thumb': {
    background: 'rgba(255,179,0,0.25)',
    borderRadius: 2,
  },
};

const amberInputSx = {
  '& .MuiOutlinedInput-root': {
    backgroundColor: '#0f0f14',
    backgroundImage: 'none',
    '& fieldset': { borderColor: 'rgba(255,179,0,0.22)' },
    '&:hover fieldset': { borderColor: 'rgba(255,179,0,0.38)' },
    '&.Mui-focused fieldset': { borderColor: '#ffb300' },
  },
  '& .MuiInputLabel-root': { color: 'rgba(255,255,255,0.68)' },
  '& .MuiInputLabel-root.Mui-focused': { color: '#ffb300' },
  '& .MuiInputBase-input': { caretColor: '#ffb300' },
  '& .MuiSelect-icon': { color: 'rgba(255,179,0,0.8)' },
};

const MAX_PREVIEW_ROWS = 5;

function SolidPaper(props: PaperProps) {
  return (
    <Paper
      {...props}
      sx={{
        ...((props.sx ?? {}) as object),
        backgroundColor: '#0f0f14 !important',
        backgroundImage: 'none !important',
        border: '1px solid rgba(255,179,0,0.15)',
        boxShadow: '0 8px 24px rgba(0,0,0,0.6)',
      }}
    />
  );
}

function validateFiniteValues(values: number[], label: string): void {
  for (let idx = 0; idx < values.length; idx++) {
    if (!Number.isFinite(values[idx])) {
      throw new Error(`${label} has a non-finite value at index ${idx}`);
    }
  }
}

function validateLibrarySelection(sequences: DataSequence[]): void {
  if (sequences.length === 0) {
    return;
  }

  const expectedLength = sequences[0]?.data.length ?? 0;

  for (const seq of sequences) {
    if (seq.data.length !== expectedLength) {
      throw new Error(
        `Sequence '${seq.name}' has ${seq.data.length} rows but expected ${expectedLength}`
      );
    }

    validateFiniteValues(seq.data, `Sequence '${seq.name}'`);

    if (seq.uncertainties && seq.uncertainties.length > 0) {
      if (seq.uncertainties.length !== seq.data.length) {
        throw new Error(
          `Uncertainty column for '${seq.name}' has ${seq.uncertainties.length} rows but expected ${seq.data.length}`
        );
      }
      validateFiniteValues(
        seq.uncertainties,
        `Uncertainty column for '${seq.name}'`
      );
    }
  }
}

export default function DataSourceSection({
  mode,
  importedData,
  librarySequences,
  onModeChange,
  onDataImported,
}: DataSourceSectionProps) {
  const [csvSettings, setCsvSettings] =
    useState<CsvImportSettings>(DEFAULT_CSV_SETTINGS);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);

  const handleLibrarySelect = useCallback(
    (_: unknown, sequences: DataSequence[]) => {
      try {
        if (sequences.length === 0) {
          setImportError(null);
          onDataImported(null);
          return;
        }

        validateLibrarySelection(sequences);

        const importedColumns = sequences.flatMap((seq) => {
          const cols: Array<{ name: string; data: number[] }> = [
            {
              name: seq.name,
              data: seq.data,
            },
          ];

          if (seq.uncertainties && seq.uncertainties.length > 0) {
            cols.push({ name: `σ(${seq.name})`, data: seq.uncertainties });
          }

          return cols;
        });

        setImportError(null);
        onDataImported({
          columns: importedColumns,
          sourceName: 'Library',
          rowCount: importedColumns[0]?.data.length ?? 0,
        });
      } catch (error) {
        onDataImported(null);
        setImportError(error instanceof Error ? error.message : String(error));
      }
    },
    [onDataImported]
  );

  const handleCsvImport = useCallback(async () => {
    try {
      const filePath = await open({
        multiple: false,
        filters: [{ name: 'CSV', extensions: ['csv', 'tsv', 'txt', 'dat'] }],
      });

      if (typeof filePath !== 'string') {
        return;
      }

      const text = await invoke<string>('read_file_text', { path: filePath });
      const parsed = parseCsvText(
        text,
        csvSettings,
        filePath.split('/').pop() ?? 'CSV'
      );

      setImportError(null);
      onDataImported(parsed);
    } catch (error) {
      onDataImported(null);
      setImportError(error instanceof Error ? error.message : String(error));
    }
  }, [csvSettings, onDataImported]);

  const previewRows = importedData
    ? Math.min(MAX_PREVIEW_ROWS, importedData.rowCount)
    : 0;

  return (
    <Box sx={sectionSx}>
      <Typography
        variant="subtitle2"
        sx={{ fontWeight: 700, mb: 1, color: 'primary.main' }}
      >
        1. Data Source
      </Typography>

      <ToggleButtonGroup
        value={mode}
        exclusive
        onChange={(_, nextMode: unknown) => {
          if (nextMode === 'library' || nextMode === 'csv') {
            setImportError(null);
            onModeChange(nextMode);
          }
        }}
        size="small"
        fullWidth
        sx={{ mb: 1.5 }}
      >
        <ToggleButton value="library">
          <LibraryBooks sx={{ mr: 0.5, fontSize: 16 }} /> Library
        </ToggleButton>
        <ToggleButton value="csv">
          <FolderOpen sx={{ mr: 0.5, fontSize: 16 }} /> CSV / File
        </ToggleButton>
      </ToggleButtonGroup>

      {mode === 'library' && (
        <Autocomplete
          multiple
          fullWidth
          size="small"
          options={librarySequences}
          getOptionLabel={(seq) => seq.name}
          onChange={handleLibrarySelect}
          renderInput={(params: AutocompleteRenderInputParams) => (
            <TextField
              fullWidth
              size="small"
              placeholder="Search sequences..."
              variant="outlined"
              inputRef={params.InputProps.ref}
              InputProps={params.InputProps}
              inputProps={params.inputProps}
              sx={amberInputSx}
            />
          )}
          renderValue={(value, getTagProps) =>
            value.map((option, index) => (
              <Chip
                {...getTagProps({ index })}
                key={option.id}
                label={option.name}
                size="small"
              />
            ))
          }
          slots={{ paper: SolidPaper }}
          disablePortal
          sx={{ mb: 1, position: 'relative', width: '100%' }}
        />
      )}

      {mode === 'csv' && (
        <Box sx={{ display: 'flex', gap: 1, mb: 1 }}>
          <Button
            variant="outlined"
            size="small"
            startIcon={<FolderOpen />}
            onClick={() => {
              void handleCsvImport();
            }}
            sx={{ flex: 1 }}
          >
            Open File
          </Button>
          <IconButton
            size="small"
            onClick={() => setAdvancedOpen(true)}
            title="Import settings"
          >
            <Settings fontSize="small" />
          </IconButton>
        </Box>
      )}

      {importError && (
        <Alert severity="error" sx={{ mb: 1, py: 0 }}>
          {importError}
        </Alert>
      )}

      {importedData && importedData.columns.length > 0 && (
        <Box sx={{ mt: 1 }}>
          <Typography
            variant="caption"
            color="text.secondary"
            sx={{ mb: 0.5, display: 'block' }}
          >
            {importedData.rowCount} rows · {importedData.columns.length} cols (
            {importedData.sourceName})
          </Typography>

          <Box
            sx={{
              overflow: 'auto',
              maxHeight: 140,
              borderRadius: 1,
              border: '1px solid rgba(148, 163, 184, 0.12)',
              background: 'rgba(0,0,0,0.2)',
              '&::-webkit-scrollbar': { width: 4, height: 4 },
              '&::-webkit-scrollbar-thumb': {
                background: 'rgba(255,179,0,0.25)',
                borderRadius: 2,
              },
            }}
          >
            <Box
              component="table"
              sx={{
                width: '100%',
                borderCollapse: 'collapse',
                fontSize: '0.7rem',
                fontFamily: 'monospace',
                tableLayout: 'fixed',
              }}
            >
              <Box component="thead">
                <Box component="tr">
                  {importedData.columns.map((col) => (
                    <Box
                      component="th"
                      key={`th-${col.name}`}
                      sx={{
                        px: 0.75,
                        py: 0.4,
                        textAlign: 'left',
                        fontWeight: 700,
                        fontSize: '0.65rem',
                        color: col.name.startsWith('σ')
                          ? 'warning.dark'
                          : 'text.secondary',
                        borderBottom: '1px solid rgba(148,163,184,0.15)',
                        whiteSpace: 'nowrap',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        maxWidth: 90,
                      }}
                      title={col.name}
                    >
                      {col.name}
                    </Box>
                  ))}
                </Box>
              </Box>

              <Box component="tbody">
                {Array.from({ length: previewRows }, (_, rowIdx) => (
                  <Box
                    component="tr"
                    // biome-ignore lint/suspicious/noArrayIndexKey: Table row order is static
                    key={`preview-tr-${rowIdx}`}
                    sx={{
                      '&:hover': { background: 'rgba(255,179,0,0.04)' },
                    }}
                  >
                    {importedData.columns.map((col) => (
                      <Box
                        component="td"
                        key={`td-${col.name}`}
                        sx={{
                          px: 0.75,
                          py: 0.25,
                          color: 'text.secondary',
                          borderBottom: '1px solid rgba(255,255,255,0.03)',
                          whiteSpace: 'nowrap',
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          maxWidth: 90,
                        }}
                      >
                        {col.data[rowIdx] !== undefined
                          ? Number(col.data[rowIdx]).toPrecision(5)
                          : '—'}
                      </Box>
                    ))}
                  </Box>
                ))}
              </Box>
            </Box>
          </Box>
        </Box>
      )}

      <Dialog
        open={advancedOpen}
        onClose={() => setAdvancedOpen(false)}
        maxWidth="xs"
        fullWidth
        slotProps={{
          paper: {
            sx: {
              backgroundColor: '#0f0f14 !important',
              backgroundImage: 'none !important',
              opacity: 1,
              backdropFilter: 'none !important',
              mixBlendMode: 'normal',
              border: '1px solid rgba(255,179,0,0.15)',
              boxShadow: '0 8px 24px rgba(0,0,0,0.6)',
            },
          },
        }}
      >
        <DialogTitle>Import Settings</DialogTitle>
        <DialogContent
          sx={{
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
            pt: 2,
          }}
        >
          <FormControl size="small" fullWidth sx={[amberInputSx, { mt: 1 }]}>
            <InputLabel>Separator</InputLabel>
            <Select
              color="warning"
              value={csvSettings.separator}
              label="Separator"
              MenuProps={{
                PaperProps: {
                  sx: {
                    backgroundColor: '#1a1a22 !important',
                    backgroundImage: 'none !important',
                    opacity: 1,
                    backdropFilter: 'none !important',
                    mixBlendMode: 'normal',
                    color: 'rgba(255,255,255,0.95)',
                    '& .MuiMenuItem-root': { color: 'inherit' },
                    '& .MuiMenuItem-root:hover': {
                      backgroundColor: 'rgba(255,179,0,0.06)',
                    },
                    '& .MuiMenuItem-root.Mui-selected, & .MuiMenuItem-root.Mui-selected:hover':
                      {
                        backgroundColor: 'rgba(255,179,0,0.12) !important',
                        color: 'inherit',
                      },
                  },
                },
              }}
              onChange={(event) => {
                setCsvSettings((prev) => ({
                  ...prev,
                  separator: event.target.value,
                }));
              }}
            >
              <MenuItem value="auto">Auto-detect</MenuItem>
              <MenuItem value=",">Comma (,)</MenuItem>
              <MenuItem value=";">Semicolon (;)</MenuItem>
              <MenuItem value={'\t'}>Tab</MenuItem>
            </Select>
          </FormControl>

          <FormControl size="small" fullWidth sx={amberInputSx}>
            <InputLabel>Decimal</InputLabel>
            <Select
              color="warning"
              value={csvSettings.decimalFormat}
              label="Decimal"
              MenuProps={{
                PaperProps: {
                  sx: {
                    backgroundColor: '#1a1a22 !important',
                    backgroundImage: 'none !important',
                    opacity: 1,
                    backdropFilter: 'none !important',
                    mixBlendMode: 'normal',
                    color: 'rgba(255,255,255,0.95)',
                    '& .MuiMenuItem-root': { color: 'inherit' },
                    '& .MuiMenuItem-root:hover': {
                      backgroundColor: 'rgba(255,179,0,0.06)',
                    },
                    '& .MuiMenuItem-root.Mui-selected, & .MuiMenuItem-root.Mui-selected:hover':
                      {
                        backgroundColor: 'rgba(255,179,0,0.12) !important',
                        color: 'inherit',
                      },
                  },
                },
              }}
              onChange={(event) => {
                setCsvSettings((prev) => ({
                  ...prev,
                  decimalFormat: event.target.value,
                }));
              }}
            >
              <MenuItem value=".">Point (.)</MenuItem>
              <MenuItem value=",">Comma (,)</MenuItem>
            </Select>
          </FormControl>

          <TextField
            color="warning"
            size="small"
            type="number"
            label="Skip rows"
            value={csvSettings.skipRows}
            sx={amberInputSx}
            onChange={(event) => {
              setCsvSettings((prev) => ({
                ...prev,
                skipRows: Math.max(0, Number(event.target.value)),
              }));
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button color="warning" onClick={() => setAdvancedOpen(false)}>
            Close
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
