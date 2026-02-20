import {
  Box,
  List,
  ListItemButton,
  TextField,
  Typography,
} from '@mui/material';
import type { Theme } from '@mui/material/styles';
import type React from 'react';
import { memo, useCallback, useMemo } from 'react';

// Static styles to prevent recreation
const CONTAINER_STYLES = {
  width: 200,
  flexShrink: 0,
  borderRight: 1,
  borderColor: 'divider',
  display: 'flex',
  flexDirection: 'column',
  height: '100%',
} as const;

const HEADER_STYLES = {
  p: 1.5,
  borderBottom: 1,
  borderColor: 'divider',
  flexShrink: 0,
} as const;

const LIST_CONTAINER_STYLES = {
  flex: 1,
  p: 1,
  overflow: 'auto',
  minHeight: 0,
  // Cross-browser scrollbar styling
  // Firefox
  scrollbarWidth: 'thin',
  scrollbarColor: (theme: Theme) =>
    `${theme.palette.primary.main} ${theme.palette.background.default}`,
  // WebKit browsers (Chrome, Safari, Edge)
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
  },
  // Fallback for browsers that don't support custom scrollbars
  '@supports not (scrollbar-width: thin)': {
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
    },
  },
} as const;

const CONFIG_CONTAINER_STYLES = {
  p: 2,
  borderTop: 1,
  borderColor: 'divider',
  width: '100%',
} as const;

const TEXT_FIELD_STYLES = {
  '& .MuiOutlinedInput-root': {
    backgroundColor: 'background.paper',
    '& fieldset': { borderColor: 'divider' },
    '&:hover fieldset': { borderColor: 'primary.light' },
    '&.Mui-focused fieldset': { borderColor: 'primary.light' },
  },
  '& .MuiOutlinedInput-input': { color: 'text.primary', fontSize: '0.9rem' },
  '& .MuiInputLabel-root': { color: 'text.secondary', fontSize: '0.85rem' },
} as const;

interface Variable {
  id: string;
  name: string;
  value: string;
  uncertainty: string;
}

interface VariableManagerProps {
  variables: Variable[];
  selectedIndex: number;
  onVariableSelect: (index: number) => void;
  onVariableUpdate: (
    index: number,
    field: keyof Variable,
    value: string
  ) => void;
}

const VariableManager: React.FC<VariableManagerProps> = memo(
  ({ variables, selectedIndex, onVariableSelect, onVariableUpdate }) => {
    // Memoized handlers
    const handleValueChange = useCallback(
      (e: React.ChangeEvent<HTMLInputElement>) => {
        onVariableUpdate(selectedIndex, 'value', e.target.value);
      },
      [onVariableUpdate, selectedIndex]
    );

    const handleUncertaintyChange = useCallback(
      (e: React.ChangeEvent<HTMLInputElement>) => {
        onVariableUpdate(selectedIndex, 'uncertainty', e.target.value);
      },
      [onVariableUpdate, selectedIndex]
    );

    // Memoized selected variable
    const selectedVariable = useMemo(
      () => variables[selectedIndex] ?? null,
      [variables, selectedIndex]
    );

    return (
      <Box sx={CONTAINER_STYLES}>
        {/* Variables Header */}
        <Box sx={HEADER_STYLES}>
          <Typography
            variant="subtitle2"
            sx={{ color: 'text.primary', fontWeight: 'bold' }}
          >
            Variables ({variables.length})
          </Typography>
        </Box>

        {/* Variables List */}
        <Box sx={LIST_CONTAINER_STYLES}>
          <List dense sx={{ p: 0 }}>
            {variables.map((variable, index) => (
              <ListItemButton
                key={variable.id}
                selected={selectedIndex === index}
                onClick={() => onVariableSelect(index)}
                sx={{
                  mb: 0.5,
                  borderRadius: '6px',
                  border: '1px solid',
                  borderColor:
                    selectedIndex === index
                      ? 'primary.main'
                      : 'rgba(255,255,255,0.1)',
                  bgcolor:
                    selectedIndex === index
                      ? 'rgba(156, 39, 176, 0.1)'
                      : 'transparent',
                  '&:hover': {
                    bgcolor:
                      selectedIndex === index
                        ? 'rgba(156, 39, 176, 0.15)'
                        : 'rgba(255,255,255,0.05)',
                    borderColor: 'primary.light',
                  },
                }}
              >
                <Box
                  sx={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    width: '100%',
                    py: 0.5,
                  }}
                >
                  <Typography
                    sx={{
                      fontSize: '1.2rem',
                      fontFamily: 'monospace',
                      fontWeight: 'bold',
                      color:
                        selectedIndex === index
                          ? 'primary.main'
                          : 'text.primary',
                    }}
                  >
                    {variable.name}
                  </Typography>
                  <Typography
                    variant="caption"
                    sx={{
                      fontSize: '0.7rem',
                      color: 'text.secondary',
                      mt: -0.5,
                    }}
                  >
                    variable
                  </Typography>
                </Box>
              </ListItemButton>
            ))}
          </List>
        </Box>

        {/* Variable Configuration */}
        {selectedVariable && (
          <Box sx={CONFIG_CONTAINER_STYLES}>
            <Typography
              variant="subtitle2"
              sx={{ mb: 1.5, color: 'text.primary', fontWeight: 'bold' }}
            >
              Configure {selectedVariable.name}
            </Typography>

            <Box
              sx={{
                display: 'flex',
                flexDirection: 'column',
                gap: 1.5,
                width: '100%',
              }}
            >
              <TextField
                label="Value"
                type="number"
                placeholder={`Value of ${selectedVariable.name}`}
                value={selectedVariable.value}
                onChange={handleValueChange}
                variant="outlined"
                size="small"
                fullWidth
                sx={TEXT_FIELD_STYLES}
              />

              <TextField
                label="Uncertainty"
                type="number"
                placeholder={`Uncertainty of ${selectedVariable.name} (optional)`}
                value={selectedVariable.uncertainty}
                onChange={handleUncertaintyChange}
                variant="outlined"
                size="small"
                fullWidth
                sx={TEXT_FIELD_STYLES}
              />
            </Box>
          </Box>
        )}
      </Box>
    );
  }
);

VariableManager.displayName = 'VariableManager';

export default VariableManager;
