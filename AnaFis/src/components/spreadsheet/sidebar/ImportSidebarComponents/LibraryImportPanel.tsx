import React, { useState, useCallback, useEffect } from 'react';
import {
  Box,
  Typography,
  TextField,
  Button,
  Alert,
  RadioGroup,
  FormLabel,
  CircularProgress,
  Chip,
  Checkbox,
  FormControlLabel,
  Radio,
} from '@mui/material';
import FileUploadIcon from '@mui/icons-material/FileUpload';
import { invoke } from '@tauri-apps/api/core';

import { sidebarStyles } from '@/utils/sidebarStyles';
import SidebarCard from '../SidebarCard';
import { anafisColors } from '@/themes';
import { RangeValidationWarning } from './RangeValidationWarning';
import { useImportValidation } from './useImportValidation';
import { extractStartCell } from '../../univer/utils/rangeUtils';

import type { CellValue, SpreadsheetRef as SpreadsheetInterface } from '@/components/spreadsheet/SpreadsheetInterface';
import type { ImportResult } from '@/types/import';
import type { DataSequence } from '@/types/dataLibrary';

interface LibraryImportPanelProps {
  spreadsheetRef: React.RefObject<SpreadsheetInterface | null>;
  libraryDataRange: string;
  setLibraryDataRange: (range: string) => void;
  libraryUncertaintyRange: string;
  setLibraryUncertaintyRange: (range: string) => void;
  onInputFocus: (field: 'libraryDataRange' | 'libraryUncertaintyRange') => void;
  onInputBlur: () => void;
  focusedInput: 'targetRange' | 'libraryDataRange' | 'libraryUncertaintyRange' | null;
}

/**
 * Library Import Panel Component
 * Handles all data library import functionality including sequence selection, search, filter, and range configuration
 */
export const LibraryImportPanel: React.FC<LibraryImportPanelProps> = ({
  spreadsheetRef,
  libraryDataRange,
  setLibraryDataRange,
  libraryUncertaintyRange,
  setLibraryUncertaintyRange,
  onInputFocus,
  onInputBlur,
  focusedInput,
}) => {
  const { validateLibraryDataRange, validateLibraryUncertaintyRange } = useImportValidation();

  // Data Library import state
  const [availableSequences, setAvailableSequences] = useState<DataSequence[]>([]);
  const [selectedSequence, setSelectedSequence] = useState<string | null>(null);
  const [libraryDataRangeValidation, setLibraryDataRangeValidation] = useState<ImportResult['rangeValidation'] | null>(null);
  const [libraryUncertaintyRangeValidation, setLibraryUncertaintyRangeValidation] = useState<ImportResult['rangeValidation'] | null>(null);
  const [includeUncertainties, setIncludeUncertainties] = useState(true);
  const [isLoadingSequences, setIsLoadingSequences] = useState(false);

  // Data Library search/filter state
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [availableTags, setAvailableTags] = useState<string[]>([]);

  // UI state
  const [isImporting, setIsImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Load sequences from data library
  const loadSequences = useCallback(async () => {
    setIsLoadingSequences(true);
    setError(null);

    try {
      const result = await invoke<{ sequences: DataSequence[]; total_count: number; pinned_count: number }>('get_sequences', {
        search: {
          query: searchQuery || null,
          tags: selectedTags.length > 0 ? selectedTags : null,
          source: null,
          sort_by: 'name',
          sort_order: 'ascending'
        }
      });

      setAvailableSequences(result.sequences);
    } catch (err) {
      setError(`Failed to load sequences: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsLoadingSequences(false);
    }
  }, [searchQuery, selectedTags]);

  // Load available tags
  const loadTags = useCallback(async () => {
    try {
      const tags = await invoke<string[]>('get_all_tags');
      setAvailableTags(tags);
    } catch (err) {
      console.error('Failed to load tags:', err);
    }
  }, []);

  // Load sequences and tags on mount
  useEffect(() => {
    void loadSequences();
    void loadTags();
  }, [loadSequences, loadTags]);

  // Reload sequences when search/filter changes
  useEffect(() => {
    void loadSequences();
  }, [searchQuery, selectedTags, loadSequences]);

  // Handle import from data library
  const handleImportFromLibrary = useCallback(async () => {
    if (!selectedSequence) {
      setError('Please select a sequence to import');
      return;
    }

    setError(null);
    setSuccess(null);
    setIsImporting(true);

    try {
      const spreadsheetAPI = spreadsheetRef.current;
      if (!spreadsheetAPI) {
        setError('Spreadsheet not ready');
        return;
      }

      // Get the selected sequence
      const sequence = availableSequences.find(s => s.id === selectedSequence);
      if (!sequence) {
        setError('Selected sequence not found');
        return;
      }

      // Import data to the data range
      const dataStartCell = extractStartCell(libraryDataRange);
      const dataArray: CellValue[][] = sequence.data.map(val => [{ v: val }]);

      // Apply data range truncation if needed
      let dataToImport = dataArray;
      if (libraryDataRangeValidation?.willTruncate && libraryDataRangeValidation.selectedRange) {
        dataToImport = dataArray.slice(0, libraryDataRangeValidation.selectedRange.rows);
      }

      await spreadsheetAPI.updateRange(dataStartCell, dataToImport);

      // Import uncertainties if included and available
      if (includeUncertainties && sequence.uncertainties && sequence.uncertainties.length > 0) {
        const uncertaintyStartCell = extractStartCell(libraryUncertaintyRange);
        const uncertaintyArray: CellValue[][] = sequence.uncertainties.map(val => [{ v: val }]);

        // Apply uncertainty range truncation if needed
        let uncertaintyToImport = uncertaintyArray;
        if (libraryUncertaintyRangeValidation?.willTruncate && libraryUncertaintyRangeValidation.selectedRange) {
          uncertaintyToImport = uncertaintyArray.slice(0, libraryUncertaintyRangeValidation.selectedRange.rows);
        }

        await spreadsheetAPI.updateRange(uncertaintyStartCell, uncertaintyToImport);
      }

      setSuccess(`✅ Imported "${sequence.name}" (${sequence.data.length} data points${includeUncertainties && sequence.uncertainties ? `, ${sequence.uncertainties.length} uncertainties` : ''})`);
    } catch (err) {
      setError(`Import failed: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsImporting(false);
    }
  }, [selectedSequence, availableSequences, includeUncertainties, libraryDataRange, libraryDataRangeValidation, libraryUncertaintyRange, libraryUncertaintyRangeValidation, spreadsheetRef]);

  // Get selected sequence
  const selectedSeq = selectedSequence ? availableSequences.find(s => s.id === selectedSequence) : undefined;

  // Validate library ranges when sequence or ranges change
  useEffect(() => {
    const validation = validateLibraryDataRange(libraryDataRange, selectedSeq);
    setLibraryDataRangeValidation(validation);
  }, [validateLibraryDataRange, libraryDataRange, selectedSeq]);

  useEffect(() => {
    const validation = validateLibraryUncertaintyRange(libraryUncertaintyRange, selectedSeq, includeUncertainties);
    setLibraryUncertaintyRangeValidation(validation);
  }, [validateLibraryUncertaintyRange, libraryUncertaintyRange, selectedSeq, includeUncertainties]);

  return (
    <>
      {/* Search and Filter */}
      <SidebarCard title="Search & Filter" defaultExpanded={false}>
        <TextField
          fullWidth
          size="small"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search by name..."
          sx={{
            mb: 2,
            '& .MuiOutlinedInput-root': {
              bgcolor: 'rgba(33, 150, 243, 0.05)',
              borderRadius: '6px',
              '& fieldset': { borderColor: anafisColors.ui.border },
              '&:hover fieldset': {
                borderColor: 'rgba(33, 150, 243, 0.4)',
              },
              '&.Mui-focused fieldset': {
                borderColor: anafisColors.spreadsheet,
              },
              '& input': {
                color: anafisColors.ui.text.primary,
                fontSize: 13,
              },
            },
          }}
        />

        {/* Tag Filter */}
        {availableTags.length > 0 && (
          <Box>
            <FormLabel
              sx={{
                color: anafisColors.spreadsheet,
                mb: 1,
                fontWeight: 'bold',
                fontSize: 11,
                textTransform: 'uppercase',
                letterSpacing: 0.5,
                display: 'block',
              }}
            >
              Filter by Tags
            </FormLabel>
            <Box
              sx={{
                display: 'flex',
                flexWrap: 'wrap',
                gap: 0.5,
                maxHeight: '120px',
                overflowY: 'auto',
                overflowX: 'hidden',
                pr: 0.5,
                '&::-webkit-scrollbar': { width: '6px' },
                '&::-webkit-scrollbar-track': {
                  bgcolor: 'rgba(255, 255, 255, 0.05)',
                  borderRadius: '3px',
                },
                '&::-webkit-scrollbar-thumb': {
                  bgcolor: 'rgba(33, 150, 243, 0.3)',
                  borderRadius: '3px',
                  '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.5)' },
                },
              }}
            >
              {availableTags.map((tag) => (
                <Chip
                  key={tag}
                  label={tag}
                  size="small"
                  color={selectedTags.includes(tag) ? 'primary' : 'default'}
                  onClick={() => {
                    setSelectedTags((prev) =>
                      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag]
                    );
                  }}
                  sx={{
                    cursor: 'pointer',
                    fontSize: 11,
                    '&.MuiChip-colorPrimary': {
                      bgcolor: anafisColors.spreadsheet,
                      color: anafisColors.ui.text.primary,
                    },
                    '&.MuiChip-colorDefault': {
                      bgcolor: anafisColors.ui.paper,
                      color: anafisColors.ui.text.secondary,
                      border: `1px solid ${anafisColors.ui.border}`,
                      '&:hover': {
                        bgcolor: 'rgba(255, 255, 255, 0.08)',
                        borderColor: 'rgba(255, 255, 255, 0.15)',
                      },
                    },
                  }}
                />
              ))}
            </Box>
          </Box>
        )}

        <Typography
          sx={{
            color: anafisColors.ui.text.tertiary,
            fontSize: 11,
            mt: 2,
            fontStyle: 'italic',
          }}
        >
          {availableSequences.length} sequence
          {availableSequences.length !== 1 ? 's' : ''} found
        </Typography>
      </SidebarCard>

      {/* Step 1: Select Sequence */}
      <SidebarCard title="1. Select Sequence" defaultExpanded={true}>
        {isLoadingSequences ? (
          <Box sx={{ display: 'flex', justifyContent: 'center', py: 3 }}>
            <CircularProgress size={30} sx={{ color: anafisColors.spreadsheet }} />
          </Box>
        ) : availableSequences.length === 0 ? (
          <Typography
            sx={{
              color: anafisColors.ui.text.tertiary,
              fontSize: 12,
              textAlign: 'center',
              fontStyle: 'italic',
              py: 2,
            }}
          >
            No sequences available in data library
          </Typography>
        ) : (
          <RadioGroup
            value={selectedSequence ?? ''}
            onChange={(e) => setSelectedSequence(e.target.value)}
            sx={{
              maxHeight: '400px',
              overflowY: 'auto',
              overflowX: 'hidden',
              pr: 0.5,
              '&::-webkit-scrollbar': { width: '6px' },
              '&::-webkit-scrollbar-track': {
                bgcolor: 'rgba(255, 255, 255, 0.05)',
                borderRadius: '3px',
              },
              '&::-webkit-scrollbar-thumb': {
                bgcolor: 'rgba(33, 150, 243, 0.3)',
                borderRadius: '3px',
                '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.5)' },
              },
            }}
          >
            {availableSequences.map((seq) => (
              <Box
                key={seq.id}
                sx={{
                  display: 'flex',
                  alignItems: 'flex-start',
                  mb: 1,
                  p: 1,
                  borderRadius: '6px',
                  bgcolor:
                    selectedSequence === seq.id
                      ? 'rgba(33, 150, 243, 0.15)'
                      : anafisColors.ui.paper,
                  border: '1px solid',
                  borderColor:
                    selectedSequence === seq.id
                      ? 'rgba(33, 150, 243, 0.4)'
                      : anafisColors.ui.border,
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                  '&:hover': {
                    bgcolor:
                      selectedSequence === seq.id
                        ? 'rgba(33, 150, 243, 0.15)'
                        : 'rgba(255, 255, 255, 0.05)',
                    borderColor:
                      selectedSequence === seq.id
                        ? 'rgba(33, 150, 243, 0.4)'
                        : 'rgba(255, 255, 255, 0.15)',
                  },
                }}
                onClick={() => setSelectedSequence(seq.id)}
              >
                <Radio
                  value={seq.id}
                  sx={{
                    color: 'rgba(33, 150, 243, 0.5)',
                    '&.Mui-checked': {
                      color: anafisColors.spreadsheet,
                    },
                    py: 0.5,
                    pr: 1,
                  }}
                />
                <Box sx={{ flex: 1, pt: 0.5 }}>
                  <Typography
                    sx={{
                      fontSize: 13,
                      fontWeight: 500,
                      color: anafisColors.ui.text.primary,
                      mb: 0.5,
                    }}
                  >
                    {seq.name}
                  </Typography>
                  <Typography
                    sx={{
                      fontSize: 11,
                      color: anafisColors.ui.text.tertiary,
                    }}
                  >
                    {seq.data.length} points
                    {seq.uncertainties ? ` • ${seq.uncertainties.length} uncertainties` : ''}
                    {seq.unit ? ` • ${seq.unit}` : ''}
                  </Typography>
                  {seq.tags.length > 0 && (
                    <Box
                      sx={{
                        display: 'flex',
                        flexWrap: 'wrap',
                        gap: 0.5,
                        mt: 0.5,
                      }}
                    >
                      {seq.tags.map((tag) => (
                        <Chip
                          key={tag}
                          label={tag}
                          size="small"
                          sx={{
                            height: 18,
                            fontSize: 10,
                            bgcolor: 'rgba(33, 150, 243, 0.2)',
                            color: 'rgba(33, 150, 243, 0.9)',
                            '& .MuiChip-label': { px: 1 },
                          }}
                        />
                      ))}
                    </Box>
                  )}
                </Box>
              </Box>
            ))}
          </RadioGroup>
        )}
      </SidebarCard>

      {/* Step 2: Configure Import */}
      <SidebarCard title="2. Configure Import" defaultExpanded={true}>
        {/* Data Range */}
        <Box sx={{ mb: 2 }}>
          <FormLabel
            sx={{
              color: anafisColors.spreadsheet,
              mb: 1,
              fontWeight: 'bold',
              fontSize: 11,
              textTransform: 'uppercase',
              letterSpacing: 0.5,
            }}
          >
            Data Range
          </FormLabel>
          <TextField
            fullWidth
            size="small"
            value={libraryDataRange}
            onChange={(e) => setLibraryDataRange(e.target.value)}
            onFocus={() => onInputFocus('libraryDataRange')}
            onBlur={onInputBlur}
            placeholder="A1"
            helperText="Starting cell or range where data will be placed"
            sx={{
              mt: 0.5,
              '& .MuiOutlinedInput-root': {
                bgcolor:
                  focusedInput === 'libraryDataRange'
                    ? 'rgba(33, 150, 243, 0.1)'
                    : 'rgba(33, 150, 243, 0.05)',
                borderRadius: '6px',
                '& fieldset': {
                  borderColor:
                    focusedInput === 'libraryDataRange'
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
        </Box>

        {/* Data Range Validation */}
        {libraryDataRangeValidation && (
          <RangeValidationWarning validation={libraryDataRangeValidation} />
        )}

        {/* Include Uncertainties */}
        <FormControlLabel
          control={
            <Checkbox
              checked={includeUncertainties}
              onChange={(e) => setIncludeUncertainties(e.target.checked)}
              sx={{
                color: anafisColors.spreadsheet,
                '&.Mui-checked': {
                  color: anafisColors.spreadsheet,
                },
              }}
            />
          }
          label="Include Uncertainties"
          sx={{
            color: 'rgba(255, 255, 255, 0.9)',
            mb: includeUncertainties && selectedSeq?.uncertainties ? 2 : 0,
            display: selectedSeq?.uncertainties ? 'flex' : 'none',
          }}
        />

        {/* Uncertainty Range */}
        {includeUncertainties && selectedSeq?.uncertainties && (
          <Box sx={{ mb: 2 }}>
            <FormLabel
              sx={{
                color: anafisColors.spreadsheet,
                mb: 1,
                fontWeight: 'bold',
                fontSize: 11,
                textTransform: 'uppercase',
                letterSpacing: 0.5,
              }}
            >
              Uncertainty Range
            </FormLabel>
            <TextField
              fullWidth
              size="small"
              value={libraryUncertaintyRange}
              onChange={(e) => setLibraryUncertaintyRange(e.target.value)}
              onFocus={() => onInputFocus('libraryUncertaintyRange')}
              onBlur={onInputBlur}
              placeholder="B1"
              helperText="Starting cell or range where uncertainties will be placed"
              sx={{
                mt: 0.5,
                '& .MuiOutlinedInput-root': {
                  bgcolor:
                    focusedInput === 'libraryUncertaintyRange'
                      ? 'rgba(33, 150, 243, 0.1)'
                      : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': {
                    borderColor:
                      focusedInput === 'libraryUncertaintyRange'
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
          </Box>
        )}

        {/* Uncertainty Range Validation */}
        {libraryUncertaintyRangeValidation &&
          includeUncertainties &&
          selectedSeq?.uncertainties && (
            <RangeValidationWarning validation={libraryUncertaintyRangeValidation} />
          )}
      </SidebarCard>

      {/* Step 3: Import */}
      <SidebarCard title="3. Import" defaultExpanded={true}>
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
          onClick={() => void handleImportFromLibrary()}
          disabled={isImporting || !selectedSequence}
          sx={sidebarStyles.button.primary}
        >
          {isImporting ? 'Importing...' : 'Import from Library'}
        </Button>
      </SidebarCard>
    </>
  );
};
