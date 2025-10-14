import React, { useState } from 'react';
import { Box, Typography, IconButton, List, ListItemButton, ListItemText, TextField, Button } from '@mui/material';
import { Close as CloseIcon, Add as AddIcon, Delete as DeleteIcon, PlayArrow as RunIcon } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { SpreadsheetRef } from './SpreadsheetInterface';
import { useSpreadsheetSelection } from '../../hooks/useSpreadsheetSelection';
import { sidebarStyles } from '../../utils/sidebarStyles';
import SidebarCard from '../ui/SidebarCard';
import { anafisColors } from '../../themes';

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
  univerRef: React.RefObject<SpreadsheetRef | null>;
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
    <Box
      data-uncertainty-sidebar
      sx={{ ...sidebarStyles.container, px: 1, pt: 2 }}
    >
      {/* Header */}
      <Box sx={sidebarStyles.header}>
        <Typography sx={sidebarStyles.text.header}>
          Uncertainty Propagation
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
      <Box sx={{ flex: 1, display: 'flex', overflow: 'hidden', gap: 1.5, p: 1.5 }}>
        {/* Variables List */}
        <SidebarCard title="Variables" sx={{ width: 140, flexShrink: 0, mx: 0.5 }}>
          <Button
            fullWidth
            size="small"
            startIcon={<AddIcon sx={{ fontSize: 16 }} />}
            onClick={addVariable}
            sx={sidebarStyles.button.secondary}
          >
            Add Variable
          </Button>

          <List dense sx={{ mt: 1.5 }}>
            {variables.map((variable, index) => (
              <ListItemButton
                key={index}
                selected={selectedVariable === index}
                onClick={() => setSelectedVariable(index)}
                sx={{
                  px: 1,
                  py: 0.75,
                  mb: 0.5,
                  borderRadius: '6px',
                  border: '1px solid rgba(33, 150, 243, 0.2)',
                  bgcolor: selectedVariable === index ? 'rgba(33, 150, 243, 0.2)' : 'rgba(33, 150, 243, 0.05)',
                  color: selectedVariable === index ? anafisColors.spreadsheet : 'rgba(255, 255, 255, 0.7)',
                  transition: 'all 0.2s',
                  '&:hover': {
                    bgcolor: selectedVariable === index ? 'rgba(33, 150, 243, 0.25)' : 'rgba(33, 150, 243, 0.15)',
                    borderColor: anafisColors.spreadsheet,
                    color: '#ffffff',
                    transform: 'translateY(-1px)',
                    boxShadow: '0 2px 8px rgba(33, 150, 243, 0.3)'
                  }
                }}
              >
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 0.25 }}>
                      <Typography component="span" sx={{ fontSize: 18, fontFamily: 'monospace', fontWeight: 600 }}>
                        {variable.name}
                      </Typography>
                      <Typography variant="body2" sx={{ fontSize: 9, fontWeight: 500, textAlign: 'center', lineHeight: 1.2 }}>
                        variable
                      </Typography>
                    </Box>
                  }
                />
              </ListItemButton>
            ))}
          </List>
        </SidebarCard>
        {/* Variable Configuration */}
        <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 1.5 }}>
          {/* Variable Details */}
          <SidebarCard title={`Variable ${currentVariable.name}`} sx={{ mx: 0.5 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1.5 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flex: 1 }}>
                <Typography sx={{ ...sidebarStyles.text.label, minWidth: 'fit-content' }}>
                  Name:
                </Typography>
                <TextField
                  value={currentVariable.name}
                  onChange={(e) => updateVariable(selectedVariable, 'name', e.target.value)}
                  size="small"
                  placeholder="a"
                  sx={{
                    maxWidth: 80,
                    ...sidebarStyles.input
                  }}
                  inputProps={{
                    style: {
                      color: anafisColors.spreadsheet,
                      fontFamily: 'monospace',
                      fontSize: 14,
                      fontWeight: 600,
                      textAlign: 'center',
                      padding: '4px 8px'
                    }
                  }}
                />
              </Box>
              {variables.length > 1 && (
                <IconButton
                  onClick={() => removeVariable(selectedVariable)}
                  size="small"
                  sx={{
                    color: '#f44336',
                    borderRadius: '6px',
                    '&:hover': {
                      bgcolor: 'rgba(244, 67, 54, 0.1)',
                      transform: 'scale(1.1)'
                    }
                  }}
                >
                  <DeleteIcon fontSize="small" />
                </IconButton>
              )}
            </Box>

            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
              <TextField
                label="Value Range"
                value={currentVariable.valueRange}
                onChange={(e) => updateVariable(selectedVariable, 'valueRange', e.target.value)}
                onFocus={() => handleInputFocus({ type: 'valueRange', varIndex: selectedVariable })}
                onBlur={handleInputBlur}
                placeholder="A1:A10"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
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
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
            </Box>
          </SidebarCard>

          {/* Formula */}
          <SidebarCard title="Formula" sx={{ mx: 0.5 }}>
            <TextField
              value={formula}
              onChange={(e) => setFormula(e.target.value)}
              placeholder={`Variables: ${variables.map(v => v.name).join(', ')}`}
              multiline
              rows={2}
              fullWidth
              sx={sidebarStyles.input}
              slotProps={{
                input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 13 } }
              }}
            />
            <Typography sx={{ ...sidebarStyles.text.caption, mt: 0.5 }}>
              Examples: x+y, x*y^2, sqrt(x^2+y^2)
            </Typography>
          </SidebarCard>

          {/* Output */}
          <SidebarCard title="Output" sx={{ mx: 0.5 }}>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
              <TextField
                label="Result Values"
                value={outputValueRange}
                onChange={(e) => setOutputValueRange(e.target.value)}
                onFocus={() => handleInputFocus({ type: 'outputValueRange' })}
                onBlur={handleInputBlur}
                placeholder="C1:C10"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
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
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
              <Button
                fullWidth
                variant="contained"
                startIcon={<RunIcon />}
                onClick={handlePropagate}
                disabled={isProcessing}
                sx={sidebarStyles.button.primary}
              >
                {isProcessing ? 'Processing...' : 'Propagate'}
              </Button>
            </Box>

            {error && (
              <Box sx={{
                mt: 1.5,
                p: 1,
                bgcolor: 'rgba(244, 67, 54, 0.1)',
                borderRadius: '6px',
                border: '1px solid rgba(244, 67, 54, 0.3)'
              }}>
                <Typography sx={{ ...sidebarStyles.text.caption, color: '#f44336' }}>
                  {error}
                </Typography>
              </Box>
            )}
          </SidebarCard>
        </Box>
      </Box>
    </Box>
  );
};

export default UncertaintySidebar;
