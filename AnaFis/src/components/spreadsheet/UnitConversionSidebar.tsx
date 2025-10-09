import React, { useState, useEffect } from 'react';
import {
  Paper,
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
  Radio,
  RadioGroup,
  FormControlLabel,
  FormControl,
  FormLabel,
  CircularProgress
} from '@mui/material';
import {
  Close as CloseIcon,
  SwapHoriz as SwapIcon,
  Search as SearchIcon,
  Transform as ConvertIcon
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { UniverSpreadsheetRef } from './UniverSpreadsheet';
import { useSpreadsheetSelection } from '../../hooks/useSpreadsheetSelection';

interface UnitConversionSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef?: React.RefObject<UniverSpreadsheetRef | null>;
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

interface ConversionResult {
  value: number;
  formatted_result: string;
  conversion_factor: number;
}

interface UnitInfo {
  symbol: string;
  name: string;
  category: string;
  description: string;
}

type OutputMode = 'cell' | 'range' | 'inPlace';

type FocusedInputType = 'value' | 'outputTarget' | null;

const UnitConversionSidebar: React.FC<UnitConversionSidebarProps> = ({ 
  open, 
  onClose,
  univerRef,
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
  const [categories, setCategories] = useState<string[]>([]);
  const [availableUnits, setAvailableUnits] = useState<Record<string, UnitInfo>>({});
  const [filteredUnits, setFilteredUnits] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [activeUnitInput, setActiveUnitInput] = useState<'from' | 'to' | null>(null);
  
  const [outputMode, setOutputMode] = useState<OutputMode>('cell');
  const [outputTarget, setOutputTarget] = useState<string>('');
  const [result, setResult] = useState<string>('');
  
  // Use the spreadsheet selection hook
  const { focusedInput, handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange,
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
  const [error, setError] = useState<string>('');
  const [isConverting, setIsConverting] = useState<boolean>(false);
  const [compatibilityError, setCompatibilityError] = useState<string>('');

  // Category icons mapping (blue theme)
  const categoryIcons: Record<string, string> = {
    'length': '📏',
    'mass': '⚖️',
    'time': '⏱️',
    'temperature': '🌡️',
    'energy': '⚡',
    'power': '💡',
    'pressure': '🔽',
    'velocity': '',
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

  // Load categories on mount
  useEffect(() => {
    if (open) {
      loadCategories();
    }
  }, [open]);

  // Load units when category changes
  useEffect(() => {
    if (category) {
      loadUnits();
    }
  }, [category]);

  // Filter units based on search
  useEffect(() => {
    const unitSymbols = Object.keys(availableUnits);
    if (searchQuery.trim() === '') {
      // Filter by category
      const filtered = unitSymbols.filter(symbol => {
        const unitInfo = availableUnits[symbol];
        return unitInfo.category.toLowerCase() === category.toLowerCase();
      });
      setFilteredUnits(filtered);
    } else {
      const query = searchQuery.toLowerCase();
      const filtered = unitSymbols.filter(symbol => {
        const unitInfo = availableUnits[symbol];
        return unitInfo.category.toLowerCase() === category.toLowerCase() &&
               (symbol.toLowerCase().includes(query) || 
                unitInfo.name.toLowerCase().includes(query));
      });
      setFilteredUnits(filtered);
    }
  }, [searchQuery, availableUnits, category]);

  const loadCategories = async () => {
    try {
      const cats: string[] = await invoke('get_supported_categories');
      setCategories(cats);
      if (cats.length > 0 && !category) {
        setCategory(cats[0]);
      }
    } catch (err) {
      setError('Failed to load categories');
      console.error(err);
    }
  };

  const loadUnits = async () => {
    try {
      const units: Record<string, UnitInfo> = await invoke('get_available_units');
      setAvailableUnits(units);
    } catch (err) {
      setError('Failed to load units');
      console.error(err);
    }
  };

  const handleUnitClick = (unit: string) => {
    if (activeUnitInput === 'from') {
      setFromUnit(unit);
    } else if (activeUnitInput === 'to') {
      setToUnit(unit);
    }
  };

  const insertSymbol = (symbol: string, inputType: 'from' | 'to') => {
    if (inputType === 'from') {
      setFromUnit(fromUnit + symbol);
    } else {
      setToUnit(toUnit + symbol);
    }
  };

  const handleSwapUnits = () => {
    const temp = fromUnit;
    setFromUnit(toUnit);
    setToUnit(temp);
  };

  // Check unit compatibility when units change
  useEffect(() => {
    const checkCompatibility = async () => {
      if (!fromUnit || !toUnit) {
        setCompatibilityError('');
        return;
      }

      try {
        const isCompatible: boolean = await invoke('check_unit_compatibility', {
          fromUnit,
          toUnit
        });

        if (!isCompatible) {
                    setCompatibilityError(`Incompatible dimensions: ${fromUnit} and ${toUnit} cannot be converted`);
        } else {
          setCompatibilityError('');
        }
      } catch (err) {
        console.error('Failed to check compatibility:', err);
      }
    };

    checkCompatibility();
  }, [fromUnit, toUnit]);

  const handleConvert = async () => {
    setError('');
    setResult('');

    if (!fromUnit || !toUnit) {
      setError('Please enter both units');
      return;
    }

    setIsConverting(true);
    try {
      // If value is empty, default to 1 (show conversion rate)
      const valueToConvert = value.trim() || '1';
      
      // Check if value is a number, cell reference, or range
      const isRange = valueToConvert.includes(':');
      const isCellRef = /^[A-Z]+\d+$/.test(valueToConvert.trim());

      if (isRange) {
        // Handle range conversion
        await handleRangeConversion(valueToConvert);
      } else if (isCellRef) {
        // Handle single cell conversion
        await handleCellConversion(valueToConvert);
      } else {
        // Handle direct number conversion
        const numValue = parseFloat(valueToConvert);
        if (isNaN(numValue)) {
          setError('Invalid number format');
          return;
        }

        const conversionResult: ConversionResult = await invoke('convert_value', {
          request: {
            value: numValue,
            from_unit: fromUnit,
            to_unit: toUnit
          }
        });

        // Show if this was the conversion rate (when value was empty)
        const isConversionRate = value.trim() === '';
        const prefix = isConversionRate ? 'Conversion rate: 1 ' + fromUnit + ' = ' : '';
        setResult(`${prefix}${conversionResult.value.toFixed(6)} ${toUnit}`);
        
        // If output mode is cell or range, write to spreadsheet (only if not showing conversion rate)
        if (!isConversionRate && outputMode === 'cell' && outputTarget && univerRef?.current) {
          univerRef.current.updateCell(outputTarget, { v: conversionResult.value });
        }
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setIsConverting(false);
    }
  };

  const handleCellConversion = async (cellRef: string) => {
    if (!univerRef?.current) {
      setError('Spreadsheet not initialized');
      return;
    }

    try {
      // Read value from cell
      const cellValue = univerRef.current.getCellValue(cellRef);
      if (cellValue === null || cellValue === undefined) {
        setError(`Cell ${cellRef} is empty`);
        return;
      }

      const numValue = typeof cellValue === 'number' ? cellValue : parseFloat(String(cellValue));
      if (isNaN(numValue)) {
        setError(`Cell ${cellRef} does not contain a valid number`);
        return;
      }

      // Convert the value
      const conversionResult: ConversionResult = await invoke('convert_value', {
        request: {
          value: numValue,
          from_unit: fromUnit,
          to_unit: toUnit
        }
      });

      setResult(`${cellRef}: ${numValue} ${fromUnit} = ${conversionResult.value.toFixed(6)} ${toUnit}`);

      // Handle output based on mode
      if (outputMode === 'inPlace') {
        // Replace the original cell value
        univerRef.current.updateCell(cellRef, { v: conversionResult.value });
      } else if (outputMode === 'cell' && outputTarget) {
        // Write to output cell
        univerRef.current.updateCell(outputTarget, { v: conversionResult.value });
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleRangeConversion = async (rangeRef: string) => {
    if (!univerRef?.current) {
      setError('Spreadsheet not initialized');
      return;
    }

    try {
      // Parse range (e.g., "A1:A10")
      const match = rangeRef.match(/^([A-Z]+)(\d+):([A-Z]+)(\d+)$/);
      if (!match) {
        setError('Invalid range format. Use format like A1:A10');
        return;
      }

      const startCol = match[1];
      const startRow = parseInt(match[2]);
      const endCol = match[3];
      const endRow = parseInt(match[4]);

      if (startCol !== endCol) {
        setError('Range must be in a single column');
        return;
      }

      // Determine output range
      let outputCol = startCol;
      let outputStartRow = startRow;
      
      if (outputMode === 'cell' || outputMode === 'range') {
        if (!outputTarget) {
          setError('Please specify output location');
          return;
        }
        
        const outputMatch = outputTarget.match(/^([A-Z]+)(\d+)/);
        if (!outputMatch) {
          setError('Invalid output format');
          return;
        }
        outputCol = outputMatch[1];
        outputStartRow = parseInt(outputMatch[2]);
      }

      // Convert all values in range
      let convertedCount = 0;
      const results: string[] = [];

      for (let row = startRow; row <= endRow; row++) {
        const cellRef = `${startCol}${row}`;
        const cellValue = univerRef.current.getCellValue(cellRef);
        
        if (cellValue === null || cellValue === undefined) {
          continue; // Skip empty cells
        }

        const numValue = typeof cellValue === 'number' ? cellValue : parseFloat(String(cellValue));
        if (isNaN(numValue)) {
          continue; // Skip non-numeric cells
        }

        // Convert the value
        const conversionResult: ConversionResult = await invoke('convert_value', {
          request: {
            value: numValue,
            from_unit: fromUnit,
            to_unit: toUnit
          }
        });

        const outputRow = outputStartRow + (row - startRow);
        const outputCellRef = `${outputCol}${outputRow}`;

        // Write to output location
        if (outputMode === 'inPlace') {
          univerRef.current.updateCell(cellRef, { v: conversionResult.value });
        } else {
          univerRef.current.updateCell(outputCellRef, { v: conversionResult.value });
        }

        convertedCount++;
        if (results.length < 3) {
          results.push(`${cellRef}: ${numValue} → ${conversionResult.value.toFixed(4)}`);
        }
      }

      if (convertedCount === 0) {
        setError('No valid numeric values found in range');
      } else {
        const preview = results.join('\n');
        const more = convertedCount > 3 ? `\n... and ${convertedCount - 3} more` : '';
        setResult(`Converted ${convertedCount} values:\n${preview}${more}`);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  if (!open) return null;

  return (
    <Paper data-unit-converter-sidebar elevation={3} sx={{ 
      width: 420, 
      height: '100%', 
      display: 'flex', 
      flexDirection: 'column', 
      bgcolor: 'rgba(10, 25, 45, 0.98)', 
      border: '1px solid rgba(33, 150, 243, 0.2)', 
      borderLeft: '2px solid rgba(33, 150, 243, 0.5)',
      borderRadius: 0,
      overflow: 'hidden'
    }}>
      {/* Header */}
      <Box sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'space-between', 
        p: 2, 
        bgcolor: 'rgba(33, 150, 243, 0.08)', 
        borderBottom: '1px solid rgba(33, 150, 243, 0.2)' 
      }}>
        <Typography variant="h6" sx={{ fontWeight: 600, color: '#2196f3' }}>
          Unit Converter
        </Typography>
        <IconButton 
          onClick={onClose} 
          size="small" 
          sx={{ 
            color: 'rgba(255, 255, 255, 0.7)', 
            borderRadius: '6px', 
            '&:hover': { 
              bgcolor: 'rgba(33, 150, 243, 0.2)', 
              color: 'rgba(255, 255, 255, 0.9)' 
            } 
          }}
        >
          <CloseIcon />
        </IconButton>
      </Box>

      {/* Tip */}
      <Box sx={{ 
        p: 1.5, 
        bgcolor: 'rgba(33, 150, 243, 0.08)', 
        borderBottom: '1px solid rgba(33, 150, 243, 0.15)' 
      }}>
        <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.8)', fontSize: 11, lineHeight: 1.4 }}>
          💡 Browse units, enter manually with symbols, convert numbers or cells
        </Typography>
      </Box>

      {/* Main Content */}
      <Box sx={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Unit Browser Sidebar */}
        <Box sx={{ 
          width: 140, 
          borderRight: '1px solid rgba(33, 150, 243, 0.2)', 
          bgcolor: 'rgba(33, 150, 243, 0.03)', 
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden'
        }}>
          {/* Category Selection */}
          <Box sx={{ p: 1, borderBottom: '1px solid rgba(33, 150, 243, 0.2)' }}>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
              CATEGORY
            </Typography>
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
              {categories.map((cat) => (
                <Chip
                  key={cat}
                  label={`${categoryIcons[cat] || '📦'}`}
                  size="small"
                  onClick={() => setCategory(cat)}
                  sx={{
                    fontSize: 16,
                    height: 28,
                    bgcolor: category === cat ? 'rgba(33, 150, 243, 0.2)' : 'rgba(33, 150, 243, 0.05)',
                    border: '1px solid',
                    borderColor: category === cat ? '#2196f3' : 'rgba(33, 150, 243, 0.2)',
                    color: '#2196f3',
                    '&:hover': {
                      bgcolor: 'rgba(33, 150, 243, 0.15)',
                      borderColor: '#2196f3'
                    }
                  }}
                />
              ))}
            </Box>
            <Typography variant="caption" sx={{ color: '#2196f3', fontSize: 9, mt: 0.5, display: 'block', textAlign: 'center', textTransform: 'capitalize' }}>
              {category}
            </Typography>
          </Box>

          {/* Unit Search */}
          <Box sx={{ p: 1, borderBottom: '1px solid rgba(33, 150, 243, 0.2)' }}>
            <TextField
              fullWidth
              size="small"
              placeholder="Search units..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              slotProps={{
                input: {
                  startAdornment: <SearchIcon sx={{ fontSize: 16, color: 'rgba(255, 255, 255, 0.5)', mr: 0.5 }} />,
                  style: { fontSize: 11 }
                }
              }}
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: '#2196f3' },
                  '& input': { color: 'white', padding: '6px 8px' }
                }
              }}
            />
          </Box>

          {/* Unit List - Browse Only */}
          <Box sx={{ flex: 1, overflow: 'auto' }}>
            <List dense sx={{ px: 0.5, py: 0.5 }}>
              {filteredUnits.map((symbol) => {
                const unitInfo = availableUnits[symbol];
                return (
                  <ListItemButton
                    key={symbol}
                    onClick={() => handleUnitClick(symbol)}
                    disabled={!activeUnitInput}
                    sx={{
                      px: 1,
                      py: 0.5,
                      mb: 0.5,
                      borderRadius: '6px',
                      border: '1px solid rgba(33, 150, 243, 0.2)',
                      bgcolor: 'rgba(33, 150, 243, 0.05)',
                      '&:hover': {
                        bgcolor: 'rgba(33, 150, 243, 0.15)',
                        borderColor: '#2196f3',
                        transform: 'translateY(-1px)',
                      },
                      '&.Mui-disabled': {
                        opacity: 0.5
                      }
                    }}
                  >
                    <ListItemText 
                      primary={
                        <Box>
                          <Typography sx={{ fontSize: 11, fontFamily: 'monospace', color: 'rgba(255, 255, 255, 0.9)', fontWeight: 600 }}>
                            {symbol}
                          </Typography>
                          <Typography sx={{ fontSize: 9, color: 'rgba(255, 255, 255, 0.6)' }}>
                            {unitInfo?.name}
                          </Typography>
                        </Box>
                      }
                    />
                  </ListItemButton>
                );
              })}
            </List>
          </Box>
        </Box>

        {/* Conversion Panel */}
        <Box sx={{ flex: 1, p: 1.5, overflow: 'auto', display: 'flex', flexDirection: 'column', gap: 1.5 }}>
          {/* From Unit */}
          <Box>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 0.5 }}>
              <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600 }}>
                FROM UNIT {activeUnitInput === 'from' && '← click unit'}
              </Typography>
            </Box>
            <TextField
              fullWidth
              size="small"
              value={fromUnit}
              onChange={(e) => setFromUnit(e.target.value)}
              onFocus={() => setActiveUnitInput('from')}
              placeholder="Enter unit (e.g., m, kg, °C)"
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: activeUnitInput === 'from' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: activeUnitInput === 'from' ? '#2196f3' : 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: '#2196f3' },
                  '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                }
              }}
            />
            {/* Symbol Buttons for From Unit */}
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
              {specialSymbols.map((sym) => (
                <Button
                  key={sym.symbol}
                  size="small"
                  onClick={() => insertSymbol(sym.symbol, 'from')}
                  sx={{
                    minWidth: 32,
                    height: 24,
                    fontSize: 12,
                    padding: '2px 6px',
                    bgcolor: 'rgba(33, 150, 243, 0.1)',
                    color: '#2196f3',
                    border: '1px solid rgba(33, 150, 243, 0.3)',
                    '&:hover': {
                      bgcolor: 'rgba(33, 150, 243, 0.2)',
                      borderColor: '#2196f3'
                    }
                  }}
                  title={sym.label}
                >
                  {sym.symbol}
                </Button>
              ))}
            </Box>
          </Box>

          {/* Swap Button */}
          <Box sx={{ display: 'flex', justifyContent: 'center' }}>
            <IconButton
              onClick={handleSwapUnits}
              sx={{
                bgcolor: 'rgba(33, 150, 243, 0.1)',
                border: '1px solid rgba(33, 150, 243, 0.3)',
                '&:hover': {
                  bgcolor: 'rgba(33, 150, 243, 0.2)',
                  borderColor: '#2196f3',
                  transform: 'rotate(180deg)',
                  transition: 'transform 0.3s'
                }
              }}
            >
              <SwapIcon sx={{ color: '#2196f3' }} />
            </IconButton>
          </Box>

          {/* To Unit */}
          <Box>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 0.5 }}>
              <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600 }}>
                TO UNIT {activeUnitInput === 'to' && '← click unit'}
              </Typography>
            </Box>
            <TextField
              fullWidth
              size="small"
              value={toUnit}
              onChange={(e) => setToUnit(e.target.value)}
              onFocus={() => setActiveUnitInput('to')}
              placeholder="Enter unit (e.g., km, g, °F)"
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: activeUnitInput === 'to' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: activeUnitInput === 'to' ? '#2196f3' : 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: '#2196f3' },
                  '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                }
              }}
            />
            {/* Symbol Buttons for To Unit */}
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
              {specialSymbols.map((sym) => (
                <Button
                  key={sym.symbol}
                  size="small"
                  onClick={() => insertSymbol(sym.symbol, 'to')}
                  sx={{
                    minWidth: 32,
                    height: 24,
                    fontSize: 12,
                    padding: '2px 6px',
                    bgcolor: 'rgba(33, 150, 243, 0.1)',
                    color: '#2196f3',
                    border: '1px solid rgba(33, 150, 243, 0.3)',
                    '&:hover': {
                      bgcolor: 'rgba(33, 150, 243, 0.2)',
                      borderColor: '#2196f3'
                    }
                  }}
                  title={sym.label}
                >
                  {sym.symbol}
                </Button>
              ))}
            </Box>
          </Box>

          <Box sx={{ height: '1px', bgcolor: 'rgba(33, 150, 243, 0.2)', my: 0.5 }} />

          {/* Value Input */}
          <Box>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
              VALUE (number, cell, or range) {focusedInput === 'value' && '← select on spreadsheet'}
            </Typography>
            <TextField
              fullWidth
              size="small"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              onFocus={() => handleInputFocus('value')}
              onBlur={handleInputBlur}
              placeholder="1 or A1 or A1:A10"
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: focusedInput === 'value' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: focusedInput === 'value' ? '#2196f3' : 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: '#2196f3' },
                  '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                }
              }}
            />
          </Box>

          {/* Output Mode */}
          <Box>
            <FormControl component="fieldset" size="small">
              <FormLabel component="legend" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5 }}>
                OUTPUT
              </FormLabel>
              <RadioGroup
                value={outputMode}
                onChange={(e) => setOutputMode(e.target.value as OutputMode)}
                sx={{ gap: 0.5 }}
              >
                <FormControlLabel
                  value="cell"
                  control={<Radio size="small" sx={{ py: 0.5, color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: '#2196f3' } }} />}
                  label={<Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.9)' }}>To specific cell</Typography>}
                  sx={{ height: 28 }}
                />
                <FormControlLabel
                  value="range"
                  control={<Radio size="small" sx={{ py: 0.5, color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: '#2196f3' } }} />}
                  label={<Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.9)' }}>To range</Typography>}
                  sx={{ height: 28 }}
                />
                <FormControlLabel
                  value="inPlace"
                  control={<Radio size="small" sx={{ py: 0.5, color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: '#2196f3' } }} />}
                  label={<Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.9)' }}>Replace in-place</Typography>}
                  sx={{ height: 28 }}
                />
              </RadioGroup>
            </FormControl>
            
            {(outputMode === 'cell' || outputMode === 'range') && (
              <Box>
                <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
                  {focusedInput === 'outputTarget' && '← select on spreadsheet'}
                </Typography>
                <TextField
                  fullWidth
                  size="small"
                  value={outputTarget}
                  onChange={(e) => setOutputTarget(e.target.value)}
                  onFocus={() => handleInputFocus('outputTarget')}
                  onBlur={handleInputBlur}
                  placeholder={outputMode === 'cell' ? 'e.g., B1' : 'e.g., B1:B10'}
                  sx={{
                    mt: 0.5,
                    '& .MuiOutlinedInput-root': {
                      bgcolor: focusedInput === 'outputTarget' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                      borderRadius: '6px',
                      '& fieldset': { borderColor: focusedInput === 'outputTarget' ? '#2196f3' : 'rgba(33, 150, 243, 0.2)' },
                      '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                      '&.Mui-focused fieldset': { borderColor: '#2196f3' },
                      '& input': { color: 'white', fontFamily: 'monospace', fontSize: 12 }
                    }
                  }}
                />
              </Box>
            )}
          </Box>

          {/* Convert Button */}
          <Button
            fullWidth
            variant="contained"
            startIcon={isConverting ? <CircularProgress size={16} /> : <ConvertIcon />}
            onClick={handleConvert}
            disabled={isConverting || !!compatibilityError}
            sx={{
              bgcolor: '#2196f3',
              fontWeight: 600,
              fontSize: 12,
              py: 1,
              outline: 'none',
              '&:hover': { bgcolor: '#2196f3' },
              '&:disabled': { bgcolor: '#424242' },
              '&:focus': { bgcolor: '#2196f3', outline: 'none' },
              '&:focus-visible': { bgcolor: '#2196f3', outline: 'none', boxShadow: 'none' },
              '&:active': { bgcolor: '#2196f3' }
            }}
          >
            {isConverting ? 'Converting...' : 'Convert'}
          </Button>

          {/* Compatibility Error Display */}
          {compatibilityError && (
            <Alert severity="error" onClose={() => setCompatibilityError('')} sx={{ py: 0.5, fontSize: 11 }}>
              {compatibilityError}
            </Alert>
          )}

          {/* Result Display */}
          {result && (
            <Paper sx={{
              p: 1.5,
              bgcolor: 'rgba(33, 150, 243, 0.08)',
              border: '1px solid rgba(33, 150, 243, 0.3)',
              borderRadius: '6px'
            }}>
              <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, display: 'block', mb: 0.5 }}>
                RESULT
              </Typography>
              <Typography sx={{ color: '#2196f3', fontFamily: 'monospace', fontSize: 14, fontWeight: 600 }}>
                {result}
              </Typography>
            </Paper>
          )}

          {/* Error Display */}
          {error && (
            <Alert severity="error" onClose={() => setError('')} sx={{ py: 0.5, fontSize: 11 }}>
              {error}
            </Alert>
          )}
        </Box>
      </Box>
    </Paper>
  );
};

export default UnitConversionSidebar;
