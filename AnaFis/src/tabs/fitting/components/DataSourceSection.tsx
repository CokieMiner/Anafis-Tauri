// § 1. Data Source Section — Library/CSV selector with mini preview grid

import { FolderOpen, LibraryBooks, Settings } from '@mui/icons-material';
import {
  Alert,
  Autocomplete,
  Box,
  Button,
  Chip,
  IconButton,
  Paper,
  type PaperProps,
  TextField,
  ToggleButton,
  ToggleButtonGroup,
  Typography,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useMemo, useState } from 'react';
import type { DataSequence } from '@/core/types/dataLibrary';
import { anafisTheme } from '@/shared/theme/unifiedTheme';
import { openWithMemory } from '@/shared/utils/dialogMemory';
import CsvSettingsDialog from '@/tabs/fitting/components/CsvSettingsDialog';
import DataPreviewTable from '@/tabs/fitting/components/DataPreviewTable';
import {
  type CsvImportSettings,
  type DataSourceMode,
  DEFAULT_CSV_SETTINGS,
  type ImportedData,
} from '@/tabs/fitting/types/fittingTypes';
import { parseCsvText } from '@/tabs/fitting/utils/csvParser';

interface DataSourceSectionProps {
  mode: DataSourceMode;
  importedData: ImportedData | null;
  librarySequences: DataSequence[];
  isLibraryLoading?: boolean;
  onRefreshLibrarySequences?: () => void;
  onModeChange: (mode: DataSourceMode) => void;
  onDataImported: (data: ImportedData | null) => void;
}

const DROPDOWN_MAX_HEIGHT = 300;

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
    background: `${anafisTheme.colors.tabs.fitting.main}40`,
    borderRadius: 2,
  },
};

const amberInputSx = {
  '& .MuiOutlinedInput-root': {
    backgroundColor: '#0f0f14',
    backgroundImage: 'none',
    '& fieldset': { borderColor: `${anafisTheme.colors.tabs.fitting.main}38` },
    '&:hover fieldset': {
      borderColor: `${anafisTheme.colors.tabs.fitting.main}61`,
    },
    '&.Mui-focused fieldset': {
      borderColor: anafisTheme.colors.tabs.fitting.main,
    },
  },
  '& .MuiInputLabel-root': { color: 'rgba(255,255,255,0.68)' },
  '& .MuiInputLabel-root.Mui-focused': {
    color: anafisTheme.colors.tabs.fitting.main,
  },
  '& .MuiInputBase-input': { caretColor: anafisTheme.colors.tabs.fitting.main },
  '& .MuiSelect-icon': { color: `${anafisTheme.colors.tabs.fitting.main}CC` },
};

function SolidPaper(props: PaperProps) {
  return (
    <Paper
      {...props}
      sx={{
        ...((props.sx ?? {}) as object),
        backgroundColor: '#0f0f14 !important',
        backgroundImage: 'none !important',
        border: `1px solid ${anafisTheme.colors.tabs.fitting.main}26`,
        boxShadow: anafisTheme.shadows.lg,
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
  isLibraryLoading = false,
  onRefreshLibrarySequences,
  onModeChange,
  onDataImported,
}: DataSourceSectionProps) {
  const [csvSettings, setCsvSettings] =
    useState<CsvImportSettings>(DEFAULT_CSV_SETTINGS);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [selectedLibrarySequenceIds, setSelectedLibrarySequenceIds] = useState<
    string[]
  >([]);

  const selectedLibrarySequences = useMemo(
    () =>
      selectedLibrarySequenceIds
        .map((id) => librarySequences.find((seq) => seq.id === id))
        .filter((seq): seq is DataSequence => Boolean(seq)),
    [selectedLibrarySequenceIds, librarySequences]
  );

  const buildImportedDataFromSequences = useCallback(
    (sequences: DataSequence[]) => {
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

  const handleLibrarySelect = useCallback(
    (_: unknown, sequences: DataSequence[]) => {
      setSelectedLibrarySequenceIds(sequences.map((seq) => seq.id));
      buildImportedDataFromSequences(sequences);
    },
    [buildImportedDataFromSequences]
  );

  // Keep selected IDs and imported data in sync when library contents change.
  useEffect(() => {
    if (mode !== 'library' || selectedLibrarySequenceIds.length === 0) {
      return;
    }

    const nextSelected = selectedLibrarySequenceIds
      .map((id) => librarySequences.find((seq) => seq.id === id))
      .filter((seq): seq is DataSequence => Boolean(seq));

    if (nextSelected.length !== selectedLibrarySequenceIds.length) {
      setSelectedLibrarySequenceIds(nextSelected.map((seq) => seq.id));
    }

    buildImportedDataFromSequences(nextSelected);
  }, [
    mode,
    selectedLibrarySequenceIds,
    librarySequences,
    buildImportedDataFromSequences,
  ]);

  const handleCsvImport = useCallback(async () => {
    try {
      const filePath = await openWithMemory({
        filters: [{ name: 'CSV', extensions: ['csv', 'tsv', 'txt', 'dat'] }],
      });

      if (typeof filePath !== 'string') {
        return;
      }

      const filePathStr = filePath as string;

      const text = await invoke<string>('read_file_text', {
        path: filePathStr,
      });
      const parsed = parseCsvText(
        text,
        csvSettings,
        filePathStr.split('/').pop() ?? 'CSV'
      );

      setImportError(null);
      onDataImported(parsed);
    } catch (error) {
      onDataImported(null);
      setImportError(error instanceof Error ? error.message : String(error));
    }
  }, [csvSettings, onDataImported]);

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
          value={selectedLibrarySequences}
          loading={isLibraryLoading}
          getOptionLabel={(seq) => seq.name}
          isOptionEqualToValue={(option, value) => option.id === value.id}
          onOpen={() => onRefreshLibrarySequences?.()}
          onChange={handleLibrarySelect}
          renderInput={(params) => (
            <TextField
              {...params}
              fullWidth
              size="small"
              placeholder="Search sequences..."
              variant="outlined"
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
          slotProps={{
            paper: {
              sx: {
                maxHeight: DROPDOWN_MAX_HEIGHT,
                overflowY: 'auto',
              },
            },
            listbox: {
              sx: {
                maxHeight: DROPDOWN_MAX_HEIGHT,
                overflowY: 'auto',
              },
            },
          }}
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
        <DataPreviewTable importedData={importedData} />
      )}

      <CsvSettingsDialog
        open={advancedOpen}
        settings={csvSettings}
        onClose={() => setAdvancedOpen(false)}
        onSettingsChange={setCsvSettings}
      />
    </Box>
  );
}
