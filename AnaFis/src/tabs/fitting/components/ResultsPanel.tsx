// Results Panel — Parameters with uncertainties, goodness-of-fit metrics, covariance dropdown

import {
  Alert,
  Box,
  Button,
  Collapse,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  Typography,
} from '@mui/material';
import { useState } from 'react';

import type { FitStatus, OdrFitResponse } from '../types/fittingTypes';

function formatScientific(value: number): string {
  if (value === 0) {
    return '0';
  }
  return value.toPrecision(4);
}

function formatValueWithUncertainty(
  value: number,
  uncertainty: number
): string {
  if (uncertainty === 0) {
    return value.toPrecision(6);
  }
  const digits = Math.max(
    0,
    -Math.floor(Math.log10(Math.abs(uncertainty))) + 1
  );
  return `${value.toFixed(digits)} ± ${uncertainty.toFixed(digits)}`;
}

interface ResultsPanelProps {
  fitResult: OdrFitResponse | null;
  canRunFit: boolean;
  fitStatus: FitStatus;
  fitError: string | null;
  onRunFit: () => void;
}

export default function ResultsPanel({
  fitResult,
  canRunFit,
  fitStatus,
  fitError,
  onRunFit,
}: ResultsPanelProps) {
  const [covExpanded, setCovExpanded] = useState(false);

  const hasResult = Boolean(fitResult?.success);

  const panelSx = {
    height: '100%',
    p: 1.5,
    borderRadius: 2,
    border: '1px solid rgba(148, 163, 184, 0.18)',
    background:
      'linear-gradient(140deg, rgba(19, 19, 24, 0.95) 0%, rgba(14, 14, 18, 0.9) 100%)',
    overflow: 'auto',
  };

  if (!hasResult) {
    return (
      <Box sx={panelSx}>
        <Button
          variant="contained"
          fullWidth
          disabled={!canRunFit || fitStatus === 'running'}
          onClick={onRunFit}
          sx={{
            mb: 1,
            py: 1,
            fontWeight: 700,
            fontSize: '0.9rem',
            letterSpacing: '0.04em',
            background: canRunFit
              ? 'linear-gradient(135deg, #ffb300 0%, #f57c00 100%)'
              : undefined,
            color: canRunFit ? '#111' : undefined,
            boxShadow: canRunFit ? '0 4px 16px rgba(255, 179, 0, 0.3)' : 'none',
            '&:hover': {
              background: canRunFit
                ? 'linear-gradient(135deg, #ffca28 0%, #ffb300 100%)'
                : undefined,
              boxShadow: canRunFit
                ? '0 6px 20px rgba(255, 179, 0, 0.45)'
                : 'none',
            },
            '&.Mui-disabled': {
              background: 'rgba(255,255,255,0.06)',
              color: 'rgba(255,255,255,0.25)',
            },
          }}
        >
          {fitStatus === 'running' ? 'Fitting…' : 'Run Fit'}
        </Button>

        {fitError && (
          <Alert severity="error" sx={{ mb: 1, py: 0 }}>
            {fitError}
          </Alert>
        )}

        <Box
          sx={{
            minHeight: 120,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <Typography variant="body2" color="text.secondary">
            Run a fit to see results
          </Typography>
        </Box>
      </Box>
    );
  }

  const {
    parameterNames,
    parameterValues,
    parameterUncertainties,
    parameterCovariance,
    chiSquared,
    chiSquaredReduced,
    rmse,
    rSquared,
    iterations,
    message,
  } = fitResult as OdrFitResponse;

  return (
    <Box sx={panelSx}>
      <Button
        variant="contained"
        fullWidth
        disabled={!canRunFit || fitStatus === 'running'}
        onClick={onRunFit}
        sx={{
          mb: 1,
          py: 1,
          fontWeight: 700,
          fontSize: '0.9rem',
          letterSpacing: '0.04em',
          background: canRunFit
            ? 'linear-gradient(135deg, #ffb300 0%, #f57c00 100%)'
            : undefined,
          color: canRunFit ? '#111' : undefined,
          boxShadow: canRunFit ? '0 4px 16px rgba(255, 179, 0, 0.3)' : 'none',
          '&:hover': {
            background: canRunFit
              ? 'linear-gradient(135deg, #ffca28 0%, #ffb300 100%)'
              : undefined,
            boxShadow: canRunFit
              ? '0 6px 20px rgba(255, 179, 0, 0.45)'
              : 'none',
          },
          '&.Mui-disabled': {
            background: 'rgba(255,255,255,0.06)',
            color: 'rgba(255,255,255,0.25)',
          },
        }}
      >
        {fitStatus === 'running' ? 'Fitting…' : 'Run Fit'}
      </Button>

      {fitError && (
        <Alert severity="error" sx={{ mb: 1, py: 0 }}>
          {fitError}
        </Alert>
      )}

      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          mb: 1,
        }}
      >
        <Typography
          variant="subtitle2"
          sx={{ fontWeight: 700, color: 'text.secondary' }}
        >
          Results
        </Typography>
        <Typography
          variant="caption"
          sx={{
            px: 0.8,
            py: 0.2,
            borderRadius: 1,
            border: '1px solid rgba(255,179,0,0.35)',
            color: 'primary.light',
            fontWeight: 600,
          }}
        >
          {iterations} iterations
        </Typography>
      </Box>

      {message && (
        <Alert severity="info" sx={{ mb: 1, py: 0, fontSize: '0.7rem' }}>
          {message}
        </Alert>
      )}

      <Box
        sx={{
          mb: 1,
          p: 1,
          borderRadius: 1.5,
          border: '1px solid rgba(148, 163, 184, 0.16)',
          background: 'rgba(255, 255, 255, 0.02)',
        }}
      >
        <Typography
          variant="caption"
          sx={{ color: 'text.secondary', display: 'block', mb: 0.6 }}
        >
          Parameters
        </Typography>
        <Table size="small" sx={{ '& td': { borderBottom: 'none' } }}>
          <TableBody>
            {parameterNames.map((name, idx) => (
              <TableRow key={name}>
                <TableCell
                  sx={{
                    width: 64,
                    py: 0.15,
                    px: 0.3,
                    fontSize: '0.78rem',
                    fontFamily: 'monospace',
                    color: 'primary.main',
                    fontWeight: 700,
                  }}
                >
                  {name}
                </TableCell>
                <TableCell
                  sx={{
                    py: 0.15,
                    px: 0.3,
                    fontSize: '0.78rem',
                    fontFamily: 'monospace',
                  }}
                >
                  {formatValueWithUncertainty(
                    parameterValues[idx] ?? 0,
                    parameterUncertainties[idx] ?? 0
                  )}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </Box>

      <Box
        sx={{
          mb: 1,
          p: 1,
          borderRadius: 1.5,
          border: '1px solid rgba(148, 163, 184, 0.16)',
          background: 'rgba(255, 255, 255, 0.015)',
        }}
      >
        <Typography
          variant="caption"
          sx={{ color: 'text.secondary', display: 'block', mb: 0.6 }}
        >
          Fit quality
        </Typography>
        <Box
          sx={{
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: 0.6,
            fontSize: '0.75rem',
            fontFamily: 'monospace',
            color: 'text.secondary',
          }}
        >
          <Typography variant="caption" sx={{ fontFamily: 'inherit' }}>
            χ² = {formatScientific(chiSquared)}
          </Typography>
          <Typography variant="caption" sx={{ fontFamily: 'inherit' }}>
            χ²red = {formatScientific(chiSquaredReduced)}
          </Typography>
          <Typography variant="caption" sx={{ fontFamily: 'inherit' }}>
            RMSE = {formatScientific(rmse)}
          </Typography>
          <Typography variant="caption" sx={{ fontFamily: 'inherit' }}>
            R² = {formatScientific(rSquared)}
          </Typography>
        </Box>
      </Box>

      <Box
        sx={{
          p: 1,
          borderRadius: 1.5,
          border: '1px solid rgba(148, 163, 184, 0.16)',
          background: 'rgba(255, 255, 255, 0.015)',
        }}
      >
        <Button
          size="small"
          onClick={() => setCovExpanded((value) => !value)}
          sx={{
            textTransform: 'none',
            fontSize: '0.75rem',
            fontWeight: 600,
            color: 'primary.main',
            px: 0,
            minWidth: 0,
          }}
        >
          {covExpanded
            ? 'Hide covariance matrix ▾'
            : 'Show covariance matrix ▸'}
        </Button>

        <Collapse in={covExpanded} timeout={180} unmountOnExit>
          <Typography
            variant="caption"
            color="text.secondary"
            sx={{ display: 'block', mt: 0.7 }}
          >
            Full covariance matrix from ODR fit (scaled by χ²red).
          </Typography>
          <Box
            sx={{
              mt: 0.8,
              maxHeight: 220,
              overflow: 'auto',
              border: '1px solid rgba(148, 163, 184, 0.22)',
              borderRadius: 1,
              '&::-webkit-scrollbar': { width: 8, height: 8 },
              '&::-webkit-scrollbar-thumb': {
                background: 'rgba(255,179,0,0.30)',
                borderRadius: 8,
              },
            }}
          >
            <Table
              size="small"
              sx={{ minWidth: parameterNames.length * 96 + 96 }}
            >
              <TableHead>
                <TableRow>
                  <TableCell />
                  {parameterNames.map((name) => (
                    <TableCell
                      key={name}
                      align="center"
                      sx={{
                        fontWeight: 700,
                        fontSize: '0.74rem',
                        fontFamily: 'monospace',
                        whiteSpace: 'nowrap',
                        px: 0.8,
                        py: 0.6,
                      }}
                    >
                      {name}
                    </TableCell>
                  ))}
                </TableRow>
              </TableHead>
              <TableBody>
                {parameterNames.map((rowName, rowIdx) => (
                  <TableRow key={rowName}>
                    <TableCell
                      sx={{
                        fontWeight: 700,
                        fontSize: '0.74rem',
                        fontFamily: 'monospace',
                        whiteSpace: 'nowrap',
                        px: 0.8,
                        py: 0.5,
                      }}
                    >
                      {rowName}
                    </TableCell>
                    {parameterNames.map((_, colIdx) => {
                      const covValue = parameterCovariance?.[rowIdx]?.[colIdx];
                      const isDiagonal = rowIdx === colIdx;
                      const isValid =
                        covValue !== undefined && Number.isFinite(covValue);

                      return (
                        <TableCell
                          // biome-ignore lint/suspicious/noArrayIndexKey: Matrix positions are static
                          key={`matrix-cov-${rowName}-${colIdx}`}
                          align="center"
                          sx={{
                            fontSize: '0.72rem',
                            fontFamily: 'monospace',
                            whiteSpace: 'nowrap',
                            px: 0.8,
                            py: 0.5,
                            backgroundColor: isDiagonal
                              ? 'rgba(255,179,0,0.08)'
                              : 'transparent',
                          }}
                        >
                          {isValid ? formatScientific(covValue) : '—'}
                        </TableCell>
                      );
                    })}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </Box>
        </Collapse>
      </Box>
    </Box>
  );
}
