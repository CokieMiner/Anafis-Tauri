import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  Box,
  Typography,
  IconButton,
  TextField,
  Button,
  Alert,
  Chip,
  List,
  ListItemButton,
  ListItemText,
  CircularProgress
} from '@mui/material';
import {
  Close as CloseIcon,
  SwapHoriz as SwapIcon,
  Search as SearchIcon,
  Transform as ConvertIcon
} from '@mui/icons-material';
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { useSpreadsheetSelection } from '@/tabs/spreadsheet/managers/useSpreadsheetSelection';
import { sidebarStyles } from '@/tabs/spreadsheet/components/sidebar/utils/sidebarStyles';
import SidebarCard from '@/tabs/spreadsheet/components/sidebar/SidebarCard';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';
import { spreadsheetEventBus } from '@/tabs/spreadsheet/managers/SpreadsheetEventBus';
import { useUnitData } from '@/tabs/spreadsheet/components/sidebar/logic/useUnitData';
import { useSpreadsheetUnitConversion } from '@/tabs/spreadsheet/components/sidebar/logic/useSpreadsheetUnitConversion';

interface UnitConversionSidebarProps {
  open: boolean;
  onClose: () => void;
  spreadsheetRef?: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  // Lifted state for persistence
  category: string;
  setCategory: (category: string) => void;
  fromUnit: string;
  setFromUnit: (unit: string) => void;
  toUnit: string;
  setToUnit: (unit: string) => void;
  value: string;
  setValue: (value: string) => void;
}

type FocusedInputType = 'value' | 'outputTarget' | null;



const UnitConversionSidebar = React.memo<UnitConversionSidebarProps>(({
  open,
  onClose,
  spreadsheetRef,
  onSelectionChange,
  category,
  setCategory,
  fromUnit,
  setFromUnit,
  toUnit,
  setToUnit,
  value,
  setValue
}) => {
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [activeUnitInput, setActiveUnitInput] = useState<'from' | 'to' | null>(null);
  const [lastFocusedUnitInput, setLastFocusedUnitInput] = useState<'from' | 'to'>('from');
  const [outputTarget, setOutputTarget] = useState<string>('');

  // Use the extracted hooks
  const { categories, availableUnits, getFilteredUnits, loadCategories, loadUnits } = useUnitData();
  const {
    result,
    error,
    isConverting,
    compatibilityError,
    checkCompatibility,
    convert
  } = useSpreadsheetUnitConversion({
    spreadsheetRef,
    fromUnit,
    toUnit,
    value
  });

  // Use the spreadsheet selection hook
  const { focusedInput, handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange: onSelectionChange ?? (() => { }),
    updateField: (inputType, selection) => {
      switch (inputType) {
        case 'value':
          setValue(selection);
          break;
        case 'outputTarget':
          setOutputTarget(selection);
          break;
      }
    },
    sidebarDataAttribute: 'data-unit-converter-sidebar',
    handlerName: '__unitConverterSelectionHandler'
  });

  // Filtered units computation
  const filteredUnitsComputed = useMemo(() => {
    return getFilteredUnits(category, searchQuery);
  }, [getFilteredUnits, category, searchQuery]);

  // Subscribe to spreadsheet selection events via event bus
  useEffect(() => {
    if (!open) { return; }

    const unsubscribe = spreadsheetEventBus.on('selection-change', (cellRef) => {
      // Call the window handler that the hook is listening to
      const handler = window.__unitConverterSelectionHandler;
      if (handler) {
        handler(cellRef);
      }
    });

    return unsubscribe;
  }, [open]);

  // Load categories on mount
  useEffect(() => {
    if (open) {
      void loadCategories();
    }
  }, [open, loadCategories]);

  // Auto-select "All" category when categories are loaded
  useEffect(() => {
    if (open && categories.length > 0 && (!category || category === '')) {
      setCategory('All');
    }
  }, [open, categories, category, setCategory]);

  // Load units when category changes
  useEffect(() => {
    if (category) {
      void loadUnits();
    }
  }, [category, loadUnits]);

  // Check unit compatibility when both units are selected
  useEffect(() => {
    if (fromUnit && toUnit && fromUnit !== toUnit) {
      void checkCompatibility();
    }
  }, [fromUnit, toUnit, checkCompatibility]);

  // Handle convert button click
  const handleConvert = useCallback(async () => {
    await convert(outputTarget);
  }, [convert, outputTarget]);

  // Category icons mapping (blue theme)
  const categoryIcons: Record<string, string> = {
    'All': '🌍',
    'length': '📏',
    'mass': '⚖️',
    'time': '⏱️',
    'temperature': '🌡️',
    'energy': '⚡',
    'power': '💡',
    'pressure': '🔽',
    'velocity': '🚀',
    'force': '💪',
    'frequency': '〰️',
    'current': '⚡',
    'other': '📦'
  };

  // Special symbols for unit entry
  const specialSymbols = [
    { symbol: 'μ', label: 'micro (μ)' },
    { symbol: '°', label: 'degree (°)' },
    { symbol: '²', label: 'squared (²)' },
    { symbol: '³', label: 'cubed (³)' },
    { symbol: '⁴', label: 'power 4 (⁴)' },
    { symbol: '⁻¹', label: 'inverse (⁻¹)' },
    { symbol: '⁻²', label: 'inverse squared (⁻²)' },
    { symbol: 'Ω', label: 'ohm (Ω)' },
  ];



  if (!open) { return null; }

  return (
    <Box
      data-unit-converter-sidebar
      sx={{ ...sidebarStyles.container, px: 1, pt: 2 }}
    >
      {/* Header */}
      <Box sx={sidebarStyles.header}>
        <Typography sx={sidebarStyles.text.header}>
          Unit Converter
        </Typography>
        <IconButton
          onClick={onClose}
          size="small"
          sx={sidebarStyles.iconButton}
        >
          <CloseIcon />
        </IconButton>
      </Box>

      {/* Main Content */}
      <Box sx={sidebarStyles.contentWrapper}>


        {/* Conversion Panel */}
        <SidebarCard title="Unit Conversion" sx={{ mx: 0.5 }}>
          {/* From Unit */}
          <Box sx={{ mb: 1 }}>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 12, fontWeight: 600, mb: 0.5, display: 'block' }}>
              FROM
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <TextField
                size="small"
                value={fromUnit}
                onChange={(e) => setFromUnit(e.target.value)}
                onFocus={() => {
                  setActiveUnitInput('from');
                  setLastFocusedUnitInput('from');
                }}
                onBlur={() => setActiveUnitInput(null)}
                placeholder="e.g., m, kg, s"
                sx={{
                  flex: 1,
                  '& .MuiOutlinedInput-root': {
                    bgcolor: activeUnitInput === 'from' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '6px',
                    '& fieldset': { borderColor: activeUnitInput === 'from' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                    '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                    '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                    '& input': { color: 'white', fontFamily: 'monospace', fontSize: 12 }
                  }
                }}
              />
            </Box>

            {/* Special Symbols for From Unit */}
            <Box sx={{ mt: 0.5, display: 'flex', flexWrap: 'wrap', gap: 0.3 }}>
              {specialSymbols.map(({ symbol }) => (
                <Chip
                  key={`from-${symbol}`}
                  label={symbol}
                  size="small"
                  onClick={() => setFromUnit(fromUnit + symbol)}
                  sx={{
                    fontSize: 14,
                    height: 26,
                    bgcolor: 'rgba(33, 150, 243, 0.05)',
                    border: '1px solid rgba(33, 150, 243, 0.2)',
                    color: anafisColors.spreadsheet,
                    '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.15)' }
                  }}
                />
              ))}
            </Box>
          </Box>

          {/* Value Input */}
          <Box sx={{ mb: 1 }}>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 12, fontWeight: 600, mb: 0.5, display: 'block' }}>
              VALUE
              {focusedInput === 'value' && ' ← select on spreadsheet'}
            </Typography>
            <TextField
              fullWidth
              size="small"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              onFocus={() => handleInputFocus('value')}
              onBlur={handleInputBlur}
              placeholder="e.g., 5.2 or A1 or A1:A10"
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: focusedInput === 'value' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: focusedInput === 'value' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontFamily: 'monospace', fontSize: 12 }
                }
              }}
            />
          </Box>

          {/* Swap Button */}
          <Box sx={{ display: 'flex', justifyContent: 'center', mb: 1 }}>
            <IconButton
              onClick={() => {
                const temp = fromUnit;
                setFromUnit(toUnit);
                setToUnit(temp);
              }}
              size="small"
              sx={{
                color: anafisColors.spreadsheet,
                '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.1)' }
              }}
            >
              <SwapIcon />
            </IconButton>
          </Box>

          {/* To Unit */}
          <Box sx={{ mb: 1 }}>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 12, fontWeight: 600, mb: 0.5, display: 'block' }}>
              TO
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <TextField
                size="small"
                value={toUnit}
                onChange={(e) => setToUnit(e.target.value)}
                onFocus={() => {
                  setActiveUnitInput('to');
                  setLastFocusedUnitInput('to');
                }}
                onBlur={() => setActiveUnitInput(null)}
                placeholder="e.g., ft, lb, min"
                sx={{
                  flex: 1,
                  '& .MuiOutlinedInput-root': {
                    bgcolor: activeUnitInput === 'to' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '6px',
                    '& fieldset': { borderColor: activeUnitInput === 'to' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                    '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                    '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                    '& input': { color: 'white', fontFamily: 'monospace', fontSize: 12 }
                  }
                }}
              />
            </Box>

            {/* Special Symbols for To Unit */}
            <Box sx={{ mt: 0.5, display: 'flex', flexWrap: 'wrap', gap: 0.3 }}>
              {specialSymbols.map(({ symbol }) => (
                <Chip
                  key={`to-${symbol}`}
                  label={symbol}
                  size="small"
                  onClick={() => setToUnit(toUnit + symbol)}
                  sx={{
                    fontSize: 14,
                    height: 26,
                    bgcolor: 'rgba(33, 150, 243, 0.05)',
                    border: '1px solid rgba(33, 150, 243, 0.2)',
                    color: anafisColors.spreadsheet,
                    '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.15)' }
                  }}
                />
              ))}
            </Box>
          </Box>

          {/* To Field */}
          <Box sx={{ mt: 1 }}>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 12, fontWeight: 600, mb: 0.5, display: 'block' }}>
              TO (optional - leave empty to show result below)
              {focusedInput === 'outputTarget' && ' ← select on spreadsheet'}
            </Typography>
            <TextField
              fullWidth
              size="small"
              value={outputTarget}
              onChange={(e) => setOutputTarget(e.target.value)}
              onFocus={() => handleInputFocus('outputTarget')}
              onBlur={handleInputBlur}
              placeholder="e.g., B1 or B1:B10 (optional)"
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: focusedInput === 'outputTarget' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: focusedInput === 'outputTarget' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontFamily: 'monospace', fontSize: 12 }
                }
              }}
            />
          </Box>

          {/* Convert Button */}
          <Button
            fullWidth
            variant="contained"
            startIcon={isConverting ? <CircularProgress size={16} /> : <ConvertIcon />}
            onClick={() => void handleConvert()}
            disabled={isConverting || !!compatibilityError}
            sx={{
              ...sidebarStyles.button.primary,
              mt: 2,
              fontSize: 14,
              py: 1.5
            }}
          >
            {isConverting ? 'Converting...' : 'Convert'}
          </Button>

          {/* Error Display */}
          {error && (
            <Alert severity="error" sx={{ mt: 2, py: 0.5, fontSize: 12 }}>
              {error}
            </Alert>
          )}

          {/* Result Display */}
          {result && (
            <Box sx={{ mt: 2, p: 1.5, bgcolor: 'rgba(33, 150, 243, 0.1)', borderRadius: 1, border: '1px solid rgba(33, 150, 243, 0.2)' }}>
              <Typography sx={{ ...sidebarStyles.text.body, fontFamily: 'monospace', fontSize: '0.9rem', color: anafisColors.spreadsheet }}>
                {result}
              </Typography>
            </Box>
          )}

          {/* Compatibility Error Display */}
          {compatibilityError && (
            <Alert severity="error" sx={{ mt: 1, py: 0.5, fontSize: 12 }}>
              {compatibilityError}
            </Alert>
          )}
        </SidebarCard>

        {/* Category Selection, Search, and Units List */}
        <Box>
          <SidebarCard variant="compact" title="Category" sx={{ mx: 0.5 }}>
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
              {categories.map((cat) => (
                <Chip
                  key={cat}
                  label={`${categoryIcons[cat] ?? '📦'}`}
                  size="small"
                  onClick={() => setCategory(cat)}
                  sx={{
                    fontSize: 16,
                    height: 28,
                    bgcolor: category === cat ? 'rgba(33, 150, 243, 0.2)' : 'rgba(33, 150, 243, 0.05)',
                    border: '1px solid',
                    borderColor: category === cat ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)',
                    color: anafisColors.spreadsheet,
                    '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.15)' }
                  }}
                />
              ))}
            </Box>
          </SidebarCard>

          <SidebarCard variant="compact" title="Search Units" sx={{ mx: 0.5, flex: 1, minHeight: 450 }}>
            <TextField
              fullWidth
              size="small"
              placeholder="Search units..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              InputProps={{
                startAdornment: <SearchIcon sx={{ mr: 1, color: 'text.secondary' }} />,
              }}
              sx={{
                mb: 1,
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontSize: 12 }
                }
              }}
            />

            <List dense sx={{ maxHeight: 450, overflow: 'auto' }}>
              {filteredUnitsComputed.slice(0, 50).map((symbol) => {
                const unitInfo = availableUnits[symbol];
                if (!unitInfo) { return null; }

                return (
                  <ListItemButton
                    key={symbol}
                    onClick={() => {
                      // Use activeUnitInput if available, otherwise use lastFocusedUnitInput
                      const targetInput = activeUnitInput ?? lastFocusedUnitInput;

                      if (targetInput === 'from') {
                        setFromUnit(fromUnit + symbol);
                      } else {
                        setToUnit(toUnit + symbol);
                      }
                    }}
                    sx={{
                      py: 0.3,
                      px: 0.8,
                      borderRadius: 1,
                      mb: 0.3,
                      bgcolor: (fromUnit === symbol || toUnit === symbol) ? 'rgba(33, 150, 243, 0.1)' : 'transparent',
                      '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.15)' }
                    }}
                  >
                    <ListItemText
                      primary={
                        <Typography sx={{ fontSize: 12, fontWeight: 600, color: anafisColors.spreadsheet }}>
                          {symbol}
                        </Typography>
                      }
                      secondary={
                        <Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.7)' }}>
                          {unitInfo.name}
                        </Typography>
                      }
                    />
                  </ListItemButton>
                );
              })}
            </List>
          </SidebarCard>
        </Box>
      </Box>
    </Box>
  );
});

UnitConversionSidebar.displayName = 'UnitConversionSidebar';

export default UnitConversionSidebar;