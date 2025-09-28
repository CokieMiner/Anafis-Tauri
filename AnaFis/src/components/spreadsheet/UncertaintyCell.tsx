import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Box, TextField, Typography, IconButton, Menu, MenuItem } from '@mui/material';
import { SwapHoriz as SwapIcon } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';

interface UncertaintyComponents {
  value: number;
  uncertainty: number;
  uncertainty_type: 'absolute' | 'percentage' | 'standard_deviation' | 'standard_error';
  display_string: string;
}

interface UncertaintyClickResult {
  focus_area: 'Value' | 'Uncertainty';
  components: UncertaintyComponents;
}

interface UncertaintyCellProps {
  cellRef: string;
  initialValue?: string;
  isEditing: boolean;
  onValueChange: (value: string) => void;
  onEditingChange: (editing: boolean) => void;
  onFocusAreaChange?: (area: 'value' | 'uncertainty') => void;
  width?: number;
}

const UncertaintyCell: React.FC<UncertaintyCellProps> = ({
  cellRef,
  initialValue = '',
  isEditing,
  onValueChange,
  onEditingChange,
  onFocusAreaChange,
  width = 100,
}) => {
  const [components, setComponents] = useState<UncertaintyComponents | null>(null);
  const [focusArea, setFocusArea] = useState<'value' | 'uncertainty'>('value');
  const [valueInput, setValueInput] = useState('');
  const [uncertaintyInput, setUncertaintyInput] = useState('');
  const [conversionMenuAnchor, setConversionMenuAnchor] = useState<null | HTMLElement>(null);
  
  const valueInputRef = useRef<HTMLInputElement>(null);
  const uncertaintyInputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Load uncertainty components when cell reference changes
  useEffect(() => {
    const loadComponents = async () => {
      try {
        const result = await invoke<UncertaintyComponents | null>('get_uncertainty_cell_components', {
          cellRef
        });
        
        if (result) {
          setComponents(result);
          setValueInput(result.value.toString());
          setUncertaintyInput(result.uncertainty.toString());
        } else {
          // Try to detect if input contains uncertainty notation
          const hasUncertainty = await invoke<boolean>('detect_uncertainty_mode', {
            input: initialValue
          });
          
          if (hasUncertainty) {
            // Enable uncertainty mode for this cell
            await invoke('toggle_uncertainty_cell_mode', {
              cellRef,
              enable: true
            });
            
            // Reload components
            const newResult = await invoke<UncertaintyComponents | null>('get_uncertainty_cell_components', {
              cellRef
            });
            
            if (newResult) {
              setComponents(newResult);
              setValueInput(newResult.value.toString());
              setUncertaintyInput(newResult.uncertainty.toString());
            }
          }
        }
      } catch (error) {
        console.error('Failed to load uncertainty components:', error);
      }
    };

    if (cellRef) {
      loadComponents();
    }
  }, [cellRef, initialValue]);

  // Handle click position detection
  const handleCellClick = useCallback(async (event: React.MouseEvent) => {
    if (!isEditing && components) {
      try {
        const rect = containerRef.current?.getBoundingClientRect();
        if (rect) {
          const clickX = event.clientX - rect.left;
          const result = await invoke<UncertaintyClickResult>('handle_uncertainty_cell_click', {
            cellRef,
            clickX,
            cellWidth: width
          });
          
          const newFocusArea = result.focus_area === 'Value' ? 'value' : 'uncertainty';
          setFocusArea(newFocusArea);
          onFocusAreaChange?.(newFocusArea);
          
          // Start editing mode
          onEditingChange(true);
        }
      } catch (error) {
        console.error('Failed to handle uncertainty cell click:', error);
      }
    }
  }, [isEditing, components, cellRef, width, onFocusAreaChange, onEditingChange]);

  // Focus appropriate input when editing starts
  useEffect(() => {
    if (isEditing) {
      setTimeout(() => {
        if (focusArea === 'value' && valueInputRef.current) {
          valueInputRef.current.focus();
          valueInputRef.current.select();
        } else if (focusArea === 'uncertainty' && uncertaintyInputRef.current) {
          uncertaintyInputRef.current.focus();
          uncertaintyInputRef.current.select();
        }
      }, 0);
    }
  }, [isEditing, focusArea]);

  // Handle Tab navigation between inputs
  const handleKeyDown = useCallback((event: React.KeyboardEvent, inputType: 'value' | 'uncertainty') => {
    if (event.key === 'Tab') {
      event.preventDefault();
      const newFocusArea = inputType === 'value' ? 'uncertainty' : 'value';
      setFocusArea(newFocusArea);
      onFocusAreaChange?.(newFocusArea);
      
      setTimeout(() => {
        if (newFocusArea === 'value' && valueInputRef.current) {
          valueInputRef.current.focus();
          valueInputRef.current.select();
        } else if (newFocusArea === 'uncertainty' && uncertaintyInputRef.current) {
          uncertaintyInputRef.current.focus();
          uncertaintyInputRef.current.select();
        }
      }, 0);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      handleSubmit();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      onEditingChange(false);
    }
  }, [onFocusAreaChange, onEditingChange]);

  // Handle value submission
  const handleSubmit = useCallback(async () => {
    if (!components) return;
    
    try {
      const value = parseFloat(valueInput);
      const uncertainty = parseFloat(uncertaintyInput);
      
      if (isNaN(value) || isNaN(uncertainty)) {
        console.error('Invalid numeric input');
        return;
      }
      
      await invoke('set_uncertainty_cell_value', {
        cellRef,
        value,
        uncertainty,
        uncertaintyType: components.uncertainty_type
      });
      
      // Reload components to get updated display string
      const result = await invoke<UncertaintyComponents | null>('get_uncertainty_cell_components', {
        cellRef
      });
      
      if (result) {
        setComponents(result);
        onValueChange(result.display_string);
      }
      
      onEditingChange(false);
    } catch (error) {
      console.error('Failed to submit uncertainty value:', error);
    }
  }, [cellRef, valueInput, uncertaintyInput, components, onValueChange, onEditingChange]);

  // Handle uncertainty type conversion
  const handleConvertUncertaintyType = useCallback(async (targetType: string) => {
    try {
      const result = await invoke<UncertaintyComponents>('convert_uncertainty_type', {
        cellRef,
        targetType
      });
      
      setComponents(result);
      setValueInput(result.value.toString());
      setUncertaintyInput(result.uncertainty.toString());
      onValueChange(result.display_string);
      
      setConversionMenuAnchor(null);
    } catch (error) {
      console.error('Failed to convert uncertainty type:', error);
    }
  }, [cellRef, onValueChange]);

  // Handle input changes
  const handleValueInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setValueInput(event.target.value);
  };

  const handleUncertaintyInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setUncertaintyInput(event.target.value);
  };

  if (!components) {
    // Not an uncertainty cell, render as regular text
    return (
      <Box
        sx={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          px: 1,
          cursor: 'text',
        }}
        onClick={handleCellClick}
      >
        <Typography variant="body2" sx={{ fontSize: '14px' }}>
          {initialValue}
        </Typography>
      </Box>
    );
  }

  if (isEditing) {
    // Dual-input editing mode
    return (
      <Box
        ref={containerRef}
        sx={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          gap: 0.5,
          px: 0.5,
          bgcolor: focusArea === 'value' ? 'rgba(100, 181, 246, 0.1)' : 'transparent',
        }}
      >
        {/* Value Input */}
        <TextField
          ref={valueInputRef}
          value={valueInput}
          onChange={handleValueInputChange}
          onKeyDown={(e) => handleKeyDown(e, 'value')}
          variant="standard"
          size="small"
          sx={{
            flex: 1,
            '& .MuiInput-root': {
              fontSize: '12px',
              '&:before': { borderBottom: focusArea === 'value' ? '2px solid #64b5f6' : '1px solid rgba(0,0,0,0.2)' },
              '&:hover:before': { borderBottom: '2px solid #64b5f6' },
              '&:after': { borderBottom: '2px solid #64b5f6' },
            },
            '& .MuiInput-input': {
              padding: '2px 4px',
              textAlign: 'center',
              backgroundColor: focusArea === 'value' ? 'rgba(100, 181, 246, 0.05)' : 'transparent',
            },
          }}
        />
        
        {/* Separator */}
        <Typography variant="body2" sx={{ fontSize: '12px', color: 'text.secondary', mx: 0.5 }}>
          ±
        </Typography>
        
        {/* Uncertainty Input */}
        <TextField
          ref={uncertaintyInputRef}
          value={uncertaintyInput}
          onChange={handleUncertaintyInputChange}
          onKeyDown={(e) => handleKeyDown(e, 'uncertainty')}
          variant="standard"
          size="small"
          sx={{
            flex: 1,
            '& .MuiInput-root': {
              fontSize: '12px',
              '&:before': { borderBottom: focusArea === 'uncertainty' ? '2px solid #64b5f6' : '1px solid rgba(0,0,0,0.2)' },
              '&:hover:before': { borderBottom: '2px solid #64b5f6' },
              '&:after': { borderBottom: '2px solid #64b5f6' },
            },
            '& .MuiInput-input': {
              padding: '2px 4px',
              textAlign: 'center',
              backgroundColor: focusArea === 'uncertainty' ? 'rgba(100, 181, 246, 0.05)' : 'transparent',
            },
          }}
        />
        
        {/* Conversion Button */}
        <IconButton
          size="small"
          onClick={(e) => setConversionMenuAnchor(e.currentTarget)}
          sx={{ 
            p: 0.25, 
            ml: 0.5,
            '&:hover': { bgcolor: 'rgba(100, 181, 246, 0.1)' }
          }}
        >
          <SwapIcon sx={{ fontSize: '12px' }} />
        </IconButton>
        
        {/* Conversion Menu */}
        <Menu
          anchorEl={conversionMenuAnchor}
          open={Boolean(conversionMenuAnchor)}
          onClose={() => setConversionMenuAnchor(null)}
          PaperProps={{
            sx: { minWidth: 150 }
          }}
        >
          <MenuItem 
            onClick={() => handleConvertUncertaintyType('absolute')}
            selected={components.uncertainty_type === 'absolute'}
          >
            Absolute
          </MenuItem>
          <MenuItem 
            onClick={() => handleConvertUncertaintyType('percentage')}
            selected={components.uncertainty_type === 'percentage'}
          >
            Percentage
          </MenuItem>
          <MenuItem 
            onClick={() => handleConvertUncertaintyType('standard_deviation')}
            selected={components.uncertainty_type === 'standard_deviation'}
          >
            Std. Deviation
          </MenuItem>
          <MenuItem 
            onClick={() => handleConvertUncertaintyType('standard_error')}
            selected={components.uncertainty_type === 'standard_error'}
          >
            Std. Error
          </MenuItem>
        </Menu>
      </Box>
    );
  }

  // Display mode
  return (
    <Box
      ref={containerRef}
      sx={{
        width: '100%',
        height: '100%',
        display: 'flex',
        alignItems: 'center',
        px: 1,
        cursor: 'text',
        '&:hover': {
          bgcolor: 'rgba(100, 181, 246, 0.05)',
        },
      }}
      onClick={handleCellClick}
    >
      <Typography 
        variant="body2" 
        sx={{ 
          fontSize: '14px',
          fontFamily: 'monospace',
          whiteSpace: 'nowrap',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
        }}
      >
        {components.display_string}
      </Typography>
    </Box>
  );
};

export default UncertaintyCell;