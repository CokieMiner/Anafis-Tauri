import { ContentCopy as ContentCopyIcon } from '@mui/icons-material';
import {
  Box,
  Button,
  CssBaseline,
  FormControl,
  FormControlLabel,
  FormLabel,
  IconButton,
  InputAdornment,
  Paper,
  Radio,
  RadioGroup,
  Stack,
  TextField,
  ThemeProvider,
  Typography,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { memo, useCallback, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import 'katex/dist/katex.min.css';
import { BlockMath } from 'react-katex';
import CustomTitleBar from '@/shared/components/CustomTitleBar';
import { createNoTransitionTheme } from '@/tabs/spreadsheet/components/sidebar/themes';
import VariableManager from '@/windows/uncertaintyCalculator/components/VariableManager';

const theme = createNoTransitionTheme();

interface Variable {
  id: string;
  name: string;
  value: string;
  uncertainty: string;
}

// Custom hook for variable management
const useVariableManager = () => {
  const defaultVariables = [
    { id: 'var-0', name: 'a', value: '', uncertainty: '' },
  ];
  const [variablesInput, setVariablesInput] = useState('a');
  const [variables, setVariables] = useState<Variable[]>(defaultVariables);
  const [selectedVariableIndex, setSelectedVariableIndex] = useState(0);
  const variablesRef = useRef<Variable[]>(defaultVariables);

  const handleVariablesInputChange = useCallback(
    (value: string) => {
      setVariablesInput(value);

      const newVariableNames = value
        .split(',')
        .map((v) => v.trim())
        .filter(Boolean);
      const updatedVariables: Variable[] = [];
      const existingValues: Record<
        string,
        { value: string; uncertainty: string }
      > = {};

      variablesRef.current.forEach((v) => {
        existingValues[v.name] = { value: v.value, uncertainty: v.uncertainty };
      });

      newVariableNames.forEach((name) => {
        // Generate a stable ID based on the variable name to ensure consistency
        const id = `var-${name}`;
        updatedVariables.push({
          id,
          name,
          value: existingValues[name]?.value ?? '',
          uncertainty: existingValues[name]?.uncertainty ?? '',
        });
      });

      setVariables(updatedVariables);
      variablesRef.current = updatedVariables;

      if (selectedVariableIndex >= updatedVariables.length) {
        setSelectedVariableIndex(Math.max(0, updatedVariables.length - 1));
      }
    },
    [selectedVariableIndex]
  );

  const updateVariable = useCallback(
    (index: number, field: keyof Variable, value: string) => {
      // Only allow updating specific fields to maintain stable ID invariant
      const allowedFields: (keyof Variable)[] = [
        'name',
        'value',
        'uncertainty',
      ];
      if (!allowedFields.includes(field)) {
        return; // Return unchanged if field is not allowed
      }

      setVariables((prev) => {
        const updated = [...prev];
        if (updated[index]) {
          // Clone the variable object to avoid mutating existing objects
          updated[index] = { ...updated[index], [field]: value };
          variablesRef.current = updated;
        }
        return updated;
      });
    },
    []
  );

  return {
    variablesInput,
    variables,
    selectedVariableIndex,
    setSelectedVariableIndex,
    handleVariablesInputChange,
    updateVariable,
  };
};

// Memoized result display component
const ResultDisplay = memo(
  ({
    mode,
    calculationResult,
    stringRepresentation,
    latexFormula,
    onOpenLatexPreview,
  }: {
    mode: 'calculate' | 'propagate';
    calculationResult: string;
    stringRepresentation: string;
    latexFormula: string;
    onOpenLatexPreview: () => Promise<void>;
  }) => {
    if (mode === 'calculate') {
      return (
        <Paper
          elevation={0}
          sx={{
            p: 2,
            backgroundColor: 'background.paper',
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
          }}
        >
          <Typography
            variant="h6"
            sx={{
              mb: 1.5,
              color: 'text.primary',
              fontWeight: 'bold',
              fontSize: '0.95rem',
            }}
          >
            Calculation Result
          </Typography>
          <Typography
            component="pre"
            sx={{
              color: 'text.primary',
              fontFamily: 'monospace',
              whiteSpace: 'pre-wrap',
              fontSize: '0.85rem',
              margin: 0,
            }}
          >
            {calculationResult}
          </Typography>
        </Paper>
      );
    }

    return (
      <Stack spacing={1}>
        <Paper
          elevation={0}
          sx={{
            p: 1.5,
            backgroundColor: 'background.paper',
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
          }}
        >
          <Typography
            variant="subtitle2"
            sx={{
              mb: 1,
              color: 'text.primary',
              fontWeight: 'bold',
              fontSize: '0.9rem',
            }}
          >
            String Representation
          </Typography>
          <TextField
            value={stringRepresentation}
            slotProps={{
              input: {
                readOnly: true,
                endAdornment: (
                  <InputAdornment position="end">
                    <IconButton
                      size="small"
                      onClick={() =>
                        void navigator.clipboard.writeText(stringRepresentation)
                      }
                    >
                      <ContentCopyIcon fontSize="small" />
                    </IconButton>
                  </InputAdornment>
                ),
              },
            }}
            variant="outlined"
            size="small"
            multiline
            minRows={1}
            maxRows={2}
            fullWidth
            sx={{ '& .MuiOutlinedInput-input': { fontSize: '0.75rem' } }}
          />
        </Paper>

        <Paper
          elevation={0}
          sx={{
            p: 1.5,
            backgroundColor: 'background.paper',
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
          }}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
            <Typography
              variant="subtitle2"
              sx={{
                color: 'text.primary',
                fontWeight: 'bold',
                fontSize: '0.9rem',
              }}
            >
              LaTeX Formula
            </Typography>
          </Box>
          <TextField
            value={latexFormula}
            slotProps={{
              input: {
                readOnly: true,
                endAdornment: (
                  <InputAdornment position="end">
                    <IconButton
                      size="small"
                      onClick={() =>
                        void navigator.clipboard.writeText(latexFormula)
                      }
                    >
                      <ContentCopyIcon fontSize="small" />
                    </IconButton>
                  </InputAdornment>
                ),
              },
            }}
            variant="outlined"
            size="small"
            multiline
            minRows={1}
            maxRows={2}
            fullWidth
            sx={{ '& .MuiOutlinedInput-input': { fontSize: '0.75rem' } }}
          />
        </Paper>

        <Paper
          elevation={0}
          sx={{
            p: 1.5,
            backgroundColor: 'background.paper',
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
          }}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
            <Typography
              variant="subtitle2"
              sx={{
                color: 'text.primary',
                fontWeight: 'bold',
                fontSize: '0.9rem',
              }}
            >
              Rendered Formula
            </Typography>
            <Button
              variant="outlined"
              size="small"
              onClick={() => void onOpenLatexPreview()}
              disabled={!latexFormula || latexFormula.startsWith('Error:')}
              sx={{ minWidth: 'auto', px: 1, py: 0.25, fontSize: '0.7rem' }}
            >
              Full View
            </Button>
          </Box>
          <Box
            sx={{
              p: 1,
              backgroundColor: 'background.default',
              borderRadius: 1,
              minHeight: '40px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            {latexFormula && !latexFormula.startsWith('Error:') ? (
              <Box sx={{ '& .katex': { fontSize: '0.8em' } }}>
                <BlockMath math={latexFormula} />
              </Box>
            ) : (
              <Typography
                sx={{
                  color: 'text.secondary',
                  fontSize: '0.8rem',
                  fontStyle: 'italic',
                }}
              >
                Formula will render here
              </Typography>
            )}
          </Box>
        </Paper>
      </Stack>
    );
  }
);

ResultDisplay.displayName = 'ResultDisplay';

function UncertaintyCalculatorWindow() {
  const [formula, setFormula] = useState('');
  const [mode, setMode] = useState<'calculate' | 'propagate'>('calculate');
  const [calculationResult, setCalculationResult] = useState(
    'Value: N/A\nUncertainty: N/A'
  );
  const [stringRepresentation, setStringRepresentation] = useState('');
  const [latexFormula, setLatexFormula] = useState('');

  const {
    variablesInput,
    variables,
    selectedVariableIndex,
    setSelectedVariableIndex,
    handleVariablesInputChange,
    updateVariable,
  } = useVariableManager();

  const handleOpenLatexPreview = useCallback(async () => {
    if (!latexFormula || latexFormula.startsWith('Error:')) {
      alert('Please generate a valid LaTeX formula first.');
      return;
    }

    try {
      await invoke('open_latex_preview_window', {
        latexFormula,
        title: 'LaTeX Formula Preview',
      });
    } catch (error) {
      console.error('Error opening LaTeX preview window:', error);
      alert(`Failed to open LaTeX preview window: ${String(error)}`);
    }
  }, [latexFormula]);

  const handleCalculate = useCallback(async () => {
    if (!formula) {
      alert('Please enter a formula.');
      return;
    }
    if (variables.some((v) => !v.value)) {
      alert('Please enter values for all variables.');
      return;
    }

    try {
      const backendVariables = variables.map((v) => ({
        name: v.name,
        value: parseFloat(v.value),
        uncertainty: parseFloat(v.uncertainty || '0'),
      }));

      const result = await invoke('calculate_uncertainty', {
        formula,
        variables: backendVariables,
      });

      const calculationResult = result as {
        value: number;
        uncertainty: number;
      };
      const displayValue: string = calculationResult.value.toPrecision(6);
      setCalculationResult(
        `Value: ${displayValue}\nUncertainty: ${calculationResult.uncertainty.toPrecision(6)}`
      );
    } catch (error) {
      console.error('Calculation error:', error);
      setCalculationResult(`Error: ${String(error)}`);
    }
  }, [formula, variables]);

  const handleGenerateLatex = useCallback(async () => {
    if (!formula) {
      alert('Please enter a formula.');
      return;
    }

    try {
      const variableNames = variables.map((v) => v.name);
      const result = await invoke('generate_latex', {
        formula,
        variables: variableNames,
      });

      const latexResult = result as { string: string; latex: string };
      setStringRepresentation(latexResult.string);
      setLatexFormula(latexResult.latex);
    } catch (error) {
      console.error('LaTeX generation error:', error);
      setStringRepresentation(`Error: ${String(error)}`);
      setLatexFormula(`Error: ${String(error)}`);
    }
  }, [formula, variables]);

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        sx={{
          width: '100%',
          height: '100vh',
          display: 'flex',
          flexDirection: 'column',
          bgcolor: 'background.default',
          overflow: 'hidden',
        }}
      >
        <CustomTitleBar title="Uncertainty Calculator" />

        {/* Formula Input Section */}
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
          <TextField
            label="Formula"
            placeholder="Enter your mathematical formula here (e.g., x*sin(y) + z)"
            value={formula}
            onChange={(e) => setFormula(e.target.value)}
            variant="outlined"
            multiline
            minRows={2}
            maxRows={4}
            size="small"
            fullWidth
            sx={{ '& .MuiOutlinedInput-input': { fontFamily: 'monospace' } }}
          />
          <Typography
            sx={{
              color: 'text.secondary',
              fontSize: '0.8rem',
              mt: 0.5,
              fontStyle: 'italic',
            }}
          >
            Examples: x+y, x*y^2, sqrt(x^2+y^2), sin(x), cos(y), exp(z)
          </Typography>
        </Box>

        {/* Variables Input */}
        <Box sx={{ px: 2, py: 1, borderBottom: 1, borderColor: 'divider' }}>
          <TextField
            label="Variables (comma-separated)"
            placeholder="Enter variable names separated by commas (e.g., x, y, z)"
            value={variablesInput}
            onChange={(e) => handleVariablesInputChange(e.target.value)}
            variant="outlined"
            size="small"
            fullWidth
            sx={{ '& .MuiOutlinedInput-input': { fontFamily: 'monospace' } }}
          />
        </Box>

        {/* Mode Selection and Action Button */}
        <Box
          sx={{
            px: 2,
            py: 1,
            borderBottom: 1,
            borderColor: 'divider',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: 2,
          }}
        >
          <FormControl component="fieldset">
            <FormLabel
              component="legend"
              sx={{
                color: 'text.primary',
                fontWeight: 'bold',
                fontSize: '0.9rem',
              }}
            >
              Mode
            </FormLabel>
            <RadioGroup
              row
              value={mode}
              onChange={(e) =>
                setMode(e.target.value as 'calculate' | 'propagate')
              }
              sx={{ gap: 3 }}
            >
              <FormControlLabel
                value="calculate"
                control={<Radio size="small" />}
                label="Calculate Value"
                sx={{ '& .MuiFormControlLabel-label': { fontSize: '0.8rem' } }}
              />
              <FormControlLabel
                value="propagate"
                control={<Radio size="small" />}
                label="Generate LaTeX"
                sx={{ '& .MuiFormControlLabel-label': { fontSize: '0.8rem' } }}
              />
            </RadioGroup>
          </FormControl>
          <Button
            variant="contained"
            onClick={() =>
              void (mode === 'calculate'
                ? handleCalculate()
                : handleGenerateLatex())
            }
            size="small"
            sx={{ minWidth: '140px' }}
          >
            {mode === 'calculate' ? 'Calculate Result' : 'Generate LaTeX'}
          </Button>
        </Box>

        {/* Main Content */}
        <Box sx={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
          {mode === 'calculate' && (
            <VariableManager
              variables={variables}
              selectedIndex={selectedVariableIndex}
              onVariableSelect={setSelectedVariableIndex}
              onVariableUpdate={updateVariable}
            />
          )}

          <Box sx={{ flex: 1, p: 2, overflow: 'auto' }}>
            <ResultDisplay
              mode={mode}
              calculationResult={calculationResult}
              stringRepresentation={stringRepresentation}
              latexFormula={latexFormula}
              onOpenLatexPreview={handleOpenLatexPreview}
            />
          </Box>
        </Box>
      </Box>
    </ThemeProvider>
  );
}

export default UncertaintyCalculatorWindow;

// Auto-render immediately when this module loads
const renderUncertaintyCalculatorWindow = () => {
  const container = document.getElementById('root');
  if (container) {
    try {
      const root = createRoot(container);
      root.render(<UncertaintyCalculatorWindow />);
    } catch {
      // UncertaintyCalculatorWindow: Error rendering
    }
  } else {
    // UncertaintyCalculatorWindow: Root container not found
  }
};

// Try to render immediately
if (document.readyState === 'complete') {
  renderUncertaintyCalculatorWindow();
} else {
  // Wait for DOM to be ready
  window.addEventListener('load', renderUncertaintyCalculatorWindow);
}
