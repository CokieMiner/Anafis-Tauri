import React, { useState } from 'react';
import { Box, TextField, Typography, IconButton } from '@mui/material';
import { Functions as FunctionsIcon } from '@mui/icons-material';

interface FormulaBarProps {
  activeCell: { row: number; col: number } | null;
  onFormulaSubmit?: (formula: string) => void;
  onCancel?: () => void;
  onEditingChange?: (editing: boolean, value?: string) => void;
  onValueChange?: (value: string) => void;
}

const FormulaBar: React.FC<FormulaBarProps> = ({
  activeCell,
  onFormulaSubmit,
  onCancel,
  onValueChange,
}) => {
  const [formulaValue, setFormulaValue] = useState('');

  // Generate cell reference from activeCell (visual only)
  const cellReference = activeCell
    ? `${String.fromCharCode(65 + activeCell.col)}${activeCell.row + 1}`
    : 'A1';

  const handleFormulaChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = event.target.value;
    setFormulaValue(newValue);
    onValueChange?.(newValue);
  };

  const handleSubmit = () => {
    onFormulaSubmit?.(formulaValue);
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      handleSubmit();
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