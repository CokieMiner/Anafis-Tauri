import React, { useState, useEffect } from 'react';

// Material-UI Imports
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import RadioGroup from '@mui/material/RadioGroup';
import FormControlLabel from '@mui/material/FormControlLabel';
import Radio from '@mui/material/Radio';
import FormControl from '@mui/material/FormControl';
import FormLabel from '@mui/material/FormLabel';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Paper from '@mui/material/Paper';

interface UncertaintyCalculatorDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const UncertaintyCalculatorDialog: React.FC<UncertaintyCalculatorDialogProps> = ({ isOpen, onClose: _onClose }) => {
  const [formula, setFormula] = useState('');
  const [variablesInput, setVariablesInput] = useState('');
  const [mode, setMode] = useState<'calculate' | 'propagate'>('calculate');
  const [variableValues, setVariableValues] = useState<Record<string, { value: string; uncertainty: string }>>({});
  const [calculationResult, setCalculationResult] = useState('Value: N/A\nUncertainty: N/A');
  const [stringRepresentation, setStringRepresentation] = useState('');
  const [latexFormula, setLatexFormula] = useState('');

  // Effect to update dynamic variable inputs when variablesInput changes
  useEffect(() => {
    const newVariables = variablesInput.split(',').map(v => v.trim()).filter(Boolean);
    const updatedVariableValues: Record<string, { value: string; uncertainty: string }> = {};

    newVariables.forEach(v => {
      updatedVariableValues[v] = variableValues[v] || { value: '', uncertainty: '' };
    });
    setVariableValues(updatedVariableValues);
  }, [variablesInput]);

  if (!isOpen) return null;

  const handleCalculate = () => {
    if (!formula) {
      alert('Please enter a formula.');
      return;
    }
    if (Object.keys(variableValues).length === 0) {
      alert('Please enter variables and their values/uncertainties.');
      return;
    }

    // Simulate calculation
    const simulatedValue = Math.random() * 100;
    const simulatedUncertainty = Math.random() * 5;
    setCalculationResult(`Value: ${simulatedValue.toPrecision(6)}\nUncertainty: ${simulatedUncertainty.toPrecision(6)}`);
  };

  const handleGenerateLatex = () => {
    if (!formula) {
      alert('Please enter a formula.');
      return;
    }
    if (!variablesInput) {
      alert('Please enter variable names.');
      return;
    }

    // Simulate LaTeX generation
    const simulatedString = `Uncertainty[${formula}] = ...`;
    const simulatedLatex = `\\Delta ${formula} = \\\sqrt{\\sum_{i} \\left( \\frac{\\partial f}{\\partial x_i} \\Delta x_i \\right)^2}`; // Generic formula
    setStringRepresentation(simulatedString);
    setLatexFormula(simulatedLatex);
  };

  // Notify outer window that content changed so it can resize immediately
  // Note: do NOT dispatch on simple mode toggles (to avoid immediate resize on Mode change)
  useEffect(() => {
    const ev = new Event('anafis_content_change');
    document.dispatchEvent(ev);
  }, [stringRepresentation, latexFormula, Object.keys(variableValues).length]);

  return (
    <Box
      sx={{
        width: '100%',
        boxSizing: 'border-box',
  minWidth: '400px',
  maxWidth: '500px',
        '& *': { boxSizing: 'border-box' },
      }}
    >
      <Box sx={{ p: 1, boxSizing: 'border-box' }}>
        {/* Formula Input */}
        <Box sx={{ mb: 1.5 }}>
          <TextField
            label="Formula (e.g., x*sin(y) + z)"
            placeholder="Enter your mathematical formula here"
            value={formula}
            onChange={(e) => setFormula(e.target.value)}
            variant="outlined"
            multiline
            minRows={2}
            size="small"
            sx={{
              width: '100%',
              '& .MuiOutlinedInput-root': {
                backgroundColor: 'background.default',
                '& fieldset': { borderColor: 'divider' },
                '&:hover fieldset': { borderColor: 'primary.light' },
                '&.Mui-focused fieldset': { borderColor: 'primary.light' },
              },
              '& .MuiInputLabel-root': {
                color: 'text.secondary',
                fontSize: '0.9rem',
                '&.Mui-focused': { color: '#ffffff !important' }
              },
              '& .MuiOutlinedInput-input': { color: 'text.primary', fontSize: '0.9rem', padding: '8px 10px', lineHeight: 1.25 },
            }}
          />
        </Box>

        {/* Variables Input */}
        <Box sx={{ mb: 1.5 }}>
          <TextField
            label="Variables (comma-separated, e.g., x, y, z)"
            placeholder="Enter variable names"
            value={variablesInput}
            onChange={(e) => setVariablesInput(e.target.value)}
            variant="outlined"
            multiline
            minRows={2}
            size="small"
            sx={{
              width: '100%',
              '& .MuiOutlinedInput-root': {
                backgroundColor: 'background.default',
                '& fieldset': { borderColor: 'divider' },
                '&:hover fieldset': { borderColor: 'primary.light' },
                '&.Mui-focused fieldset': { borderColor: 'primary.light' },
              },
              '& .MuiInputLabel-root': {
                color: 'text.secondary',
                fontSize: '0.9rem',
                '&.Mui-focused': { color: '#ffffff !important' }
              },
              '& .MuiOutlinedInput-input': { color: 'text.primary', fontSize: '0.9rem', padding: '8px 10px', lineHeight: 1.25 },
            }}
          />
        </Box>

        {/* Mode Selection */}
        <Paper elevation={0} sx={{ p: 1.5, mb: 2, backgroundColor: 'background.default', border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
          <FormControl component="fieldset">
            <FormLabel
              component="legend"
              sx={{
                color: 'text.primary',
                fontWeight: 'bold',
                mb: 1,
                fontSize: '0.85rem',
                '&, &:hover, &.Mui-focused': { color: 'text.primary' },
                cursor: 'default',
                transition: 'none',
              }}
            >
              Mode
            </FormLabel>
            <RadioGroup row value={mode} onChange={(e) => setMode(e.target.value as 'calculate' | 'propagate')} sx={{ gap: 1.5, width: '100%', justifyContent: 'center' }}>
              <FormControlLabel
                value="calculate"
                control={<Radio size="small" sx={{ color: '#ffffff', '&.Mui-checked': { color: '#ffffff' } }} />}
                label="Calculate Value"
                sx={{
                  color: 'text.primary',
                  minWidth: '140px',
                  '& .MuiFormControlLabel-label': { color: 'text.primary', fontSize: '0.8rem' },
                  '&.Mui-checked, &.Mui-selected': {
                    color: 'text.primary',
                    '& .MuiFormControlLabel-label': { color: 'text.primary' },
                  },
                }}
              />
              <FormControlLabel
                value="propagate"
                control={<Radio size="small" sx={{ color: '#ffffff', '&.Mui-checked': { color: '#ffffff' } }} />}
                label="Propagate Formula"
                sx={{
                  color: 'text.primary',
                  minWidth: '140px',
                  '& .MuiFormControlLabel-label': { color: 'text.primary', fontSize: '0.8rem' },
                  '&.Mui-checked, &.Mui-selected': {
                    color: 'text.primary',
                    '& .MuiFormControlLabel-label': { color: 'text.primary' },
                  },
                }}
              />
            </RadioGroup>
          </FormControl>
        </Paper>

        {/* Dynamic Input/Output Area */}
        {mode === 'calculate' && (
          <Paper elevation={0} sx={{ p: 1.5, mb: 1.5, backgroundColor: 'background.default', border: '1px solid', borderColor: 'divider', borderRadius: 1, boxSizing: 'border-box' }}>
            <Typography variant="h6" sx={{ mb: 1.5, color: 'text.primary', fontWeight: 'bold', fontSize: '0.9rem' }}>Variable, Value and Uncertaintie</Typography>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.25 }}>
              {Object.keys(variableValues).map(varName => (
                <Box key={varName} sx={{ display: 'flex', alignItems: 'center', gap: 1, flexWrap: 'wrap' }}>
                  <Typography sx={{ minWidth: '48px', color: 'text.primary', fontWeight: 'medium', fontSize: '0.8rem' }}>{varName}:</Typography>
                  <TextField
                    type="number"
                    placeholder={`Value of ${varName}`}
                    value={variableValues[varName].value}
                    onChange={(e) => setVariableValues(prev => ({ ...prev, [varName]: { ...prev[varName], value: e.target.value } }))}
                    variant="outlined"
                    size="small"
                    sx={{ flex: '1 1 160px', minWidth: '120px', '& .MuiOutlinedInput-root': { backgroundColor: 'background.paper', '& fieldset': { borderColor: 'divider' }, '&:hover fieldset': { borderColor: 'primary.light' }, '&.Mui-focused fieldset': { borderColor: 'primary.light' } }, '& .MuiOutlinedInput-input': { color: 'text.primary', fontSize: '0.8rem' } }}
                  />
                  <TextField
                    type="number"
                    placeholder={`Uncertainty of ${varName}`}
                    value={variableValues[varName].uncertainty}
                    onChange={(e) => setVariableValues(prev => ({ ...prev, [varName]: { ...prev[varName], uncertainty: e.target.value } }))}
                    variant="outlined"
                    size="small"
                    sx={{ flex: '1 1 140px', minWidth: '120px', '& .MuiOutlinedInput-root': { backgroundColor: 'background.paper', '& fieldset': { borderColor: 'divider' }, '&:hover fieldset': { borderColor: 'primary.light' }, '&.Mui-focused fieldset': { borderColor: 'primary.light' } }, '& .MuiOutlinedInput-input': { color: 'text.primary', fontSize: '0.8rem' } }}
                  />
                </Box>
              ))}
            </Box>
            <Button variant="contained" onClick={handleCalculate} size="small" sx={{ mt: 1.5, backgroundColor: 'primary.main', '&:hover': { backgroundColor: 'primary.light' }, fontSize: '0.8rem', py: 0.75 }}>Calculate Result</Button>
            <Paper elevation={0} sx={{ mt: 1.5, p: 1.5, backgroundColor: 'background.paper', border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
              <Typography variant="h6" sx={{ mb: 1, color: 'text.primary', fontWeight: 'bold', fontSize: '0.85rem' }}>Calculation Result</Typography>
              <Typography component="pre" sx={{ color: 'text.primary', fontFamily: 'monospace', whiteSpace: 'pre-wrap', fontSize: '0.75rem' }}>{calculationResult}</Typography>
            </Paper>
          </Paper>
        )}

        {mode === 'propagate' && (
          <Paper elevation={0} sx={{ p: 1.5, mb: 1.5, backgroundColor: 'background.default', border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
            <Button variant="contained" onClick={handleGenerateLatex} size="small" sx={{ mb: 1.5, backgroundColor: 'primary.main', '&:hover': { backgroundColor: 'primary.light' }, fontSize: '0.8rem', py: 0.75 }}>Generate LaTeX Formula</Button>

            <Paper elevation={0} sx={{ p: 1.5, mb: 1.5, backgroundColor: 'background.paper', border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
              <Typography variant="h6" sx={{ mb: 1.5, color: 'text.primary', fontWeight: 'bold', fontSize: '0.9rem' }}>String Representation</Typography>
              <TextField value={stringRepresentation} slotProps={{ input: { readOnly: true } }} placeholder="String representation will appear here" variant="outlined" size="small" multiline minRows={2} sx={{ width: '100%', '& .MuiOutlinedInput-root': { backgroundColor: 'background.paper', '& fieldset': { borderColor: 'divider' }, '&:hover fieldset': { borderColor: 'primary.light' }, '&.Mui-focused fieldset': { borderColor: 'primary.light' } }, '& .MuiOutlinedInput-input': { color: 'text.primary', fontSize: '0.8rem', whiteSpace: 'pre-wrap', overflowWrap: 'anywhere', wordBreak: 'break-word' } }} />
            </Paper>

            <Paper elevation={0} sx={{ p: 1.5, mb: 1.5, backgroundColor: 'background.paper', border: '1px solid', borderColor: 'divider', borderRadius: 1, boxSizing: 'border-box' }}>
              <Typography variant="h6" sx={{ mb: 1.5, color: 'text.primary', fontWeight: 'bold', fontSize: '0.9rem' }}>Generated LaTeX Formula</Typography>
              <TextField value={latexFormula} slotProps={{ input: { readOnly: true } }} placeholder="LaTeX formula will appear here" variant="outlined" size="small" multiline minRows={2} sx={{ width: '100%', '& .MuiOutlinedInput-root': { backgroundColor: 'background.paper', '& fieldset': { borderColor: 'divider' }, '&:hover fieldset': { borderColor: 'primary.light' }, '&.Mui-focused fieldset': { borderColor: 'primary.light' } }, '& .MuiOutlinedInput-input': { color: 'text.primary', fontSize: '0.8rem', whiteSpace: 'pre-wrap', overflowWrap: 'anywhere', wordBreak: 'break-word' } }} />
            </Paper>

            <Paper elevation={0} sx={{ p: 1.5, backgroundColor: 'background.paper', border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
              <Typography variant="h6" sx={{ mb: 1.5, color: 'text.primary', fontWeight: 'bold', fontSize: '0.9rem' }}>Rendered Formula (Placeholder)</Typography>
              <Typography sx={{ color: 'text.secondary', fontStyle: 'italic', fontSize: '0.8rem' }}>Rendered formula will appear here. (Requires LaTeX rendering library)</Typography>
            </Paper>
          </Paper>
        )}
      </Box>
    </Box>
  );
};

export default UncertaintyCalculatorDialog;
