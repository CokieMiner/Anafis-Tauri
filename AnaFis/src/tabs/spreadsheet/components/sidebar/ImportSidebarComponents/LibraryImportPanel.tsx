import React, { useState, useCallback, useEffect } from 'react';
import {
  Box,
  Typography,
  TextField,
  Button,
  Alert,
  FormLabel,
  CircularProgress,
  Chip,
  Checkbox,
  FormControlLabel,
  ListItemButton,
  ListItemText,
} from '@mui/material';
import FileUploadIcon from '@mui/icons-material/FileUpload';
import { invoke } from '@tauri-apps/api/core';

import { sidebarStyles } from '@/tabs/spreadsheet/components/sidebar/utils/sidebarStyles';
import SidebarCard from '@/tabs/spreadsheet/components/sidebar/SidebarCard';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';
import { RangeValidationWarning } from '@/tabs/spreadsheet/components/sidebar/ImportSidebarComponents/RangeValidationWarning';
import { useImportValidation } from '@/tabs/spreadsheet/components/sidebar/logic/useImportValidation';
import { extractStartCell } from '@/tabs/spreadsheet/utils/rangeUtils';

import type { CellValue, SpreadsheetRef as SpreadsheetInterface } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import type { ImportResult } from '@/core/types/import';
import type { DataSequence } from '@/core/types/dataLibrary';
import { type ErrorResponse, isErrorResponse, getErrorMessage } from '@/core/types/error';

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
      const result = await invoke<{ sequences: DataSequence[]; total_count: number; pinned_count: number } | ErrorResponse>('get_sequences', {
        search: {
          query: searchQuery || null,
          tags: selectedTags.length > 0 ? selectedTags : null,
          source: null,
          sort_by: 'name',
          sort_order: 'ascending',
          page: 0,
          page_size: 10000,
        }
      });

      if (isErrorResponse(result)) {
        setError(getErrorMessage(result));
        return;
      }

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
      const tags = await invoke<string[] | ErrorResponse>('get_all_tags');
      if (isErrorResponse(tags)) {
        setError(getErrorMessage(tags));
        return;
      }
      setAvailableTags(tags);
    } catch (err) {
      console.error('Failed to load tags:', err);
    }
  }, []);

  // Load tags on mount
  useEffect(() => {
    void loadTags();
  }, [loadTags]);

  // Load/reload sequences when search/filter changes
  useEffect(() => {
    void loadSequences();
  }, [loadSequences]);

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
          <Box sx={{ 
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            overflowY: 'auto',
            minHeight: 0,
            maxHeight: '200px', // Limit to show about 3 sequences
            pr: 0.5,
            backgroundColor: 'transparent',
            /* webkit-based scrollbar styling */
            '&::-webkit-scrollbar': {
              width: '8px',
            },
            '&::-webkit-scrollbar-track': {
              backgroundColor: 'rgba(255, 255, 255, 0.1)',
              borderRadius: '4px',
            },
            '&::-webkit-scrollbar-thumb': {
              backgroundColor: 'rgba(255, 255, 255, 0.3)',
              borderRadius: '4px',
              '&:hover': {
                backgroundColor: 'rgba(255, 255, 255, 0.5)',
              },
            },
            /* Firefox scrollbar */
            scrollbarWidth: 'thin' as const,
            scrollbarColor: 'rgba(255,255,255,0.3) rgba(255,255,255,0.1)',
            /* Reserve gutter where supported to keep content visible when overlay scrollbars appear */
            scrollbarGutter: 'stable',
          }}>
            {availableSequences.map((seq) => (
              <ListItemButton
                key={seq.id}
                selected={selectedSequence === seq.id}
                onClick={() => setSelectedSequence(seq.id)}
                sx={{
                  flexShrink: 0, // Prevent expansion to fill space
                  maxHeight: '60px', // Compact height to fit 3 sequences
                  px: 1,
                  py: 0.75,
                  mb: 0.5,
                  borderRadius: '6px',
                  border: selectedSequence === seq.id ? `1px solid ${anafisColors.spreadsheet}` : '1px solid rgba(255, 255, 255, 0.2)',
                  bgcolor: selectedSequence === seq.id ? 'rgba(33, 150, 243, 0.15)' : 'transparent',
                  color: selectedSequence === seq.id ? '#ffffff' : 'rgba(255, 255, 255, 0.7)',
                  transition: 'all 0.2s',
                  '&:hover': {
                    bgcolor: selectedSequence === seq.id ? 'rgba(33, 150, 243, 0.2)' : 'rgba(255, 255, 255, 0.05)',
                    borderColor: selectedSequence === seq.id ? anafisColors.spreadsheet : 'rgba(255, 255, 255, 0.4)',
                    color: '#ffffff',
                    transform: 'translateY(-1px)',
                    boxShadow: selectedSequence === seq.id ? `0 2px 8px rgba(33, 150, 243, 0.3)` : '0 2px 8px rgba(255, 255, 255, 0.1)'
                  },
                  '&.Mui-selected': {
                    bgcolor: 'rgba(33, 150, 243, 0.15) !important',
                    borderColor: `${anafisColors.spreadsheet} !important`,
                    color: '#ffffff !important',
                    '&:hover': {
                      bgcolor: 'rgba(33, 150, 243, 0.2) !important'
                    }
                  }
                }}
              >
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: 0.25 }}>
                      <Typography component="span" sx={{ fontSize: 14, fontWeight: 600, color: 'inherit' }}>
                        {seq.name}
                      </Typography>
                      <Typography variant="body2" sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.6)', lineHeight: 1.2 }}>
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
                            mt: 0.25,
                          }}
                        >
                          {seq.tags.map((tag) => (
                            <Chip
                              key={tag}
                              label={tag}
                              size="small"
                              sx={{
                                height: 16,
                                fontSize: 9,
                                bgcolor: 'rgba(33, 150, 243, 0.2)',
                                color: 'rgba(33, 150, 243, 0.9)',
                                '& .MuiChip-label': { px: 0.75 },
                              }}
                            />
                          ))}
                        </Box>
                      )}
                    </Box>
                  }
                />
              </ListItemButton>
            ))}
          </Box>
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
