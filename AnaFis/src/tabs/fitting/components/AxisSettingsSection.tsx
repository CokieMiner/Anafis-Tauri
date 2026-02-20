import { Box, MenuItem, Select, TextField, Typography } from '@mui/material';
import type {
  AxisConfig,
  AxisScale,
  AxisSettings,
} from '../types/fittingTypes';

type AxisSettingsMode = 'empty' | '2d' | '3d' | 'predicted';

interface AxisSettingsSectionProps {
  axisSettings: AxisSettings;
  onUpdateAxisConfig: (
    axis: keyof AxisSettings,
    update: Partial<AxisConfig>
  ) => void;
  mode: AxisSettingsMode;
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
    '&.Mui-focused fieldset': { borderColor: '#ffb300' },
  },
  '& .MuiInputLabel-root.Mui-focused': { color: '#ffb300' },
};

function AxisRow({
  axis,
  config,
  onUpdate,
}: {
  axis: keyof AxisSettings;
  config: AxisConfig;
  onUpdate: (update: Partial<AxisConfig>) => void;
}) {
  return (
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns:
          'minmax(28px, 0.3fr) minmax(0, 1.7fr) minmax(80px, 0.8fr)',
        gap: 1,
        alignItems: 'center',
        py: 0.5,
      }}
    >
      <Typography
        variant="caption"
        sx={{
          fontFamily: 'monospace',
          fontWeight: 600,
          color: 'text.secondary',
        }}
      >
        {axis.toUpperCase()}
      </Typography>

      <TextField
        fullWidth
        size="small"
        placeholder="Label"
        value={config.label}
        onChange={(event) => {
          onUpdate({ label: event.target.value });
        }}
        sx={amberInputSx}
      />

      <Select
        fullWidth
        size="small"
        value={config.scale}
        onChange={(event) => {
          onUpdate({ scale: event.target.value as AxisScale });
        }}
        sx={{
          '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
            borderColor: '#ffb300',
          },
        }}
      >
        <MenuItem value="linear">Linear</MenuItem>
        <MenuItem value="log">Log</MenuItem>
      </Select>
    </Box>
  );
}

export default function AxisSettingsSection({
  axisSettings,
  onUpdateAxisConfig,
  mode,
}: AxisSettingsSectionProps) {
  const visibleAxes: Array<keyof AxisSettings> =
    mode === '3d' ? ['x', 'y', 'z'] : mode === 'empty' ? [] : ['x', 'y'];

  const helperText =
    mode === '3d'
      ? '3D mode: configure X, Y, and Z.'
      : mode === '2d'
        ? '2D mode: configure X and Y.'
        : mode === 'predicted'
          ? 'N-D mode: configuring Predicted (X) and Observed (Y).'
          : 'Define variables to enable axis settings.';

  return (
    <Box sx={sectionSx}>
      <Typography
        variant="subtitle2"
        sx={{ fontWeight: 700, mb: 0.5, color: 'primary.main' }}
      >
        4. Axis Settings
      </Typography>
      {visibleAxes.length > 0 && (
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ display: 'block', mb: 1 }}
        >
          {helperText}
        </Typography>
      )}

      {visibleAxes.length > 0 ? (
        <Box sx={{ display: 'flex', flexDirection: 'column' }}>
          {visibleAxes.map((axis) => (
            <AxisRow
              key={axis}
              axis={axis}
              config={axisSettings[axis]}
              onUpdate={(update) => {
                onUpdateAxisConfig(axis, update);
              }}
            />
          ))}
        </Box>
      ) : (
        <Box
          sx={{
            minHeight: 48,
            borderRadius: 1.25,
            border: '1px dashed rgba(148,163,184,0.2)',
            background: 'rgba(255,255,255,0.01)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <Typography variant="caption" color="text.secondary">
            {helperText}
          </Typography>
        </Box>
      )}
    </Box>
  );
}
