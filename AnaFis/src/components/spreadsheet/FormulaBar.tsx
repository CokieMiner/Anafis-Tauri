import React, { useState, useEffect } from 'react';
import { Box, TextField, Typography, IconButton, Chip } from '@mui/material';
import { Functions as FunctionsIcon } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';

interface FormulaBarProps {
  activeCell: { row: number; col: number } | null;
  currentValue?: string;
  cellReference?: string;
  onFormulaSubmit?: (formula: string) => void;
  onMultiCellFill?: (value: string) => void;
  onCancel?: () => void;
  onEditingChange?: (editing: boolean, value?: string) => void;
  onValueChange?: (value: string) => void;
}

const FormulaBar: React.FC<FormulaBarProps> = ({
  activeCell,
  currentValue = '',
  cellReference: propCellReference,
  onFormulaSubmit,
  onMultiCellFill,
  onCancel,
  onValueChange,
}) => {
  const [formulaValue, setFormulaValue] = useState(currentValue);
  const [isUncertaintyMode, setIsUncertaintyMode] = useState(false);

  // Update formula value when current value changes
  useEffect(() => {
    setFormulaValue(currentValue);
    
    // Check if current value contains uncertainty notation
    const checkUncertaintyMode = async () => {
      if (currentValue) {
        try {
          const hasUncertainty = await invoke<boolean>('detect_uncertainty_mode', {
            input: currentValue
          });
          setIsUncertaintyMode(hasUncertainty);
        } catch (error) {
          console.error('Failed to detect uncertainty mode:', error);
          setIsUncertaintyMode(false);
        }
      } else {
        setIsUncertaintyMode(false);
      }
    };
    
    checkUncertaintyMode();
  }, [currentValue]);

  // Use provided cell reference or generate from activeCell
  const cellReference = propCellReference || (activeCell
    ? `${String.fromCharCode(65 + activeCell.col)}${activeCell.row + 1}`
    : 'A1');

  const handleFormulaChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = event.target.value;
    setFormulaValue(newValue);
    onValueChange?.(newValue);
    
    // Check for uncertainty notation in real-time
    try {
      const hasUncertainty = await invoke<boolean>('detect_uncertainty_mode', {
        input: newValue
      });
      setIsUncertaintyMode(hasUncertainty);
      
      // If uncertainty notation is detected and we have an active cell, enable uncertainty mode
      if (hasUncertainty && activeCell) {
        const cellRef = `${String.fromCharCode(65 + activeCell.col)}${activeCell.row + 1}`;
        await invoke('toggle_uncertainty_cell_mode', {
          cellRef,
          enable: true
        });
      }
    } catch (error) {
      console.error('Failed to detect uncertainty mode:', error);
    }
  };

  const handleSubmit = () => {
    onFormulaSubmit?.(formulaValue);
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      
      // Check for multi-cell fill shortcuts
      if (event.ctrlKey && event.shiftKey) {
        // Ctrl+Shift+Enter - Array formula (for now, same as multi-fill)
        onMultiCellFill?.(formulaValue);
      } else if (event.ctrlKey) {
        // Ctrl+Enter - Fill all selected cells
        onMultiCellFill?.(formulaValue);
      } else {
        // Regular Enter - Single cell submit
        handleSubmit();
      }
    } else if (event.key === 'Escape') {
      event.preventDefault();
      onCancel?.();
    }
  };

  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        padding: '8px',
        backgroundColor: '#0a0a0a',
        gap: 1,
      }}
    >
      {/* Cell Reference */}
      <Box
        sx={{
          minWidth: 80,
          padding: '4px 8px',
          border: '1px solid #64b5f6',
          borderRadius: 1,
          backgroundColor: '#0a0a0a',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <Typography variant="body2" sx={{ fontSize: '12px', fontWeight: 'bold', color: 'white' }}>
          {cellReference}
        </Typography>
      </Box>

      {/* Functions Button */}
      <IconButton
        size="small"
        sx={{
          padding: '4px',
          color: 'white',
          '&:hover': {
            backgroundColor: 'rgba(100, 181, 246, 0.1)'
          }
        }}
        title="Insert Function"
      >
        <FunctionsIcon fontSize="small" />
      </IconButton>

      {/* Uncertainty Mode Indicator */}
      {isUncertaintyMode && (
        <Chip
          icon={<span style={{ fontSize: '12px' }}>±</span>}
          label="Uncertainty"
          size="small"
          variant="outlined"
          sx={{
            height: 24,
            fontSize: '10px',
            color: '#64b5f6',
            borderColor: '#64b5f6',
            '& .MuiChip-label': { 
              color: '#64b5f6',
              paddingLeft: '4px',
              paddingRight: '8px'
            },
            '& .MuiChip-icon': {
              color: '#64b5f6',
              marginLeft: '4px'
            }
          }}
        />
      )}

      {/* Formula Input */}
      <TextField
        value={formulaValue}
        onChange={handleFormulaChange}
        onKeyDown={handleKeyDown}

        placeholder="Enter value or formula..."
        variant="outlined"
        size="small"
        fullWidth
        sx={{
          '& .MuiOutlinedInput-root': {
            fontSize: '12px',
            height: 32,
            color: 'white',
            '& input': {
              padding: '8px 12px',
              color: 'white',
            },
            '& fieldset': {
              borderColor: '#64b5f6',
            },
            '&:hover fieldset': {
              borderColor: '#42a5f5',
            },
            '&.Mui-focused fieldset': {
              borderColor: '#64b5f6',
              boxShadow: '0 0 0 2px rgba(100, 181, 246, 0.2)',
            },
          },
          '& .MuiOutlinedInput-input::placeholder': {
            color: 'rgba(255, 255, 255, 0.5)',
          },
        }}
      />
    </Box>
  );
};

export default FormulaBar;