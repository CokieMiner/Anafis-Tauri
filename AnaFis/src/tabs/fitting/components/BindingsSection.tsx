// § 3. Bindings Section — Map formula variables to data columns + axis assignment

import {
  Autocomplete,
  type AutocompleteRenderInputParams,
  Box,
  MenuItem,
  Paper,
  type PaperProps,
  Select,
  TextField,
  Typography,
} from '@mui/material';
import type {
  DependentBinding,
  ImportedData,
  VariableBinding,
} from '../types/fittingTypes';

interface BindingsSectionProps {
  importedData: ImportedData | null;
  variableNames: string[];
  variableBindings: VariableBinding[];
  dependentBinding: DependentBinding;
  onUpdateVariableBinding: (
    variableName: string,
    update: Partial<VariableBinding>
  ) => void;
  onUpdateDependentBinding: (update: Partial<DependentBinding>) => void;
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

function SolidPaper(props: PaperProps) {
  return (
    <Paper
      {...props}
      sx={{
        ...((props.sx ?? {}) as object),
        backgroundColor: '#1a1a22 !important',
        backgroundImage: 'none !important',
        border: '1px solid rgba(255,179,0,0.2)',
        boxShadow: '0 8px 24px rgba(0,0,0,0.6)',
      }}
    />
  );
}

const AXIS_OPTIONS: Array<'x' | 'y' | 'z'> = ['x', 'y', 'z'];

function BindingRow({
  label,
  isDependent,
  dataColumn,
  uncColumn,
  axis,
  showAxis,
  columnNames,
  onDataChange,
  onUncChange,
  onAxisChange,
}: {
  label: string;
  isDependent?: boolean;
  dataColumn: string | null;
  uncColumn: string | null;
  axis: 'x' | 'y' | 'z' | undefined;
  showAxis: boolean;
  columnNames: string[];
  onDataChange: (col: string | null) => void;
  onUncChange: (col: string | null) => void;
  onAxisChange?: (axis: 'x' | 'y' | 'z') => void;
}) {
  const gridCols = showAxis
    ? 'minmax(50px, 0.6fr) minmax(0, 1.5fr) minmax(0, 1.5fr) minmax(48px, 0.5fr)'
    : 'minmax(50px, 0.6fr) minmax(0, 1.5fr) minmax(0, 1.5fr)';

  return (
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns: gridCols,
        gap: 1,
        alignItems: 'center',
        py: 0.5,
      }}
    >
      <Typography
        variant="caption"
        sx={{
          fontFamily: 'monospace',
          fontWeight: isDependent ? 700 : 500,
          fontSize: '0.75rem',
          color: isDependent ? '#ffb300' : 'text.secondary',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
        }}
        title={label}
      >
        {label}
      </Typography>

      <Autocomplete
        fullWidth
        size="small"
        options={columnNames}
        value={dataColumn}
        onChange={(_, value) => onDataChange(value)}
        renderInput={(params: AutocompleteRenderInputParams) => (
          <TextField
            fullWidth
            size="small"
            placeholder="Select..."
            variant="outlined"
            inputRef={params.InputProps.ref}
            InputProps={params.InputProps}
            inputProps={params.inputProps}
            sx={amberInputSx}
          />
        )}
        slots={{ paper: SolidPaper }}
        disablePortal
        sx={{ position: 'relative', width: '100%', minWidth: 0 }}
      />

      <Autocomplete
        fullWidth
        size="small"
        options={columnNames}
        value={uncColumn}
        onChange={(_, value) => onUncChange(value)}
        renderInput={(params: AutocompleteRenderInputParams) => (
          <TextField
            fullWidth
            size="small"
            placeholder="Select..."
            variant="outlined"
            inputRef={params.InputProps.ref}
            InputProps={params.InputProps}
            inputProps={params.inputProps}
            sx={amberInputSx}
          />
        )}
        slots={{ paper: SolidPaper }}
        disablePortal
        sx={{ position: 'relative', width: '100%', minWidth: 0 }}
      />

      {showAxis && !isDependent && (
        <Select
          fullWidth
          size="small"
          value={axis ?? 'x'}
          onChange={(event) => {
            onAxisChange?.(event.target.value);
          }}
          sx={{
            width: '100%',
            minWidth: 0,
            fontFamily: 'monospace',
            fontSize: '0.75rem',
            height: 32,
            '& .MuiSelect-select': { py: 0.5 },
            '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
              borderColor: '#ffb300',
            },
          }}
        >
          {AXIS_OPTIONS.map((candidateAxis) => (
            <MenuItem
              key={candidateAxis}
              value={candidateAxis}
              sx={{ fontFamily: 'monospace', fontSize: '0.75rem' }}
            >
              {candidateAxis.toUpperCase()}
            </MenuItem>
          ))}
        </Select>
      )}

      {showAxis && isDependent && (
        <Typography
          variant="caption"
          sx={{
            fontFamily: 'monospace',
            fontSize: '0.7rem',
            color: '#ffb300',
            textAlign: 'center',
          }}
        >
          Z
        </Typography>
      )}
    </Box>
  );
}

export default function BindingsSection({
  importedData,
  variableNames,
  variableBindings,
  dependentBinding,
  onUpdateVariableBinding,
  onUpdateDependentBinding,
}: BindingsSectionProps) {
  const columnNames = importedData?.columns.map((col) => col.name) ?? [];
  const hasVariables = variableNames.length > 0;
  const showAxis =
    variableBindings.length > 1 &&
    variableBindings.length <= AXIS_OPTIONS.length;

  const headerCols = showAxis
    ? 'minmax(50px, 0.6fr) minmax(0, 1.5fr) minmax(0, 1.5fr) minmax(48px, 0.5fr)'
    : 'minmax(50px, 0.6fr) minmax(0, 1.5fr) minmax(0, 1.5fr)';

  return (
    <Box sx={sectionSx}>
      <Typography
        variant="subtitle2"
        sx={{ fontWeight: 700, mb: 0.5, color: 'primary.main' }}
      >
        3. Mappings
      </Typography>

      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: headerCols,
          gap: 1,
          mb: 0.5,
        }}
      >
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ fontSize: '0.65rem' }}
        >
          Variable
        </Typography>
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ fontSize: '0.65rem' }}
        >
          Data column
        </Typography>
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ fontSize: '0.65rem' }}
        >
          Uncertainty
        </Typography>
        {showAxis && (
          <Typography
            variant="caption"
            color="text.secondary"
            sx={{ fontSize: '0.65rem' }}
          >
            Axis
          </Typography>
        )}
      </Box>

      {hasVariables ? (
        <Box
          sx={{
            maxHeight: 220,
            overflow: 'auto',
            '&::-webkit-scrollbar': { width: 4 },
            '&::-webkit-scrollbar-thumb': {
              background: 'rgba(255,179,0,0.25)',
              borderRadius: 2,
            },
          }}
        >
          <BindingRow
            label="Dependent"
            isDependent
            dataColumn={dependentBinding.dataColumn}
            uncColumn={dependentBinding.uncColumn}
            axis={undefined}
            showAxis={showAxis}
            columnNames={columnNames}
            onDataChange={(col) =>
              onUpdateDependentBinding({ dataColumn: col })
            }
            onUncChange={(col) => onUpdateDependentBinding({ uncColumn: col })}
          />

          <Box
            sx={{ borderTop: '1px solid rgba(148,163,184,0.08)', my: 0.5 }}
          />

          {variableBindings.map((binding) => {
            return (
              <BindingRow
                key={binding.variableName}
                label={binding.variableName}
                dataColumn={binding.dataColumn}
                uncColumn={binding.uncColumn}
                axis={binding.axis}
                showAxis={showAxis}
                columnNames={columnNames}
                onDataChange={(col) => {
                  onUpdateVariableBinding(binding.variableName, {
                    dataColumn: col,
                  });
                }}
                onUncChange={(col) => {
                  onUpdateVariableBinding(binding.variableName, {
                    uncColumn: col,
                  });
                }}
                onAxisChange={(axis) => {
                  onUpdateVariableBinding(binding.variableName, { axis });
                }}
              />
            );
          })}
        </Box>
      ) : (
        <Box
          sx={{
            minHeight: 60,
            borderRadius: 1.25,
            border: '1px dashed rgba(148,163,184,0.2)',
            background: 'rgba(255,255,255,0.01)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <Typography variant="caption" color="text.secondary">
            Define variables to enable mappings.
          </Typography>
        </Box>
      )}
    </Box>
  );
}
