import {
  Box,
  FormControlLabel,
  Switch,
  TextField,
  Typography,
} from '@mui/material';
import { useEffect, useState } from 'react';
import type { AdvancedSettings, ParameterConfig } from '../types/fittingTypes';

const DEFAULT_PARAMETER_INITIAL_VALUE = 1;
const DEFAULT_MAX_ITERATIONS = 200;
const DEFAULT_TOLERANCE = 1e-9;
const DEFAULT_INITIAL_DAMPING = 1e-3;

function parseMaybeNumber(raw: string): number | null {
  const trimmed = raw.trim();
  if (trimmed.length === 0) {
    return null;
  }
  const normalized = trimmed.replace(',', '.');
  const value = Number(normalized);
  return Number.isFinite(value) ? value : null;
}

interface FitSettingsSectionProps {
  parameterConfigs: ParameterConfig[];
  advancedSettings: AdvancedSettings;
  onUpdateParameterConfig: (
    index: number,
    update: Partial<ParameterConfig>
  ) => void;
  onUpdateAdvancedSettings: (settings: AdvancedSettings) => void;
}

const sectionSx = {
  mb: 2,
  p: 1.5,
  borderRadius: 1.5,
  border: '1px solid rgba(148, 163, 184, 0.12)',
  background: 'rgba(255,255,255,0.02)',
};

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

export default function FitSettingsSection({
  parameterConfigs,
  advancedSettings,
  onUpdateParameterConfig,
  onUpdateAdvancedSettings,
}: FitSettingsSectionProps) {
  const [parameterInputs, setParameterInputs] = useState<
    Record<string, string>
  >({});
  const [maxIterationsInput, setMaxIterationsInput] = useState(
    String(advancedSettings.maxIterations)
  );
  const [toleranceInput, setToleranceInput] = useState(
    String(advancedSettings.tolerance)
  );
  const [initialDampingInput, setInitialDampingInput] = useState(
    String(advancedSettings.initialDamping)
  );

  useEffect(() => {
    setParameterInputs((prev) => {
      const next: Record<string, string> = {};
      for (const parameter of parameterConfigs) {
        const previousDraft = prev[parameter.name];
        next[parameter.name] = previousDraft ?? String(parameter.initialValue);
      }
      return next;
    });
  }, [parameterConfigs]);

  useEffect(() => {
    setMaxIterationsInput(String(advancedSettings.maxIterations));
  }, [advancedSettings.maxIterations]);

  useEffect(() => {
    setToleranceInput(String(advancedSettings.tolerance));
  }, [advancedSettings.tolerance]);

  useEffect(() => {
    setInitialDampingInput(String(advancedSettings.initialDamping));
  }, [advancedSettings.initialDamping]);

  const commitParameterInitialValue = (idx: number, raw: string) => {
    const fallback = DEFAULT_PARAMETER_INITIAL_VALUE;
    const parsed = parseMaybeNumber(raw);
    const nextValue = parsed === null ? fallback : parsed;
    onUpdateParameterConfig(idx, { initialValue: nextValue });
    const name = parameterConfigs[idx]?.name;
    if (name) {
      setParameterInputs((prev) => ({ ...prev, [name]: String(nextValue) }));
    }
  };

  const commitMaxIterations = (raw: string) => {
    const fallback = DEFAULT_MAX_ITERATIONS;
    const parsed = parseMaybeNumber(raw);
    const nextValue =
      parsed === null ? fallback : Math.max(1, Math.round(parsed));
    onUpdateAdvancedSettings({
      ...advancedSettings,
      maxIterations: nextValue,
    });
    setMaxIterationsInput(String(nextValue));
  };

  const commitTolerance = (raw: string) => {
    const fallback = DEFAULT_TOLERANCE;
    const parsed = parseMaybeNumber(raw);
    const nextValue = parsed === null || parsed <= 0 ? fallback : parsed;
    onUpdateAdvancedSettings({
      ...advancedSettings,
      tolerance: nextValue,
    });
    setToleranceInput(String(nextValue));
  };

  const commitInitialDamping = (raw: string) => {
    const fallback = DEFAULT_INITIAL_DAMPING;
    const parsed = parseMaybeNumber(raw);
    const nextValue = parsed === null || parsed <= 0 ? fallback : parsed;
    onUpdateAdvancedSettings({
      ...advancedSettings,
      initialDamping: nextValue,
    });
    setInitialDampingInput(String(nextValue));
  };

  return (
    <Box sx={sectionSx}>
      <Typography
        variant="subtitle2"
        sx={{ fontWeight: 700, mb: 1.5, color: 'primary.main' }}
      >
        3. Parameters & Settings
      </Typography>

      {parameterConfigs.length > 0 && (
        <>
          <Typography
            variant="caption"
            sx={{ color: 'text.secondary', mb: 1, display: 'block' }}
          >
            Initial Guesses
          </Typography>

          <Box
            sx={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fill, minmax(120px, 1fr))',
              gap: 1.5,
              mb: 2,
            }}
          >
            {parameterConfigs.map((param, idx) => (
              <Box key={param.name}>
                <TextField
                  size="small"
                  fullWidth
                  label={param.name}
                  type="text"
                  value={parameterInputs[param.name] ?? ''}
                  onChange={(e) => {
                    const next = e.target.value;
                    setParameterInputs((prev) => ({
                      ...prev,
                      [param.name]: next,
                    }));
                  }}
                  onBlur={(e) =>
                    commitParameterInitialValue(idx, e.target.value)
                  }
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      commitParameterInitialValue(
                        idx,
                        parameterInputs[param.name] ?? ''
                      );
                    }
                  }}
                  sx={amberInputSx}
                  slotProps={{
                    input: {
                      sx: { fontFamily: 'monospace', fontSize: '0.85rem' },
                      inputProps: { inputMode: 'decimal' },
                    },
                  }}
                />
              </Box>
            ))}
          </Box>
        </>
      )}

      <Typography
        variant="caption"
        sx={{ color: 'text.secondary', mb: 1, display: 'block' }}
      >
        Algorithm Settings
      </Typography>

      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
          gap: 1.5,
        }}
      >
        <Box>
          <TextField
            size="small"
            fullWidth
            label="Max Iterations"
            type="text"
            value={maxIterationsInput}
            onChange={(e) => setMaxIterationsInput(e.target.value)}
            onBlur={(e) => commitMaxIterations(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                commitMaxIterations(maxIterationsInput);
              }
            }}
            sx={amberInputSx}
            slotProps={{
              input: {
                sx: { fontFamily: 'monospace', fontSize: '0.85rem' },
                inputProps: { step: '1', inputMode: 'numeric' },
              },
            }}
          />
        </Box>
        <Box>
          <TextField
            size="small"
            fullWidth
            label="Tolerance"
            type="text"
            value={toleranceInput}
            onChange={(e) => setToleranceInput(e.target.value)}
            onBlur={(e) => commitTolerance(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                commitTolerance(toleranceInput);
              }
            }}
            sx={amberInputSx}
            slotProps={{
              input: {
                sx: { fontFamily: 'monospace', fontSize: '0.85rem' },
                inputProps: { step: '1e-6', inputMode: 'decimal' },
              },
            }}
          />
        </Box>
        <Box>
          <TextField
            size="small"
            fullWidth
            label="Initial Damping"
            type="text"
            value={initialDampingInput}
            onChange={(e) => setInitialDampingInput(e.target.value)}
            onBlur={(e) => commitInitialDamping(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                commitInitialDamping(initialDampingInput);
              }
            }}
            sx={amberInputSx}
            slotProps={{
              input: {
                sx: { fontFamily: 'monospace', fontSize: '0.85rem' },
                inputProps: { step: '0.001', inputMode: 'decimal' },
              },
            }}
          />
        </Box>
      </Box>
      <FormControlLabel
        control={
          <Switch
            size="small"
            checked={advancedSettings.usePoissonWeighting ?? false}
            onChange={(e) =>
              onUpdateAdvancedSettings({
                ...advancedSettings,
                usePoissonWeighting: e.target.checked,
              })
            }
          />
        }
        label="Use Poisson weighting"
        sx={{ mt: 1, color: 'text.secondary' }}
      />
    </Box>
  );
}
