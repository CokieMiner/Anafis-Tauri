import { createRoot } from 'react-dom/client';
import { useEffect, useRef, useState } from 'react';
import { ThemeProvider } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
import Box from '@mui/material/Box';
import {
  Typography,
  IconButton,
  Paper,
  Button,
  Alert,
  TextField,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Radio,
  RadioGroup,
  FormControlLabel,
  Chip,
  Tooltip
} from '@mui/material';
import {
  Close,
  SwapHoriz,
  Science,
  Calculate
} from '@mui/icons-material';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { createAnafisTheme } from './themes';

// Create theme using shared configuration
const theme = createAnafisTheme();

// Backend response types
interface ConversionRequest {
  value: number;
  from_unit: string;
  to_unit: string;
}

interface ConversionResult {
  value: number;
  formatted_result: string;
  conversion_factor: number;
}

interface ConversionPreview {
  preview_text: string;
  conversion_factor: number;
  is_valid: boolean;
}

interface DimensionalAnalysisResult {
  unit_formula: string;
  dimensional_formula: string;
  si_factor: number;
  is_valid: boolean;
  error_message?: string;
}

interface RangeConversionRequest {
  range: string;
  from_unit: string;
  to_unit: string;
}

interface RangeConversionResult {
  range: string;
  converted_count: number;
  preview: string;
}

interface UnitInfo {
  symbol: string;
  name: string;
  category: string;
  description: string;
  icon?: string;
}

function UnitConversionWindow() {
  // Unit selection mode: standard (categories) or custom (expressions)
  const [unitMode, setUnitMode] = useState<'standard' | 'custom'>('standard');

  // Conversion target: single value or spreadsheet range
  const [conversionTarget, setConversionTarget] = useState<'value' | 'range'>('value');

  // Standard unit selection - Set better defaults
  const [selectedCategory, setSelectedCategory] = useState<string>('length');
  const [fromUnit, setFromUnit] = useState('m');
  const [toUnit, setToUnit] = useState('ft');

  // Custom unit expressions - Set helpful defaults
  const [customFromUnit, setCustomFromUnit] = useState('kg*m/s^2');
  const [customToUnit, setCustomToUnit] = useState('N');

  // Input values
  const [inputValue, setInputValue] = useState('100');
  const [spreadsheetRange, setSpreadsheetRange] = useState('A1:A10');

  // Results and status
  const [result, setResult] = useState<string>('');
  const [conversionPreview, setConversionPreview] = useState<string>('');
  const [unitCompatibility, setUnitCompatibility] = useState<string>('');
  const [validationError, setValidationError] = useState<string>('');
  const [isCalculating, setIsCalculating] = useState(false);

  // State for custom unit conversion
  const [availableUnits, setAvailableUnits] = useState<Record<string, UnitInfo>>({});
  const [loadingUnits, setLoadingUnits] = useState(true);
  // Category icons are provided by the frontend mapping (single source here)

  const windowRef = useRef<any>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const [contentHeight, setContentHeight] = useState<number>(450);

  // Load available units from backend on component mount
  useEffect(() => {
    const loadUnitsFromBackend = async () => {
      try {
        setLoadingUnits(true);
        const units: Record<string, UnitInfo> = await invoke('get_available_units');
        // No backend icon fetch: frontend keeps the canonical mapping
        setAvailableUnits(units);
        // Units loaded successfully

        // Set better defaults once units are loaded
        const unitArray = Object.values(units);
        const lengthUnits = unitArray.filter(u => u.category.toLowerCase() === 'length');
        if (lengthUnits.length > 0) {
          const meter = lengthUnits.find(u => u.symbol === 'm');
          const foot = lengthUnits.find(u => u.symbol === 'ft');
          if (meter && foot) {
            setFromUnit(meter.symbol);
            setToUnit(foot.symbol);
          } else {
            setFromUnit(lengthUnits[0].symbol);
            setToUnit(lengthUnits[1]?.symbol || lengthUnits[0].symbol);
          }
        }
      } catch (error) {
        // Failed to load units from backend
        // Fall back to frontend units if backend fails
      } finally {
        setLoadingUnits(false);
      }
    };

    loadUnitsFromBackend();
  }, []);

  // Auto-resize window based on content
  useEffect(() => {
    windowRef.current = getCurrentWindow();

    const measureAndResizeWindow = async () => {
      if (!contentRef.current || !windowRef.current) return;

      try {
        // Get the actual content height
        const contentElement = contentRef.current;
        const scrollHeight = contentElement.scrollHeight;

        // Calculate optimal height (content + title bar + padding + margins)
        const titleBarHeight = 32;
        const contentPadding = 16; // Further reduced from 24 to 16 (save 8px more)
        const extraMargin = 2; // Minimal buffer reduced from 5 to 2 (save 3px more)
        const minHeight = 350;
        const maxHeight = Math.min(800, window.screen.availHeight * 0.85); // 85% of screen height or 800px max

        const optimalHeight = Math.min(
          Math.max(scrollHeight + titleBarHeight + contentPadding + extraMargin, minHeight),
          maxHeight
        );

        // Keep width fixed - only resize height
        const fixedWidth = 600;

        // Only resize if there's a significant change (avoid flicker)
        if (Math.abs(optimalHeight - contentHeight) > 15) {
          setContentHeight(optimalHeight);

          // Use Tauri's built-in window API for resizing (height only)
          await windowRef.current.setSize(new LogicalSize(
            fixedWidth,
            Math.round(optimalHeight)
          ));

          // Center the window after resizing
          await windowRef.current.center();
        }
      } catch (error) {
        // Failed to auto-resize window
      }
    };

    // Initial resize with delay to ensure DOM is fully rendered
    const initialTimeout = setTimeout(measureAndResizeWindow, 200);

    // Set up ResizeObserver for content changes
    let resizeObserver: ResizeObserver | null = null;

    if (contentRef.current) {
      resizeObserver = new ResizeObserver(() => {
        // Debounce the resize to avoid excessive calls
        clearTimeout(initialTimeout);
        setTimeout(measureAndResizeWindow, 300);
      });

      resizeObserver.observe(contentRef.current);
    }

    return () => {
      clearTimeout(initialTimeout);
      if (resizeObserver) {
        resizeObserver.disconnect();
      }
    };
  }, [contentHeight]);

  // Auto-resize when content-affecting state changes
  useEffect(() => {
    const timer = setTimeout(() => {
      if (contentRef.current) {
        const event = new CustomEvent('contentChanged');
        contentRef.current.dispatchEvent(event);
      }
    }, 150);

    return () => clearTimeout(timer);
  }, [unitMode, conversionTarget, result, validationError, unitCompatibility, conversionPreview]);

  // Backend conversion functions
  const getConversionPreview = async (from: string, to: string): Promise<{ factor: number; preview: string; isValid: boolean }> => {
    try {
      if (from === to) {
        return { factor: 1, preview: `1 ${from} = 1 ${to}`, isValid: true };
      }

      const result: ConversionPreview = await invoke('get_conversion_preview', {
        from_unit: from,
        to_unit: to,
        // compatibility: also send camelCase keys
        fromUnit: from,
        toUnit: to
      });

      return {
        factor: result.conversion_factor,
        preview: result.preview_text,
        isValid: result.is_valid
      };
    } catch (error) {
      // Error getting conversion preview
      return { factor: 0, preview: `Error: ${from} ↔ ${to}`, isValid: false };
    }
  };

  const convertValue = async (value: number, from: string, to: string): Promise<ConversionResult | null> => {
    try {
      const request: ConversionRequest = {
        value,
        from_unit: from,
        to_unit: to
      };

      const result: ConversionResult = await invoke('convert_value', { request });
      return result;
    } catch (error) {
      // Error converting value
      return null;
    }
  };

  const parseUnitFormula = async (unitFormula: string): Promise<DimensionalAnalysisResult | null> => {
    try {
      return await invoke('parse_unit_formula', { unit_formula: unitFormula, unitFormula });
    } catch (error) {
      // Error parsing unit formula
      return null;
    }
  };

  // Validate units when they change
  useEffect(() => {
    const validateUnits = async () => {
      const from = unitMode === 'custom' ? customFromUnit : fromUnit;
      const to = unitMode === 'custom' ? customToUnit : toUnit;

      if (from && to) {
        try {
          const preview = await getConversionPreview(from, to);

          if (preview.isValid) {
            setUnitCompatibility('✅ Compatible');
            setConversionPreview(preview.preview);
            setValidationError('');
          } else {
            setUnitCompatibility('⚠️ Incompatible');
            setConversionPreview(preview.preview);
            setValidationError('Units are not dimensionally compatible');
          }
        } catch (error) {
          setUnitCompatibility('⚠️ Unknown');
          setValidationError(`Validation error: ${error}`);
          setConversionPreview('');
        }
      } else {
        setUnitCompatibility('');
        setConversionPreview('');
        setValidationError('');
      }
    };

    validateUnits();
  }, [fromUnit, toUnit, customFromUnit, customToUnit, unitMode]);

  const getAvailableUnits = (categoryOverride?: string): UnitInfo[] => {
    // Return units from backend, grouped by selected category if in standard mode
    const units = Object.values(availableUnits);
    const targetCategory = categoryOverride || selectedCategory;
    if (unitMode === 'standard' && targetCategory !== 'All') {
      return units.filter(unit => unit.category.toLowerCase() === targetCategory.toLowerCase());
    }
    return units;
  };

  // Function to get category icons
  const getCategoryIcon = (category: string): string => {
    const key = category.toLowerCase().trim();

    const icons: Record<string, string> = {
      'all': '🌐',
      'length': '📏',
      'distance': '📏',
      'mass': '⚖️',
      'time': '⏱️',
      'temperature': '🌡️',
      'current': '🔌',
      'electric current': '🔌',
      'amount': '🧪',
      'luminous_intensity': '💡',
      'luminous intensity': '💡',
      'angle': '📐',
      'area': '📐',
      'volume': '🧴',
      'velocity': '🚀',
      'speed': '🚀',
      'acceleration': '🌀',
      'force': '💪',
      'pressure': '🔧',
      'energy': '🔋',
      'power': '⚡',
      'frequency': '📶',
      'voltage': '🔌',
      'resistance': '🧲',
      'capacitance': '🧪',
      'inductance': '🔁',
      'conductance': '📈',
      'magnetic_flux_density': '🧲',
      'magnetic flux density': '🧲',
      'magnetic_flux': '🧲',
      'magnetic flux': '🧲',
      'electric_charge': '🔋',
      'electric charge': '�',
      'radiation_activity': '☢️',
      'radiation activity': '☢️',
      'radiation_dose': '☣️',
      'radiation dose': '☣️',
         'illuminance': '🔆',
      'data_storage': '💾',
      'data storage': '💾',
      'data': '💾',
      'computing': '🖥️',
      'textile': '🧵',
      'other': '📊',
      // synonyms
      'storage': '💾',
      'currency': '💰',
      'density': '🧱',
      'momentum': '💨',
      'flow rate': '🌊',
      'conductivity': '📈'
    };

    // Try exact key
    if (icons[key]) return icons[key];

    // Try replacing spaces <-> underscores
    const alt = key.replace(/\s+/g, '_');
    if (icons[alt]) return icons[alt];
    const alt2 = key.replace(/_/g, ' ');
    if (icons[alt2]) return icons[alt2];

    return '📊';
  };

  // Function to get unit icon based on category
  const getUnitIcon = (unit: UnitInfo): string => {
    if (unit.icon && unit.icon.length > 0) return unit.icon;
    // Defer to centralized category icon lookup (backend first, then minimal fallback)
    return getCategoryIcon(unit.category);
  };

  // Function to capitalize unit names properly
  const formatUnitName = (name: string): string => {
    return name.charAt(0).toUpperCase() + name.slice(1).toLowerCase();
  };

  // Function to convert snake_case to Title Case
  const formatCategoryName = (name: string): string => {
    if (name === 'All') return name;
    return name
      .split('_')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
      .join(' ');
  };

  const getAvailableCategories = (): string[] => {
    const units = Object.values(availableUnits);
    const categories = [...new Set(units.map(unit => unit.category))];
    return ['All', ...categories.sort()];
  };

  const handleCalculate = async () => {
    const from = unitMode === 'custom' ? customFromUnit : fromUnit;
    const to = unitMode === 'custom' ? customToUnit : toUnit;

    if (!from || !to) {
      setValidationError('Please select both source and target units');
      return;
    }

    if (validationError) {
      return;
    }

    setIsCalculating(true);
    try {
      if (conversionTarget === 'value') {
        const value = parseFloat(inputValue);
        if (isNaN(value)) {
          setValidationError('Please enter a valid number');
          return;
        }

        // Use the backend conversion function
        const conversionResult = await convertValue(value, from, to);

        if (conversionResult) {
          setResult(conversionResult.formatted_result);
          // Quick success feedback
          setTimeout(() => {
            setResult(prev => prev ? `✅ ${prev}` : '');
          }, 100);
        } else {
          setValidationError('Conversion failed');
          setResult('');
        }
      } else {
        // Spreadsheet range conversion
        if (!spreadsheetRange) {
          setValidationError('Please enter a valid cell range');
          return;
        }

        try {
          const rangeRequest: RangeConversionRequest = {
            range: spreadsheetRange,
            from_unit: from,
            to_unit: to
          };

          const rangeResult: RangeConversionResult = await invoke('convert_spreadsheet_range', { request: rangeRequest });
          setResult(`Range conversion: ${rangeResult.preview}`);
        } catch (error) {
          setValidationError(`Range conversion error: ${error}`);
          setResult('');
        }
      }
    } catch (error) {
      setValidationError(`Conversion error: ${error}`);
      setResult('');
    } finally {
      setIsCalculating(false);
    }
  };

  const handleClose = async () => {
    try {
      if (windowRef.current) {
        await windowRef.current.close();
      }
    } catch (error) {
      // Failed to close window
    }
  };

  const testCustomUnit = async (unitStr: string, unitType: 'from' | 'to') => {
    if (!unitStr) return;

    try {
      const isValid: boolean = await invoke('validate_unit_string', { unit: unitStr });
      if (isValid) {
        const analysis = await parseUnitFormula(unitStr);
        if (analysis && analysis.is_valid) {
          alert(`✅ ${unitType === 'from' ? 'From' : 'To'} unit: ${unitStr}\n📐 ${analysis.dimensional_formula}\n⚙️ SI Factor: ${analysis.si_factor.toExponential(3)}`);
        } else {
          alert(`✅ ${unitType === 'from' ? 'From' : 'To'} unit: ${unitStr} is valid`);
        }
      } else {
        alert(`❌ Invalid unit: ${unitStr}`);
      }
    } catch (error) {
      alert(`❌ Parse error: ${error}`);
    }
  };

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        sx={{
          width: '100vw',
          minHeight: '100vh',
          background: '#0a0a0a',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {/* Custom Draggable Title Bar */}
        <Box
          data-tauri-drag-region
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            height: 32,
            px: 1.5,
            background: 'linear-gradient(135deg, rgba(59, 130, 246, 0.08) 0%, rgba(239, 68, 68, 0.08) 100%)',
            backdropFilter: 'blur(20px)',
            borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
            flexShrink: 0,
          }}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Typography variant="body2" component="div" fontWeight="bold" color="#ffffff">
              ⚖️ Unit Conversion
            </Typography>
          </Box>
          <IconButton
            onClick={handleClose}
            size="small"
            sx={{
              color: '#ffffff',
              padding: '2px',
              '&:hover': {
                color: 'error.main',
                backgroundColor: 'rgba(239, 68, 68, 0.1)',
              }
            }}
          >
            <Close fontSize="small" />
          </IconButton>
        </Box>

        {/* Main Content */}
        <Box
          ref={contentRef}
          sx={{
            flex: '0 1 auto',
            p: 1.5, // Further reduced padding
            pb: 0.5, // Minimal bottom padding
            minHeight: 'fit-content',
            maxWidth: '100%',
          }}
        >
          {/* Unit Selection Mode */}
          <Paper sx={{ p: 2, mb: 2 }}>
            <Typography variant="h6" sx={{ mb: 1.5, color: '#ffffff' }}>
              Unit Selection Mode
            </Typography>
            <Box sx={{ display: 'flex', gap: 1, mb: 1 }}>
              <Chip
                label="Standard Units"
                icon={<Science />}
                variant={unitMode === 'standard' ? 'filled' : 'outlined'}
                onClick={() => setUnitMode('standard')}
                sx={{ cursor: 'pointer' }}
              />
              <Chip
                label="Custom Expressions"
                icon={<Calculate />}
                variant={unitMode === 'custom' ? 'filled' : 'outlined'}
                onClick={() => setUnitMode('custom')}
                sx={{ cursor: 'pointer' }}
              />
            </Box>
          </Paper>          {/* Conversion Target */}
          <Paper sx={{ p: 2, mb: 2 }}>
            <Typography variant="h6" sx={{ mb: 1.5, color: '#ffffff' }}>
              Conversion Target
            </Typography>
            <RadioGroup
              row
              value={conversionTarget}
              onChange={(e) => setConversionTarget(e.target.value as 'value' | 'range')}
              sx={{ mb: 1 }}
            >
              <FormControlLabel
                value="value"
                control={<Radio size="small" />}
                label="Single Value"
                sx={{ color: '#ffffff', mr: 3 }}
              />
              <FormControlLabel
                value="range"
                control={<Radio size="small" />}
                label="Spreadsheet Range"
                sx={{ color: '#ffffff' }}
              />
            </RadioGroup>
          </Paper>

          {/* Unit Setup */}
          <Paper sx={{ p: 2, mb: 2 }}>
            <Typography variant="h6" sx={{ mb: 1.5, color: '#ffffff' }}>
              ⚖️ Unit Setup
            </Typography>

            {unitMode === 'standard' ? (
              <>
                {/* Category Selection */}
                <FormControl fullWidth size="small" sx={{ mb: 2 }}>
                  <InputLabel sx={{ color: 'rgba(255,255,255,0.7)' }}>Category</InputLabel>
                  <Select
                    value={selectedCategory}
                    label="Category"
                    disabled={loadingUnits}
                    renderValue={(value) => (
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                        <span>{getCategoryIcon(value)}</span>
                        <span>{loadingUnits ? 'Loading...' : formatCategoryName(value)}</span>
                      </Box>
                    )}
                    onChange={(e) => {
                      const newCategory = e.target.value;
                      setSelectedCategory(newCategory);
                      const availableUnitsForCategory = getAvailableUnits(newCategory);
                      if (availableUnitsForCategory.length > 0) {
                        // Set better defaults based on category
                        const category = newCategory.toLowerCase();
                        let defaultFrom = availableUnitsForCategory[0].symbol;
                        let defaultTo = availableUnitsForCategory[1]?.symbol || availableUnitsForCategory[0].symbol;

                        // Category-specific defaults
                        if (category === 'length') {
                          const meter = availableUnitsForCategory.find(u => u.symbol === 'm');
                          const foot = availableUnitsForCategory.find(u => u.symbol === 'ft');
                          if (meter) defaultFrom = meter.symbol;
                          if (foot) defaultTo = foot.symbol;
                        } else if (category === 'mass') {
                          const kg = availableUnitsForCategory.find(u => u.symbol === 'kg');
                          const lb = availableUnitsForCategory.find(u => u.symbol === 'lb');
                          if (kg) defaultFrom = kg.symbol;
                          if (lb) defaultTo = lb.symbol;
                        } else if (category === 'temperature') {
                          const celsius = availableUnitsForCategory.find(u => u.symbol === '°C');
                          const fahrenheit = availableUnitsForCategory.find(u => u.symbol === '°F');
                          if (celsius) defaultFrom = celsius.symbol;
                          if (fahrenheit) defaultTo = fahrenheit.symbol;
                        } else if (category === 'velocity') {
                          const kmh = availableUnitsForCategory.find(u => u.symbol === 'km/h');
                          const ms = availableUnitsForCategory.find(u => u.symbol === 'm/s');
                          if (kmh) defaultFrom = kmh.symbol;
                          if (ms) defaultTo = ms.symbol;
                        }

                        setFromUnit(defaultFrom);
                        setToUnit(defaultTo);
                      }
                    }}
                  >
                    {getAvailableCategories().map((categoryName) => (
                      <MenuItem key={categoryName} value={categoryName} sx={{ color: '#ffffff' }}>
                        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                          <span>{getCategoryIcon(categoryName)}</span>
                          <span>{formatCategoryName(categoryName)}</span>
                        </Box>
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>

                {/* Unit Selection */}
                <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
                  <Box sx={{ flex: '1' }}>
                    <FormControl fullWidth size="small">
                      <InputLabel sx={{ color: 'rgba(255,255,255,0.7)' }}>From</InputLabel>
                      <Select
                        value={fromUnit}
                        label="From"
                        disabled={loadingUnits}
                        onChange={(e) => setFromUnit(e.target.value)}
                        renderValue={(value) => {
                          // Search in all units, not just filtered ones, for renderValue
                          const unit = Object.values(availableUnits).find(u => u.symbol === value);
                          return unit ? (
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                              <span>{getUnitIcon(unit)}</span>
                              <span><strong>{unit.symbol}</strong> - {formatUnitName(unit.name)}</span>
                            </Box>
                          ) : value;
                        }}
                      >
                        {getAvailableUnits().map((unit) => (
                          <MenuItem key={unit.symbol} value={unit.symbol} sx={{ color: '#ffffff' }}>
                            <Tooltip title={unit.description} arrow>
                              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                                <span>{getUnitIcon(unit)}</span>
                                <Box>
                                  <strong>{unit.symbol}</strong> - {formatUnitName(unit.name)}
                                </Box>
                              </Box>
                            </Tooltip>
                          </MenuItem>
                        ))}
                      </Select>
                    </FormControl>
                  </Box>

                  <Box sx={{ display: 'flex', justifyContent: 'center', minWidth: '40px' }}>
                    <IconButton
                      disabled={loadingUnits}
                      onClick={() => {
                        const temp = fromUnit;
                        setFromUnit(toUnit);
                        setToUnit(temp);
                      }}
                      sx={{ color: '#3b82f6' }}
                    >
                      <SwapHoriz />
                    </IconButton>
                  </Box>

                  <Box sx={{ flex: '1' }}>
                    <FormControl fullWidth size="small">
                      <InputLabel sx={{ color: 'rgba(255,255,255,0.7)' }}>To</InputLabel>
                      <Select
                        value={toUnit}
                        label="To"
                        disabled={loadingUnits}
                        onChange={(e) => setToUnit(e.target.value)}
                        renderValue={(value) => {
                          // Search in all units, not just filtered ones, for renderValue
                          const unit = Object.values(availableUnits).find(u => u.symbol === value);
                          return unit ? (
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                              <span>{getUnitIcon(unit)}</span>
                              <span><strong>{unit.symbol}</strong> - {formatUnitName(unit.name)}</span>
                            </Box>
                          ) : value;
                        }}
                      >
                        {getAvailableUnits().map((unit) => (
                          <MenuItem key={unit.symbol} value={unit.symbol} sx={{ color: '#ffffff' }}>
                            <Tooltip title={unit.description} arrow>
                              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                                <span>{getUnitIcon(unit)}</span>
                                <Box>
                                  <strong>{unit.symbol}</strong> - {formatUnitName(unit.name)}
                                </Box>
                              </Box>
                            </Tooltip>
                          </MenuItem>
                        ))}
                      </Select>
                    </FormControl>
                  </Box>
                </Box>
              </>
            ) : (
              <>
                {/* Custom Unit Expressions */}
                <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 2 }}>
                  <Box sx={{ flex: '1' }}>
                    <TextField
                      fullWidth
                      size="small"
                      label="From Unit Expression"
                      value={customFromUnit}
                      onChange={(e) => setCustomFromUnit(e.target.value)}
                      placeholder="kg*m/s**2"
                    />
                  </Box>

                  <Box sx={{ display: 'flex', justifyContent: 'center', minWidth: '40px' }}>
                    <IconButton
                      onClick={() => {
                        const temp = customFromUnit;
                        setCustomFromUnit(customToUnit);
                        setCustomToUnit(temp);
                      }}
                      sx={{ color: '#3b82f6' }}
                    >
                      <SwapHoriz />
                    </IconButton>
                  </Box>

                  <Box sx={{ flex: '1' }}>
                    <TextField
                      fullWidth
                      size="small"
                      label="To Unit Expression"
                      value={customToUnit}
                      onChange={(e) => setCustomToUnit(e.target.value)}
                      placeholder="N"
                    />
                  </Box>
                </Box>

                <Box sx={{ display: 'flex', gap: 1 }}>
                  <Button
                    variant="outlined"
                    size="small"
                    onClick={() => testCustomUnit(customFromUnit, 'from')}
                    sx={{ color: '#ffffff', borderColor: '#ffffff' }}
                  >
                    🔍 Test From
                  </Button>
                  <Button
                    variant="outlined"
                    size="small"
                    onClick={() => testCustomUnit(customToUnit, 'to')}
                    sx={{ color: '#ffffff', borderColor: '#ffffff' }}
                  >
                    🔍 Test To
                  </Button>
                </Box>
              </>
            )}

            {/* Status Display */}
            {(unitCompatibility || conversionPreview) && (
              <Box sx={{ mt: 2, p: 1, bgcolor: 'rgba(255, 255, 255, 0.05)', borderRadius: 1 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                  {unitCompatibility && (
                    <Typography variant="body2" sx={{
                      color: unitCompatibility.includes('✅') ? '#10b981' :
                             unitCompatibility.includes('❌') ? '#ef4444' : '#f59e0b',
                      fontWeight: 'bold',
                      fontSize: '0.75rem'
                    }}>
                      {unitCompatibility}
                    </Typography>
                  )}
                  {conversionPreview && (
                    <Typography variant="body2" sx={{
                      fontFamily: 'monospace',
                      color: '#60a5fa',
                      fontSize: '0.7rem'
                    }}>
                      {conversionPreview}
                    </Typography>
                  )}
                </Box>
              </Box>
            )}
          </Paper>

          {/* Convert Section */}
          <Paper sx={{ p: 2, mb: 2 }}>
            <Typography variant="h6" sx={{ mb: 1.5, color: '#ffffff' }}>
              🎯 Convert
            </Typography>

            <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
              <Box sx={{ flex: '2' }}>
                {conversionTarget === 'value' ? (
                  <TextField
                    fullWidth
                    size="small"
                    label="Value"
                    value={inputValue}
                    onChange={(e) => setInputValue(e.target.value)}
                    type="number"
                    placeholder="100"
                  />
                ) : (
                  <TextField
                    fullWidth
                    size="small"
                    label="Cell Range"
                    value={spreadsheetRange}
                    onChange={(e) => setSpreadsheetRange(e.target.value)}
                    placeholder="A1:A10"
                  />
                )}
              </Box>
              <Box sx={{ flex: '1' }}>
                <Button
                  onClick={handleCalculate}
                  variant="contained"
                  disabled={isCalculating || validationError !== ''}
                  fullWidth
                  size="small"
                  sx={{
                    py: 1,
                    background: 'linear-gradient(45deg, #10b981 30%, #059669 90%)',
                    '&:hover': {
                      background: 'linear-gradient(45deg, #059669 30%, #047857 90%)',
                    }
                  }}
                >
                  {isCalculating ? '🔄 Converting...' :
                   conversionTarget === 'range' ? '🔄 Convert Range' : '⚖️ Convert'}
                </Button>
              </Box>
            </Box>
          </Paper>

          {/* Results */}
          {result && (
            <Alert severity="success" sx={{ mb: 0 }}>
              <Typography variant="body1" fontWeight="bold" color="#ffffff">
                {result}
              </Typography>
            </Alert>
          )}

          {/* Validation Error */}
          {validationError && (
            <Alert severity="error" sx={{ mb: 0 }}>
              <Typography variant="body2" color="#ffffff">
                {validationError}
              </Typography>
            </Alert>
          )}
        </Box>
      </Box>
    </ThemeProvider>
  );
}

export default UnitConversionWindow;

// Initialization
const renderUnitConversionWindow = () => {
  const container = document.getElementById('root');
  if (container) {
    try {
      const root = createRoot(container);
      root.render(<UnitConversionWindow />);
    } catch (error) {
      // UnitConversionWindow: Error rendering
    }
  } else {
    // UnitConversionWindow: Root container not found
  }
};

// Auto-render when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', renderUnitConversionWindow);
} else {
  renderUnitConversionWindow();
}

// Fallback for window load event
window.addEventListener('load', renderUnitConversionWindow);
