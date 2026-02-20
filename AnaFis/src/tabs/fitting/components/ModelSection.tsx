// § 2. Model Section — Formula, Variables, Parameters (all explicit)

import { Box, TextField, Typography } from '@mui/material';
import { useEffect, useState } from 'react';

interface ModelSectionProps {
  formula: string;
  variableNames: string[];
  parameterNames: string[];
  onFormulaChange: (formula: string) => void;
  onVariableNamesChange: (names: string[]) => void;
  onParameterNamesChange: (names: string[]) => void;
}

const sectionSx = {
  mb: 2,
  p: 1.5,
  borderRadius: 1.5,
  border: '1px solid rgba(148, 163, 184, 0.12)',
  background: 'rgba(255,255,255,0.02)',
};

/** Amber-focused outline for MUI inputs so they match the fitting theme */
const amberInputSx = {
  '& .MuiOutlinedInput-root': {
    '&.Mui-focused fieldset': {
      borderColor: '#ffb300',
    },
  },
  '& .MuiInputLabel-root.Mui-focused': {
    color: '#ffb300',
  },
};

/** Parse a comma-separated string into trimmed, non-empty tokens */
function parseList(text: string): string[] {
  return text
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

export default function ModelSection({
  formula,
  variableNames,
  parameterNames,
  onFormulaChange,
  onVariableNamesChange,
  onParameterNamesChange,
}: ModelSectionProps) {
  // Local text state so commas don't get eaten while typing
  const [varsText, setVarsText] = useState(variableNames.join(', '));
  const [paramsText, setParamsText] = useState(parameterNames.join(', '));

  // Sync from parent → local when parent changes (e.g. reset)
  useEffect(() => {
    setVarsText(variableNames.join(', '));
  }, [variableNames]);

  useEffect(() => {
    setParamsText(parameterNames.join(', '));
  }, [parameterNames]);

  const handleVarsBlur = () => {
    onVariableNamesChange(parseList(varsText));
  };

  const handleParamsBlur = () => {
    onParameterNamesChange(parseList(paramsText));
  };

  return (
    <Box sx={sectionSx}>
      <Typography
        variant="subtitle2"
        sx={{ fontWeight: 700, mb: 1, color: 'primary.main' }}
      >
        2. Model
      </Typography>

      {/* Formula */}
      <TextField
        size="small"
        fullWidth
        label="Formula"
        placeholder="e.g. a*sin(b*x) + c"
        value={formula}
        onChange={(e) => onFormulaChange(e.target.value)}
        sx={{ mb: 1.5, ...amberInputSx }}
        slotProps={{
          input: { sx: { fontFamily: 'monospace', fontSize: '0.85rem' } },
        }}
      />

      {/* Variables */}
      <TextField
        size="small"
        fullWidth
        label="Variables"
        placeholder="x"
        value={varsText}
        onChange={(e) => setVarsText(e.target.value)}
        onBlur={handleVarsBlur}
        helperText="Comma-separated independent variable names"
        sx={{ mb: 1.5, ...amberInputSx }}
        slotProps={{
          input: { sx: { fontFamily: 'monospace', fontSize: '0.85rem' } },
        }}
      />

      {/* Parameters */}
      <TextField
        size="small"
        fullWidth
        label="Parameters"
        placeholder="a, b, c"
        value={paramsText}
        onChange={(e) => setParamsText(e.target.value)}
        onBlur={handleParamsBlur}
        helperText="Comma-separated fit parameter names"
        sx={{ mb: 1, ...amberInputSx }}
        slotProps={{
          input: { sx: { fontFamily: 'monospace', fontSize: '0.85rem' } },
        }}
      />
    </Box>
  );
}
