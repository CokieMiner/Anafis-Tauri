import { createRoot } from 'react-dom/client';
import { useState, useRef } from 'react';

// Material-UI Imports
import { ThemeProvider } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
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
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';

// Tauri imports
import { invoke } from '@tauri-apps/api/core';

// KaTeX imports
import 'katex/dist/katex.min.css';
import { BlockMath } from 'react-katex';

import { createNoTransitionTheme } from './themes';
import CustomTitleBar from './components/CustomTitleBar';

// Create theme using shared configuration without transitions to prevent flickering during resize
const theme = createNoTransitionTheme();

interface Variable {
  name: string;
  value: string;
  uncertainty: string;
}

function UncertaintyCalculatorWindow() {
  // Generate default variables starting with 'a'
  const defaultVariablesInput = 'a'; // Start with just 'a'
  const defaultVariables = [{ name: 'a', value: '', uncertainty: '' }]; // Start with just one variable

  const [formula, setFormula] = useState('');
  const [variablesInput, setVariablesInput] = useState(defaultVariablesInput);
  const [variables, setVariables] = useState<Variable[]>(defaultVariables);
  const [selectedVariableIndex, setSelectedVariableIndex] = useState(0);
  const [mode, setMode] = useState<'calculate' | 'propagate'>('calculate');
  const [calculationResult, setCalculationResult] = useState('Value: N/A\nUncertainty: N/A');
  const [stringRepresentation, setStringRepresentation] = useState('');
  const [latexFormula, setLatexFormula] = useState('');

  // Ref to store current variables for preserving values
  const variablesRef = useRef<Variable[]>(defaultVariables);

  const handleOpenLatexPreview = async () => {
    try {
      await invoke('open_latex_preview_window', {
        latexFormula,
        title: 'LaTeX Formula Preview'
      });
    } catch (error) {
      console.error('Error opening LaTeX preview window:', error);
    }
  };

  // Handle variables input change
  const handleVariablesInputChange = (value: string) => {
    setVariablesInput(value);
    
    const newVariableNames = value.split(',').map(v => v.trim()).filter(Boolean);
    const updatedVariables: Variable[] = [];
    const existingValues: Record<string, { value: string; uncertainty: string }> = {};

    // Preserve existing values for variables that still exist
    variablesRef.current.forEach(v => {
      existingValues[v.name] = { value: v.value, uncertainty: v.uncertainty };
    });

    // Create new variables array
    newVariableNames.forEach(name => {
      updatedVariables.push({
        name,
        value: existingValues[name]?.value || '',
        uncertainty: existingValues[name]?.uncertainty || ''
      });
    });

    setVariables(updatedVariables);
    variablesRef.current = updatedVariables;

    // Adjust selected index if it's out of bounds
    if (selectedVariableIndex >= updatedVariables.length) {
      setSelectedVariableIndex(Math.max(0, updatedVariables.length - 1));
    }
  };

  const updateVariable = (index: number, field: keyof Variable, value: string) => {
    const updated = [...variables];
    updated[index][field] = value;
    setVariables(updated);
  };

  const handleCalculate = async () => {
    if (!formula) {
      alert('Please enter a formula.');
      return;
    }
    if (variables.some(v => !v.value)) {
      alert('Please enter values for all variables.');
      return;
    }

    try {
      // Prepare variables for the backend
      const backendVariables = variables.map(v => ({
        name: v.name,
        value: parseFloat(v.value),
        uncertainty: parseFloat(v.uncertainty || '0'),
      }));

      // Call the calculation function
      const result = await invoke('calculate_uncertainty', {
        formula,
        variables: backendVariables,
      }) as { value: number; uncertainty: number; formula: string };

      // Format and display the result
      const displayValue: string = result.value.toPrecision(6);
      setCalculationResult(`Value: ${displayValue}\nUncertainty: ${result.uncertainty.toPrecision(6)}`);
    } catch (error) {
      console.error('Calculation error:', error);
      setCalculationResult(`Error: ${error}`);
    }
  };

  const handleGenerateLatex = async () => {
    if (!formula) {
      alert('Please enter a formula.');
      return;
    }

    try {
      // Parse variable names
      const variableNames = variables.map(v => v.name);

      // Call the backend to generate LaTeX
      const result = await invoke('generate_latex', {
        formula,
        variables: variableNames,
      }) as { string: string; latex: string };

      // Set the results
      setStringRepresentation(result.string);
      setLatexFormula(result.latex);
    } catch (error) {
      console.error('LaTeX generation error:', error);
      setStringRepresentation(`Error: ${error}`);
      setLatexFormula(`Error: ${error}`);
    }
  };

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        sx={{
          width: '100%',
          height: '100vh',
          boxSizing: 'border-box',
          backgroundColor: 'background.default',
          display: 'flex',
          flexDirection: 'column',
          border: 'none',
          outline: 'none',
          overflow: 'hidden'
        }}
      >
        {/* Custom Title Bar */}
        <CustomTitleBar title="Uncertainty Calculator" />

        {/* Main Content */}
        <Box sx={{ flex: 1, display: 'flex', p: 0 }}>
          <Box
            sx={{
              display: 'flex',
              flexDirection: 'column',
              height: '100vh',
              width: '100vw',
              maxHeight: '100vh',
              maxWidth: '100vw',
              backgroundColor: 'background.default'
            }}
          >
            {/* Custom Title Bar */}
            {/* Main Content */}
            <Box
              sx={{
                flex: 1,
                display: 'flex',
                flexDirection: 'column',
                height: '100%'
              }}
            >
              {/* Formula Input Section - Full Width */}
              <Box sx={{ p: 2, pb: 1 }}>
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
                  sx={{
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
                    '& .MuiOutlinedInput-input': { 
                      color: 'text.primary', 
                      fontSize: '0.9rem', 
                      lineHeight: 1.4,
                      fontFamily: 'monospace'
                    },
                  }}
                />
                <Typography sx={{ 
                  color: 'text.secondary', 
                  fontSize: '0.8rem', 
                  mt: 0.5,
                  fontStyle: 'italic'
                }}>
                  Examples: x+y, x*y^2, sqrt(x^2+y^2), sin(x), cos(y), exp(z)
                </Typography>
              </Box>

              {/* Variables Input - Comma separated */}
              <Box sx={{ px: 2, py: 1, borderBottom: 1, borderColor: 'divider' }}>
                <TextField
                  label="Variables (comma-separated)"
                  placeholder="Enter variable names separated by commas (e.g., x, y, z)"
                  value={variablesInput}
                  onChange={(e) => handleVariablesInputChange(e.target.value)}
                  variant="outlined"
                  size="small"
                  fullWidth
                  sx={{
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
                    '& .MuiOutlinedInput-input': {
                      color: 'text.primary',
                      fontSize: '0.9rem',
                      fontFamily: 'monospace'
                    },
                  }}
                />
              </Box>

              {/* Mode Selection - Full Width, Compact */}
              <Box sx={{ 
                px: 2, 
                py: 1, 
                borderBottom: 1, 
                borderColor: 'divider',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: 2
              }}>
                <FormControl component="fieldset" sx={{ flex: 1 }}>
                  <FormLabel
                    component="legend"
                    sx={{
                      color: 'text.primary',
                      fontWeight: 'bold',
                      mb: 0.5,
                      fontSize: '0.9rem',
                      '&, &:hover, &.Mui-focused': { color: 'text.primary' },
                      cursor: 'default',
                      transition: 'none',
                    }}
                  >
                    Mode
                  </FormLabel>
                  <RadioGroup 
                    row 
                    value={mode} 
                    onChange={(e) => setMode(e.target.value as 'calculate' | 'propagate')} 
                    sx={{ gap: 3, justifyContent: 'flex-start' }}
                  >
                    <FormControlLabel
                      value="calculate"
                      control={<Radio size="small" sx={{ color: 'primary.main', '&.Mui-checked': { color: 'primary.main' } }} />}
                      label="Calculate Value"
                      sx={{
                        color: 'text.primary',
                        '& .MuiFormControlLabel-label': { color: 'text.primary', fontSize: '0.8rem' },
                        '&.Mui-checked, &.Mui-selected': {
                          color: 'text.primary',
                          '& .MuiFormControlLabel-label': { color: 'text.primary' },
                        },
                      }}
                    />
                    <FormControlLabel
                      value="propagate"
                      control={<Radio size="small" sx={{ color: 'primary.main', '&.Mui-checked': { color: 'primary.main' } }} />}
                      label="Generate LaTeX"
                      sx={{
                        color: 'text.primary',
                        '& .MuiFormControlLabel-label': { color: 'text.primary', fontSize: '0.8rem' },
                        '&.Mui-checked, &.Mui-selected': {
                          color: 'text.primary',
                          '& .MuiFormControlLabel-label': { color: 'text.primary' },
                        },
                      }}
                    />
                  </RadioGroup>
                </FormControl>

                <Button 
                  variant="contained" 
                  onClick={mode === 'calculate' ? handleCalculate : handleGenerateLatex}
                  size="small" 
                  sx={{ 
                    backgroundColor: 'primary.main', 
                    '&:hover': { backgroundColor: 'primary.light' }, 
                    fontSize: '0.85rem',
                    px: 2,
                    minWidth: '140px'
                  }}
                >
                  {mode === 'calculate' ? 'Calculate Result' : 'Generate LaTeX'}
                </Button>
              </Box>

              {/* Two-Column Main Content */}
              <Box sx={{ 
                flex: 1, 
                display: 'flex', 
                borderTop: 1,
                borderColor: 'divider',
                minHeight: 0, // Allow flex shrinking
                height: '100%' // Take full available height
              }}>
                {/* Left Column - Variables Panel */}
                <Box sx={{ 
                  width: 200, 
                  flexShrink: 0,
                  borderRight: 1,
                  borderColor: 'divider',
                  display: 'flex',
                  flexDirection: 'column',
                  height: '100%'
                }}>
                  {/* Variables Header */}
                  <Box sx={{ 
                    p: 1.5, 
                    borderBottom: 1, 
                    borderColor: 'divider',
                    flexShrink: 0
                  }}>
                    <Typography variant="subtitle2" sx={{ color: 'text.primary', fontWeight: 'bold' }}>
                      Variables ({variables.length})
                    </Typography>
                  </Box>

                  {/* Variables List */}
                  <Box sx={{ 
                    flex: 1, 
                    p: 1, 
                    overflow: 'auto', 
                    minHeight: 0,
                    '&::-webkit-scrollbar': {
                      width: '8px',
                    },
                    '&::-webkit-scrollbar-track': {
                      backgroundColor: 'background.default',
                    },
                    '&::-webkit-scrollbar-thumb': {
                      backgroundColor: 'primary.main',
                      borderRadius: '4px',
                    },
                    '&::-webkit-scrollbar-thumb:hover': {
                      backgroundColor: 'primary.light',
                    }
                  }}>
                    <List dense sx={{ p: 0 }}>
                      {variables.map((variable, index) => (
                        <ListItemButton
                          key={index}
                          selected={selectedVariableIndex === index}
                          onClick={() => setSelectedVariableIndex(index)}
                          sx={{
                            mb: 0.5,
                            borderRadius: '6px',
                            border: '1px solid',
                            borderColor: selectedVariableIndex === index ? 'primary.main' : 'rgba(255,255,255,0.1)',
                            bgcolor: selectedVariableIndex === index ? 'rgba(156, 39, 176, 0.1)' : 'transparent',
                            '&:hover': {
                              bgcolor: selectedVariableIndex === index ? 'rgba(156, 39, 176, 0.15)' : 'rgba(255,255,255,0.05)',
                              borderColor: 'primary.light'
                            }
                          }}
                        >
                          <Box sx={{ 
                            display: 'flex', 
                            flexDirection: 'column', 
                            alignItems: 'center',
                            width: '100%',
                            py: 0.5
                          }}>
                            <Typography 
                              sx={{ 
                                fontSize: '1.2rem', 
                                fontFamily: 'monospace', 
                                fontWeight: 'bold',
                                color: selectedVariableIndex === index ? 'primary.main' : 'text.primary'
                              }}
                            >
                              {variable.name}
                            </Typography>
                            <Typography 
                              variant="caption" 
                              sx={{ 
                                fontSize: '0.7rem',
                                color: 'text.secondary',
                                mt: -0.5
                              }}
                            >
                              variable
                            </Typography>
                          </Box>
                        </ListItemButton>
                      ))}
                    </List>
                  </Box>
                </Box>

                {/* Right Column - Main Panel */}
                <Box sx={{ 
                  flex: 1, 
                  display: 'flex',
                  flexDirection: 'column',
                  minWidth: 0,
                  height: '100%', // Take full height
                  minHeight: 0 // Allow flex shrinking
                }}>
                  {/* Variable Configuration - Only show in calculate mode */}
                  {mode === 'calculate' && (
                    <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider', width: '100%' }}>
                      <Typography variant="subtitle2" sx={{ mb: 1.5, color: 'text.primary', fontWeight: 'bold' }}>
                        Variable Configuration
                      </Typography>
                      
                      {variables.length > 0 && (
                        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5, width: '100%' }}>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <Typography sx={{ color: 'text.primary', fontSize: '0.9rem', fontWeight: 'bold' }}>
                              Variable: {variables[selectedVariableIndex].name}
                            </Typography>
                          </Box>

                          <TextField
                            label="Value"
                            type="number"
                            placeholder={`Value of ${variables[selectedVariableIndex].name}`}
                            value={variables[selectedVariableIndex].value}
                            onChange={(e) => updateVariable(selectedVariableIndex, 'value', e.target.value)}
                            variant="outlined"
                            size="small"
                            fullWidth
                            sx={{
                              '& .MuiOutlinedInput-root': {
                                backgroundColor: 'background.paper',
                                '& fieldset': { borderColor: 'divider' },
                                '&:hover fieldset': { borderColor: 'primary.light' },
                                '&.Mui-focused fieldset': { borderColor: 'primary.light' }
                              },
                              '& .MuiOutlinedInput-input': { color: 'text.primary', fontSize: '0.9rem' },
                              '& .MuiInputLabel-root': { color: 'text.secondary', fontSize: '0.85rem' }
                            }}
                          />

                          <TextField
                            label="Uncertainty"
                            type="number"
                            placeholder={`Uncertainty of ${variables[selectedVariableIndex].name} (optional)`}
                            value={variables[selectedVariableIndex].uncertainty}
                            onChange={(e) => updateVariable(selectedVariableIndex, 'uncertainty', e.target.value)}
                            variant="outlined"
                            size="small"
                            fullWidth
                            sx={{
                              '& .MuiOutlinedInput-root': {
                                backgroundColor: 'background.paper',
                                '& fieldset': { borderColor: 'divider' },
                                '&:hover fieldset': { borderColor: 'primary.light' },
                                '&.Mui-focused fieldset': { borderColor: 'primary.light' }
                              },
                              '& .MuiOutlinedInput-input': { color: 'text.primary', fontSize: '0.9rem' },
                              '& .MuiInputLabel-root': { color: 'text.secondary', fontSize: '0.85rem' }
                            }}
                          />
                        </Box>
                      )}
                    </Box>
                  )}

                  {/* Results Section */}
                  <Box sx={{ 
                    flex: 1, 
                    p: 2,
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 2,
                    minHeight: 0, // Allow flex shrinking
                    width: '100%',
                    overflow: 'auto' // Allow scrolling if content overflows
                  }}>
                    {mode === 'calculate' && (
                      <Paper elevation={0} sx={{ 
                        p: 2, 
                        backgroundColor: 'background.paper', 
                        border: '1px solid', 
                        borderColor: 'divider', 
                        borderRadius: 1,
                        mb: 2,
                        width: '100%'
                      }}>
                        <Typography variant="h6" sx={{ mb: 1.5, color: 'text.primary', fontWeight: 'bold', fontSize: '0.95rem' }}>
                          Calculation Result
                        </Typography>
                        <Typography 
                          component="pre" 
                          sx={{ 
                            color: 'text.primary', 
                            fontFamily: 'monospace', 
                            whiteSpace: 'pre-wrap', 
                            fontSize: '0.85rem',
                            margin: 0
                          }}
                        >
                          {calculationResult}
                        </Typography>
                      </Paper>
                    )}

                    {mode === 'propagate' && (
                      <>
                        <Paper elevation={0} sx={{
                          p: 1.5,
                          backgroundColor: 'background.paper',
                          border: '1px solid',
                          borderColor: 'divider',
                          borderRadius: 1,
                          mb: 1,
                          width: '100%'
                        }}>
                          <Typography variant="subtitle2" sx={{ mb: 1, color: 'text.primary', fontWeight: 'bold', fontSize: '0.9rem' }}>
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
                                      onClick={() => navigator.clipboard.writeText(stringRepresentation)}
                                      sx={{ color: 'text.secondary' }}
                                    >
                                      <ContentCopyIcon fontSize="small" />
                                    </IconButton>
                                  </InputAdornment>
                                )
                              }
                            }}
                            placeholder="String representation will appear here"
                            variant="outlined"
                            size="small"
                            multiline
                            minRows={1}
                            maxRows={2}
                            fullWidth
                            sx={{
                              '& .MuiOutlinedInput-root': {
                                backgroundColor: 'background.default',
                                '& fieldset': { borderColor: 'divider' }
                              },
                              '& .MuiOutlinedInput-input': {
                                color: 'text.primary',
                                fontSize: '0.75rem',
                                whiteSpace: 'pre-wrap',
                                overflowWrap: 'anywhere',
                                wordBreak: 'break-word'
                              }
                            }}
                          />
                        </Paper>

                        <Paper elevation={0} sx={{
                          p: 1.5,
                          backgroundColor: 'background.paper',
                          border: '1px solid',
                          borderColor: 'divider',
                          borderRadius: 1,
                          mb: 1,
                          width: '100%'
                        }}>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                            <Typography variant="subtitle2" sx={{ color: 'text.primary', fontWeight: 'bold', fontSize: '0.9rem' }}>
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
                                      onClick={() => navigator.clipboard.writeText(latexFormula)}
                                      sx={{ color: 'text.secondary' }}
                                    >
                                      <ContentCopyIcon fontSize="small" />
                                    </IconButton>
                                  </InputAdornment>
                                )
                              }
                            }}
                            placeholder="LaTeX formula will appear here"
                            variant="outlined"
                            size="small"
                            multiline
                            minRows={1}
                            maxRows={2}
                            fullWidth
                            sx={{
                              '& .MuiOutlinedInput-root': {
                                backgroundColor: 'background.default',
                                '& fieldset': { borderColor: 'divider' }
                              },
                              '& .MuiOutlinedInput-input': {
                                color: 'text.primary',
                                fontSize: '0.75rem',
                                whiteSpace: 'pre-wrap',
                                overflowWrap: 'anywhere',
                                wordBreak: 'break-word'
                              }
                            }}
                          />
                        </Paper>

                        <Paper elevation={0} sx={{
                          p: 1.5,
                          backgroundColor: 'background.paper',
                          border: '1px solid',
                          borderColor: 'divider',
                          borderRadius: 1,
                          width: '100%'
                        }}>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                            <Typography variant="subtitle2" sx={{ color: 'text.primary', fontWeight: 'bold', fontSize: '0.9rem' }}>
                              Rendered Formula
                            </Typography>
                            <Button
                              variant="outlined"
                              size="small"
                              onClick={handleOpenLatexPreview}
                              sx={{
                                minWidth: 'auto',
                                px: 1,
                                py: 0.25,
                                fontSize: '0.7rem',
                                backgroundColor: 'background.default',
                                borderColor: 'primary.main',
                                color: 'primary.main',
                                '&:hover': {
                                  backgroundColor: 'primary.main',
                                  color: 'primary.contrastText'
                                }
                              }}
                            >
                              Full View
                            </Button>
                          </Box>
                          <Box sx={{
                            p: 1,
                            backgroundColor: 'background.default',
                            borderRadius: 1,
                            minHeight: '40px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center'
                          }}>
                            <Box sx={{
                              maxWidth: '100%',
                              '& .katex': {
                                fontSize: '0.8em',
                                maxWidth: '100%'
                              }
                            }}>
                              {latexFormula && latexFormula !== `Error: ${latexFormula}` ? (
                                <BlockMath math={latexFormula} />
                              ) : (
                                <Typography sx={{ color: 'text.secondary', fontSize: '0.8rem', fontStyle: 'italic' }}>
                                  Formula will render here
                                </Typography>
                              )}
                            </Box>
                          </Box>
                        </Paper>
                      </>
                    )}
                  </Box>
                </Box>
              </Box>
            </Box>
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
