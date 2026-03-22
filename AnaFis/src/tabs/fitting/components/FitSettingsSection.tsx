import {
  Box,
  FormControlLabel,
  Switch,
  TextField,
  Typography,
} from '@mui/material';
import type { AdvancedSettings, ParameterConfig } from '../types/fittingTypes';

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
                  type="number"
                  value={param.initialValue}
                  onChange={(e) =>
                    onUpdateParameterConfig(idx, {
                      initialValue: Number(e.target.value),
                    })
                  }
                  sx={amberInputSx}
                  slotProps={{
                    input: {
                      sx: { fontFamily: 'monospace', fontSize: '0.85rem' },
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
            type="number"
            value={advancedSettings.maxIterations}
            onChange={(e) =>
              onUpdateAdvancedSettings({
                ...advancedSettings,
                maxIterations: Math.max(1, Number(e.target.value)),
              })
            }
            sx={amberInputSx}
            slotProps={{
              input: {
                sx: { fontFamily: 'monospace', fontSize: '0.85rem' },
                inputProps: { step: '1' },
              },
            }}
          />
        </Box>
        <Box>
          <TextField
            size="small"
            fullWidth
            label="Tolerance"
            type="number"
            value={advancedSettings.tolerance}
            onChange={(e) =>
              onUpdateAdvancedSettings({
                ...advancedSettings,
                tolerance: Number(e.target.value),
              })
            }
            sx={amberInputSx}
            slotProps={{
              input: {
                sx: { fontFamily: 'monospace', fontSize: '0.85rem' },
                inputProps: { step: '1e-6' },
              },
            }}
          />
        </Box>
        <Box>
          <TextField
            size="small"
            fullWidth
            label="Initial Damping"
            type="number"
            value={advancedSettings.initialDamping}
            onChange={(e) =>
              onUpdateAdvancedSettings({
                ...advancedSettings,
                initialDamping: Number(e.target.value),
              })
            }
            sx={amberInputSx}
            slotProps={{
              input: {
                sx: { fontFamily: 'monospace', fontSize: '0.85rem' },
                inputProps: { step: '0.001' },
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
