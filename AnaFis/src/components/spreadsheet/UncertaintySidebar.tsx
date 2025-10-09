import React, { useState } from 'react';
import { Box, Typography, IconButton, Paper, List, ListItemButton, ListItemText, TextField, Button, Divider } from '@mui/material';
import { Close as CloseIcon, Add as AddIcon, Delete as DeleteIcon, PlayArrow as RunIcon } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { UniverSpreadsheetRef } from './UniverSpreadsheet';
import { useSpreadsheetSelection } from '../../hooks/useSpreadsheetSelection';

interface Variable {
  name: string;
  valueRange: string;
  uncertaintyRange: string;
}

type FocusedInputType = 
  | { type: 'valueRange'; varIndex: number }
  | { type: 'uncertaintyRange'; varIndex: number }
  | { type: 'outputValueRange' }
  | { type: 'outputUncertaintyRange' }
  | null;

interface UncertaintySidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef: React.RefObject<UniverSpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  onPropagationComplete?: (resultRange: string) => void;
  // Lifted state
  variables: Variable[];
  setVariables: (vars: Variable[]) => void;
  formula: string;
  setFormula: (formula: string) => void;
  outputValueRange: string;
  setOutputValueRange: (range: string) => void;
  outputUncertaintyRange: string;
  setOutputUncertaintyRange: (range: string) => void;
}

export const UncertaintySidebar: React.FC<UncertaintySidebarProps> = ({ 
  open, 
  onClose, 
  univerRef, 
  onSelectionChange,
  onPropagationComplete,
  variables,
  setVariables,
  formula,
  setFormula,
  outputValueRange,
  setOutputValueRange,
  outputUncertaintyRange,
  setOutputUncertaintyRange
}) => {
  // Remove local state - now using props
  const [selectedVariable, setSelectedVariable] = useState<number>(0);
  const [isProcessing, setIsProcessing] = useState<boolean>(false);
  const [error, setError] = useState<string>('');

  // Use the spreadsheet selection hook
  const { handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange,
    updateField: (inputType, selection) => {
      if (!inputType) return;
      
      switch (inputType.type) {
        case 'valueRange':
          updateVariable(inputType.varIndex, 'valueRange', selection);
          break;
        case 'uncertaintyRange':
          updateVariable(inputType.varIndex, 'uncertaintyRange', selection);
          break;
        case 'outputValueRange':
          setOutputValueRange(selection);
          break;
        case 'outputUncertaintyRange':
          setOutputUncertaintyRange(selection);
          break;
      }
    },
    sidebarDataAttribute: 'data-uncertainty-sidebar',
    handlerName: '__uncertaintySidebarSelectionHandler'
  });

  // Generate next variable name: a-z, then aa-zz
  const getNextVariableName = () => {
    const count = variables.length;
    if (count < 26) {
      // a-z
      return String.fromCharCode(97 + count);
    } else {
      // aa-zz
      const doubleIndex = count - 26;
      const firstChar = String.fromCharCode(97 + Math.floor(doubleIndex / 26));
      const secondChar = String.fromCharCode(97 + (doubleIndex % 26));
      return firstChar + secondChar;
    }
  };

  const addVariable = () => {
    const nextName = getNextVariableName();
    setVariables([...variables, { name: nextName, valueRange: '', uncertaintyRange: '' }]);
    setSelectedVariable(variables.length);
  };

  const removeVariable = (index: number) => {
    if (variables.length > 1) {
      setVariables(variables.filter((_, i) => i !== index));
      setSelectedVariable(index > 0 ? index - 1 : 0);
    }
  };

  const updateVariable = (index: number, field: keyof Variable, value: string) => {
    const updated = [...variables];
    updated[index][field] = value;
    setVariables(updated);
  };

  const handlePropagate = async () => {
    setError('');
    if (variables.some(v => !v.valueRange)) {
      setError('Fill in all value ranges');
      return;
    }
    if (!formula || !outputValueRange || !outputUncertaintyRange) {
      setError('Fill in formula and output ranges');
      return;
    }

    if (!univerRef?.current) {
      setError('Spreadsheet not initialized');
      return;
    }

    setIsProcessing(true);
    try {
      // Call backend to generate formulas
      const result = await invoke<{
        value_formulas: string[];
        uncertainty_formulas: string[];
        success: boolean;
        error?: string;
      }>('generate_uncertainty_formulas', {
        variables: variables.map(v => ({
          name: v.name,
          value_range: v.valueRange,
          uncertainty_range: v.uncertaintyRange
        })),
        formula
      });

      if (!result.success || result.error) {
        setError(result.error || 'Formula generation failed');
        return;
      }

      // Parse output ranges to get starting cell
      const parseRange = (range: string): { col: string; row: number } | null => {
        const match = range.match(/^([A-Z]+)(\d+):/);
        if (!match) return null;
        return { col: match[1], row: parseInt(match[2]) };
      };

      const valueStart = parseRange(outputValueRange);
      const uncStart = parseRange(outputUncertaintyRange);

      if (!valueStart || !uncStart) {
        setError('Invalid output range format');
        return;
      }

      // Write value formulas to Univer
      console.log(`Writing ${result.value_formulas.length} value formulas starting at ${valueStart.col}${valueStart.row}`);
      for (let i = 0; i < result.value_formulas.length; i++) {
        const cellRef = `${valueStart.col}${valueStart.row + i}`;
        const formula = result.value_formulas[i];
        console.log(`  ${cellRef}: ${formula}`);
        univerRef.current.updateCell(cellRef, { f: formula });
      }

      // Write uncertainty formulas to Univer
      console.log(`Writing ${result.uncertainty_formulas.length} uncertainty formulas starting at ${uncStart.col}${uncStart.row}`);
      for (let i = 0; i < result.uncertainty_formulas.length; i++) {
        const cellRef = `${uncStart.col}${uncStart.row + i}`;
        const formula = result.uncertainty_formulas[i];
        console.log(`  ${cellRef}: ${formula}`);
        univerRef.current.updateCell(cellRef, { f: formula });
      }

      onPropagationComplete?.(outputValueRange);
      setError('');
    } catch (err: unknown) {
      console.error('Propagation error:', err);
      setError(String(err));
    } finally {
      setIsProcessing(false);
    }
  };

  if (!open) return null;
  const currentVariable = variables[selectedVariable];

  return (
    <Paper elevation={3} sx={{ 
      width: 420, 
      height: '100%', 
      display: 'flex', 
      flexDirection: 'column', 
      bgcolor: 'rgba(10, 25, 45, 0.98)', 
      border: '1px solid rgba(33, 150, 243, 0.2)', 
      borderLeft: '2px solid rgba(33, 150, 243, 0.5)',
      borderRadius: 0,
      overflow: 'hidden'
    }}
    data-uncertainty-sidebar
    >
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', p: 2.5, bgcolor: 'rgba(33, 150, 243, 0.08)', borderBottom: '1px solid rgba(33, 150, 243, 0.2)' }}>
        <Typography variant="h6" sx={{ fontWeight: 600, color: '#2196f3', fontSize: '1.1rem' }}>Uncertainty Propagation</Typography>
        <IconButton onClick={onClose} size="small" sx={{ color: 'rgba(255, 255, 255, 0.7)', borderRadius: '8px', '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.2)', color: 'rgba(255, 255, 255, 0.9)', transform: 'scale(1.1)' } }}>
          <CloseIcon />
        </IconButton>
      </Box>
      <Box sx={{ p: 2, bgcolor: 'rgba(33, 150, 243, 0.08)', borderBottom: '1px solid rgba(33, 150, 243, 0.15)' }}>
        <Typography variant="body2" sx={{ color: 'rgba(255, 255, 255, 0.8)', fontSize: 13, lineHeight: 1.4 }}>💡 Define variables with ranges. Formula evaluates row-by-row.</Typography>
      </Box>
      <Box sx={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        <Box sx={{ width: 120, borderRight: '1px solid rgba(33, 150, 243, 0.2)', bgcolor: 'rgba(33, 150, 243, 0.03)', height: '100%', overflow: 'auto' }}>
          <Box sx={{ p: 1.5 }}>
            <Button fullWidth size="small" startIcon={<AddIcon sx={{ fontSize: 16 }} />} onClick={addVariable} sx={{ mb: 1.5, fontSize: 12, fontWeight: 600, py: 1, borderRadius: '8px', border: '1px solid rgba(76, 175, 80, 0.3)', bgcolor: 'rgba(76, 175, 80, 0.1)', color: '#4caf50', '&:hover': { bgcolor: 'rgba(76, 175, 80, 0.2)', borderColor: '#4caf50' } }}>Add Variable</Button>
          </Box>
          <List dense sx={{ px: 1 }}>
            {variables.map((variable, index) => (
              <ListItemButton key={index} selected={selectedVariable === index} onClick={() => setSelectedVariable(index)} sx={{ px: 1.5, py: 1, mb: 0.5, borderRadius: '8px', border: '1px solid rgba(33, 150, 243, 0.2)', bgcolor: selectedVariable === index ? 'rgba(33, 150, 243, 0.2)' : 'rgba(33, 150, 243, 0.05)', color: selectedVariable === index ? '#2196f3' : 'rgba(255, 255, 255, 0.7)', transition: 'all 0.2s', '&:hover': { bgcolor: selectedVariable === index ? 'rgba(33, 150, 243, 0.25)' : 'rgba(33, 150, 243, 0.15)', borderColor: '#2196f3', color: '#ffffff', transform: 'translateY(-1px)', boxShadow: '0 2px 8px rgba(33, 150, 243, 0.3)' } }}>
                <ListItemText primary={<Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 0.5 }}><Typography component="span" sx={{ fontSize: 20, fontFamily: 'monospace', fontWeight: 600 }}>{variable.name}</Typography><Typography variant="body2" sx={{ fontSize: 10, fontWeight: 500, textAlign: 'center', lineHeight: 1.2 }}>variable</Typography></Box>} />
              </ListItemButton>
            ))}
          </List>
        </Box>
        <Box sx={{ flex: 1, p: 2, overflow: 'hidden', display: 'flex', flexDirection: 'column', height: '100%' }}>
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, flex: 1 }}>
              <Typography variant="subtitle2" sx={{ mb: 0, fontWeight: 600, color: '#2196f3', minWidth: 'fit-content', fontSize: '0.9rem' }}>
                Variable:
              </Typography>
              <TextField
                value={currentVariable.name}
                onChange={(e) => updateVariable(selectedVariable, 'name', e.target.value)}
                size="small"
                placeholder="a"
                sx={{
                  maxWidth: 100,
                  '& .MuiOutlinedInput-root': {
                    bgcolor: 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '8px',
                    '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                    '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                    '&.Mui-focused fieldset': { borderColor: '#2196f3' }
                  },
                  '& .MuiOutlinedInput-input': {
                    color: '#2196f3',
                    fontFamily: 'monospace',
                    fontSize: 16,
                    fontWeight: 600,
                    padding: '6px 12px',
                    textAlign: 'center'
                  }
                }}
              />
            </Box>
            {variables.length > 1 && (
              <IconButton onClick={() => removeVariable(selectedVariable)} size="small" sx={{ color: '#f44336', borderRadius: '6px', '&:hover': { bgcolor: 'rgba(244, 67, 54, 0.1)', transform: 'scale(1.1)' } }}>
                <DeleteIcon fontSize="small" />
              </IconButton>
            )}
          </Box>
          <Box sx={{ flex: 1, overflow: 'auto', mb: 1.5, height: '100%' }}>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
              <TextField 
                label="Value Range" 
                value={currentVariable.valueRange} 
                onChange={(e) => updateVariable(selectedVariable, 'valueRange', e.target.value)} 
                onFocus={() => handleInputFocus({ type: 'valueRange', varIndex: selectedVariable })}
                onBlur={handleInputBlur}
                placeholder="A1:A10" 
                size="small" 
                fullWidth 
                slotProps={{ 
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }} 
                sx={{ '& .MuiOutlinedInput-root': { bgcolor: 'rgba(33, 150, 243, 0.05)', borderRadius: '6px', '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' }, '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' }, '&.Mui-focused fieldset': { borderColor: '#2196f3' } } }} 
              />
              <TextField 
                label="Uncertainty Range (optional)" 
                value={currentVariable.uncertaintyRange} 
                onChange={(e) => updateVariable(selectedVariable, 'uncertaintyRange', e.target.value)} 
                onFocus={() => handleInputFocus({ type: 'uncertaintyRange', varIndex: selectedVariable })}
                onBlur={handleInputBlur}
                placeholder="B1:B10 or leave empty for zero" 
                size="small" 
                fullWidth 
                slotProps={{ 
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }} 
                sx={{ '& .MuiOutlinedInput-root': { bgcolor: 'rgba(33, 150, 243, 0.05)', borderRadius: '6px', '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' }, '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' }, '&.Mui-focused fieldset': { borderColor: '#2196f3' } } }} 
              />
            </Box>
            <Divider sx={{ my: 1.5, borderColor: 'rgba(33, 150, 243, 0.2)' }} />
            <Typography variant="subtitle2" sx={{ mb: 1, fontWeight: 600, color: 'rgba(255, 255, 255, 0.9)', fontSize: 12 }}>Formula</Typography>
            <TextField value={formula} onChange={(e) => setFormula(e.target.value)} placeholder={`Variables: ${variables.map(v => v.name).join(', ')}`} multiline rows={2} fullWidth slotProps={{ input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 13 } } }} sx={{ '& .MuiOutlinedInput-root': { bgcolor: 'rgba(33, 150, 243, 0.05)', borderRadius: '6px', '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' }, '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' }, '&.Mui-focused fieldset': { borderColor: '#2196f3' } } }} />
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.6)', fontSize: 10, mt: 0.5, display: 'block' }}>Examples: x+y, x*y^2, sqrt(x^2+y^2)</Typography>
            <Divider sx={{ my: 1.5, borderColor: 'rgba(33, 150, 243, 0.2)' }} />
            <Typography variant="subtitle2" sx={{ mb: 1, fontWeight: 600, color: 'rgba(255, 255, 255, 0.9)', fontSize: 12 }}>Output</Typography>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
              <TextField 
                label="Result Values" 
                value={outputValueRange} 
                onChange={(e) => setOutputValueRange(e.target.value)} 
                onFocus={() => handleInputFocus({ type: 'outputValueRange' })}
                onBlur={handleInputBlur}
                placeholder="C1:C10" 
                size="small" 
                fullWidth 
                slotProps={{ 
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }} 
                sx={{ '& .MuiOutlinedInput-root': { bgcolor: 'rgba(33, 150, 243, 0.05)', borderRadius: '6px', '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' }, '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' }, '&.Mui-focused fieldset': { borderColor: '#2196f3' } } }} 
              />
              <TextField 
                label="Result Uncertainties" 
                value={outputUncertaintyRange} 
                onChange={(e) => setOutputUncertaintyRange(e.target.value)} 
                onFocus={() => handleInputFocus({ type: 'outputUncertaintyRange' })}
                onBlur={handleInputBlur}
                placeholder="D1:D10" 
                size="small" 
                fullWidth 
                slotProps={{ 
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }} 
                sx={{ '& .MuiOutlinedInput-root': { bgcolor: 'rgba(33, 150, 243, 0.05)', borderRadius: '6px', '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' }, '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' }, '&.Mui-focused fieldset': { borderColor: '#2196f3' } } }} 
              />
              <Button fullWidth variant="contained" startIcon={<RunIcon />} onClick={handlePropagate} disabled={isProcessing} sx={{ mt: 1, bgcolor: '#2196f3', fontWeight: 600, fontSize: 12, py: 1, outline: 'none', '&:hover': { bgcolor: '#2196f3' }, '&:disabled': { bgcolor: '#424242' }, '&:focus': { bgcolor: '#2196f3', outline: 'none' }, '&:focus-visible': { bgcolor: '#2196f3', outline: 'none', boxShadow: 'none' }, '&:active': { bgcolor: '#2196f3' } }}>
                {isProcessing ? 'Processing...' : 'Propagate'}
              </Button>
            </Box>
            {error && (
              <Box sx={{ mt: 1.5, p: 1, bgcolor: 'rgba(244, 67, 54, 0.1)', borderRadius: '6px', border: '1px solid rgba(244, 67, 54, 0.3)' }}>
                <Typography variant="caption" sx={{ color: '#f44336', fontSize: 11 }}>{error}</Typography>
              </Box>
            )}
          </Box>
        </Box>
      </Box>
    </Paper>
  );
};

export default UncertaintySidebar;
