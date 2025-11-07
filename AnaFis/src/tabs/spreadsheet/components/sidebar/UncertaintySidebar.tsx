import React, { useState, useMemo } from 'react';
import { Box, Typography, IconButton, List, ListItemButton, ListItemText, TextField, Button } from '@mui/material';
import { Close as CloseIcon, Add as AddIcon, Delete as DeleteIcon, PlayArrow as RunIcon } from '@mui/icons-material';
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { useSpreadsheetSelection } from '@/tabs/spreadsheet/managers/useSpreadsheetSelection';
import { sidebarStyles } from '@/tabs/spreadsheet/components/sidebar/utils/sidebarStyles';
import SidebarCard from '@/tabs/spreadsheet/components/sidebar/SidebarCard';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';
import { spreadsheetEventBus } from '@/tabs/spreadsheet/managers/SpreadsheetEventBus';
import { useUncertaintyPropagation } from '@/tabs/spreadsheet/components/sidebar/logic/useUncertaintyPropagation';

type FocusedInputType =
  | { type: 'valueRange'; varIndex: number }
  | { type: 'uncertaintyRange'; varIndex: number }
  | { type: 'outputValueRange' }
  | { type: 'outputUncertaintyRange' }
  | null;

interface UncertaintySidebarProps {
  open: boolean;
  onClose: () => void;
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  onPropagationComplete?: (resultRange: string) => void;
}

export const UncertaintySidebar = React.memo<UncertaintySidebarProps>(({
  open,
  onClose,
  spreadsheetRef,
  onSelectionChange,
  onPropagationComplete,
}) => {
  // Use the uncertainty propagation hook - all business logic is now here
  const {
    variables,
    formula,
    outputValueRange,
    outputUncertaintyRange,
    outputConfidence,
    isProcessing,
    error,
    variableNames,
    setFormula,
    setOutputValueRange,
    setOutputUncertaintyRange,
    setOutputConfidence,
    addVariable,
    removeVariable,
    updateVariable,
    propagate,
  } = useUncertaintyPropagation({
    spreadsheetRef,
    ...(onPropagationComplete && { onComplete: onPropagationComplete }),
  });

  const [selectedVariable, setSelectedVariable] = useState<number>(0);

  // Memoized current variable for performance
  const currentVariable = useMemo(() =>
    variables[selectedVariable],
    [variables, selectedVariable]
  );

  // Use the spreadsheet selection hook
  const { handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange: onSelectionChange ?? (() => { }),
    updateField: (inputType, selection) => {
      if (!inputType) { return; }

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

  // Subscribe to spreadsheet selection events via event bus
  React.useEffect(() => {
    if (!open) { return; }

    const unsubscribe = spreadsheetEventBus.on('selection-change', (cellRef) => {
      // Call the window handler that the hook is listening to
      const handler = window.__uncertaintySidebarSelectionHandler;
      if (handler) {
        handler(cellRef);
      }
      // NOTE: Don't call onSelectionChange here - it would create an infinite loop
      // since onSelectionChange emits to the event bus, which triggers this handler again
    });

    return unsubscribe;
  }, [open]);

  if (!open) { return null; }

  // Return early if no current variable
  if (!currentVariable) { return null; }

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
                  border: selectedVariable === index ? `1px solid ${anafisColors.spreadsheet}` : '1px solid rgba(255, 255, 255, 0.2)',
                  bgcolor: selectedVariable === index ? 'rgba(33, 150, 243, 0.15)' : 'transparent',
                  color: selectedVariable === index ? '#ffffff' : 'rgba(255, 255, 255, 0.7)',
                  transition: 'all 0.2s',
                  '&:hover': {
                    bgcolor: selectedVariable === index ? 'rgba(33, 150, 243, 0.2)' : 'rgba(255, 255, 255, 0.05)',
                    borderColor: selectedVariable === index ? anafisColors.spreadsheet : 'rgba(255, 255, 255, 0.4)',
                    color: '#ffffff',
                    transform: 'translateY(-1px)',
                    boxShadow: selectedVariable === index ? `0 2px 8px rgba(33, 150, 243, 0.3)` : '0 2px 8px rgba(255, 255, 255, 0.1)'
                  },
                  '&.Mui-selected': {
                    bgcolor: 'rgba(33, 150, 243, 0.15) !important',
                    borderColor: `${anafisColors.spreadsheet} !important`,
                    color: '#ffffff !important',
                    '&:hover': {
                      bgcolor: 'rgba(33, 150, 243, 0.2) !important'
                    }
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
                placeholder="A1 or A1:A10"
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
                placeholder="B1 or B1:B10 or leave empty for zero"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
              <TextField
                label="Confidence (%)"
                type="number"
                value={currentVariable.confidence}
                onChange={(e) => updateVariable(selectedVariable, 'confidence', Number(e.target.value))}
                placeholder="95"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: {
                    style: { color: 'white', fontFamily: 'monospace', fontSize: 12 },
                    inputProps: { min: 50, max: 99.9, step: 0.1 }
                  },
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
              placeholder={`Variables: ${variableNames.join(', ')}`}
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
                placeholder="C1 or C1:C10"
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
                placeholder="D1 or D1:D10"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
              <TextField
                label="Output Confidence (%)"
                type="number"
                value={outputConfidence}
                onChange={(e) => setOutputConfidence(Number(e.target.value))}
                placeholder="95"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: {
                    style: { color: 'white', fontFamily: 'monospace', fontSize: 12 },
                    inputProps: { min: 50, max: 99.9, step: 0.1 }
                  },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
              <Button
                fullWidth
                variant="contained"
                startIcon={<RunIcon />}
                onClick={() => void propagate()}
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
});

UncertaintySidebar.displayName = 'UncertaintySidebar';

export default UncertaintySidebar;
