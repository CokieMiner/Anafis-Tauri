// Results Panel — Parameters with uncertainties, goodness-of-fit metrics, covariance dropdown

import { HelpOutlined as HelpOutlineIcon } from '@mui/icons-material';
import CloseIcon from '@mui/icons-material/Close';
import {
  Alert,
  Box,
  Button,
  Collapse,
  Dialog,
  DialogContent,
  DialogTitle,
  IconButton,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  Tooltip,
  Typography,
} from '@mui/material';
import { useState } from 'react';
import { anafisTheme } from '@/shared/theme/unifiedTheme';
import type {
  FitStatus,
  OdrFitResponse,
} from '@/tabs/fitting/types/fittingTypes';

function formatScientific(value: number): string {
  if (value === 0) {
    return '0';
  }
  return value.toPrecision(4);
}

function formatRSquared(value: number): string {
  if (!Number.isFinite(value)) {
    return 'NaN';
  }
  if (value < 1 && value >= 0.9995) {
    return value.toFixed(6);
  }
  return value.toPrecision(6);
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

function formatExpandedUncertainty(uncertainty: number): string {
  if (!Number.isFinite(uncertainty) || uncertainty === 0) {
    return '';
  }
  const digits = Math.max(
    0,
    -Math.floor(Math.log10(Math.abs(uncertainty))) + 1
  );
  return `± ${uncertainty.toFixed(digits)}`;
}

function terminationLabel(reason: string): {
  text: string;
  color: 'success' | 'warning' | 'error' | 'info';
} {
  switch (reason) {
    case 'scaledGradient':
      return { text: 'Converged (gradient)', color: 'success' };
    case 'scaledStep':
      return { text: 'Converged (step size)', color: 'success' };
    case 'improvement':
      return { text: 'Converged (improvement)', color: 'success' };
    case 'stagnated':
      return { text: 'Stagnated', color: 'warning' };
    case 'singular':
      return { text: 'Singular system', color: 'error' };
    case 'dampingSaturated':
      return { text: 'Damping saturated', color: 'warning' };
    case 'maxIterations':
      return { text: 'Max iterations', color: 'warning' };
    default:
      return { text: reason, color: 'info' };
  }
}

const statusColors: Record<string, string> = {
  success: '#66bb6a',
  warning: '#ffa726',
  error: '#ef5350',
  info: '#42a5f5',
};

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
  const [assumptionsOpen, setAssumptionsOpen] = useState(false);
  const [uncertaintyBasis, setUncertaintyBasis] = useState<'scaled' | 'raw'>(
    'scaled'
  );
  const [matrixKind, setMatrixKind] = useState<'covariance' | 'correlation'>(
    'covariance'
  );

  const hasResult =
    Boolean(fitResult) &&
    (fitResult?.parameterValues.length ?? 0) > 0 &&
    (fitResult?.fittedValues.length ?? 0) > 0;

  const panelSx = {
    height: '100%',
    p: 2,
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
            mb: 1.2,
            py: 1.15,
            fontWeight: 700,
            fontSize: '1rem',
            letterSpacing: '0.04em',
            background: canRunFit
              ? `linear-gradient(135deg, ${anafisTheme.colors.tabs.fitting.main} 0%, ${anafisTheme.colors.tabs.fitting.dark} 100%)`
              : undefined,
            color: canRunFit ? '#111' : undefined,
            boxShadow: canRunFit
              ? `0 4px 16px ${anafisTheme.colors.tabs.fitting.main}4D`
              : 'none',
            '&:hover': {
              background: canRunFit
                ? `linear-gradient(135deg, ${anafisTheme.colors.tabs.fitting.light} 0%, ${anafisTheme.colors.tabs.fitting.main} 100%)`
                : undefined,
              boxShadow: canRunFit
                ? `0 6px 20px ${anafisTheme.colors.tabs.fitting.main}73`
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
            minHeight: 140,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <Typography variant="body1" color="text.secondary">
            Run a fit to see results
          </Typography>
        </Box>
      </Box>
    );
  }

  const resolvedFitResult = fitResult as OdrFitResponse;

  const {
    parameterNames,
    parameterValues,
    parameterUncertainties,
    parameterUncertaintiesRaw,
    parameterExpandedUncertainties,
    parameterCovariance,
    parameterCovarianceRaw,
    parameterCorrelations,
    parameterCorrelationsRaw,
    coverageFactor,
    coverageDegreesOfFreedom,
    welchSatterthwaiteDof,
    chiSquared,
    chiSquaredObservation,
    chiSquaredObservationReduced,
    chiSquaredReduced,
    rmse,
    residualStandardError,
    rSquared,
    effectiveRank,
    conditionNumber,
    iterations,
    terminationReason,
    message,
    assumptions,
  } = resolvedFitResult;

  const termination = terminationLabel(terminationReason);
  const termColor = statusColors[termination.color] ?? statusColors.info;

  const hasExpandedUncertainties = parameterExpandedUncertainties.some(
    (u) => Number.isFinite(u) && u > 0
  );
  const displayedUncertainties =
    uncertaintyBasis === 'raw'
      ? (parameterUncertaintiesRaw ?? parameterUncertainties)
      : parameterUncertainties;
  const displayedCovariance =
    uncertaintyBasis === 'raw'
      ? (parameterCovarianceRaw ?? parameterCovariance)
      : parameterCovariance;
  const displayedCorrelation =
    uncertaintyBasis === 'raw'
      ? (parameterCorrelationsRaw ?? parameterCorrelations)
      : parameterCorrelations;
  const displayedMatrix =
    matrixKind === 'correlation' ? displayedCorrelation : displayedCovariance;
  const confidencePercent = Number.isFinite(coverageFactor)
    ? Math.round(
        // Inverse of what the backend did — approximate % from k for display
        // For dof > ~30, k ≈ 1.96 → 95%. Just show k directly.
        95
      )
    : null;

  return (
    <Box sx={panelSx}>
      <Button
        variant="contained"
        fullWidth
        disabled={!canRunFit || fitStatus === 'running'}
        onClick={onRunFit}
        sx={{
          mb: 1.2,
          py: 1.15,
          fontWeight: 700,
          fontSize: '1rem',
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

      {/* ── Header: Results + iteration count + termination ── */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          mb: 0.5,
        }}
      >
        <Typography
          variant="subtitle1"
          sx={{
            fontWeight: 700,
            color: 'text.secondary',
            letterSpacing: '0.01em',
          }}
        >
          Results
        </Typography>
        <Typography
          variant="body2"
          sx={{
            px: 1,
            py: 0.25,
            borderRadius: 1,
            border: `1px solid ${anafisTheme.colors.tabs.fitting.main}59`,
            color: 'primary.light',
            fontWeight: 600,
            fontSize: '0.82rem',
          }}
        >
          {iterations} iterations
        </Typography>
      </Box>

      {/* Termination reason */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          gap: 0.6,
          mb: 1.2,
        }}
      >
        <Box
          sx={{
            width: 7,
            height: 7,
            borderRadius: '50%',
            backgroundColor: termColor,
            flexShrink: 0,
          }}
        />
        <Typography
          variant="body2"
          sx={{
            color: termColor,
            fontWeight: 600,
            fontSize: '0.86rem',
          }}
        >
          {termination.text}
        </Typography>
      </Box>

      {message && (
        <Alert
          severity={resolvedFitResult.success ? 'info' : 'warning'}
          sx={{ mb: 1.2, py: 0.2, fontSize: '0.82rem' }}
        >
          {message}
        </Alert>
      )}

      {/* ── Parameters ── */}
      <Box
        sx={{
          mb: 1.2,
          p: 1.25,
          borderRadius: 1.5,
          border: '1px solid rgba(148, 163, 184, 0.16)',
          background: 'rgba(255, 255, 255, 0.015)',
        }}
      >
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            mb: 0.6,
            gap: 0.8,
          }}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.6 }}>
            <Typography
              variant="body2"
              sx={{ color: 'text.secondary', display: 'block' }}
            >
              Parameters
            </Typography>
            <Button
              size="small"
              onClick={() => setUncertaintyBasis('scaled')}
              sx={{
                minWidth: 0,
                px: 0.8,
                py: 0,
                lineHeight: 1.3,
                fontSize: '0.78rem',
                textTransform: 'none',
                color:
                  uncertaintyBasis === 'scaled'
                    ? 'primary.main'
                    : 'text.disabled',
              }}
            >
              scaled
            </Button>
            <Button
              size="small"
              onClick={() => setUncertaintyBasis('raw')}
              sx={{
                minWidth: 0,
                px: 0.8,
                py: 0,
                lineHeight: 1.3,
                fontSize: '0.78rem',
                textTransform: 'none',
                color:
                  uncertaintyBasis === 'raw' ? 'primary.main' : 'text.disabled',
              }}
            >
              raw
            </Button>
          </Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.6 }}>
            <Typography
              variant="body2"
              sx={{ color: 'text.disabled', fontSize: '0.78rem' }}
            >
              σ source:{' '}
              {uncertaintyBasis === 'raw' ? (
                <>
                  N<sup>-1</sup>
                </>
              ) : (
                <>
                  χ<sup>2</sup>
                  <sub>red</sub> · N<sup>-1</sup>
                </>
              )}
            </Typography>
            {hasExpandedUncertainties && confidencePercent && (
              <Typography
                variant="body2"
                sx={{
                  color: 'text.disabled',
                  fontSize: '0.78rem',
                  fontStyle: 'italic',
                }}
              >
                {confidencePercent}% CI, k = {coverageFactor.toPrecision(3)}
              </Typography>
            )}
          </Box>
        </Box>
        <Table size="small" sx={{ '& td': { borderBottom: 'none' } }}>
          <TableBody>
            {parameterNames.map((name, idx) => (
              <TableRow key={name}>
                <TableCell
                  sx={{
                    width: 54,
                    py: 0.25,
                    px: 0.45,
                    fontSize: '0.92rem',
                    fontFamily: 'monospace',
                    color: 'primary.main',
                    fontWeight: 700,
                  }}
                >
                  {name}
                </TableCell>
                <TableCell
                  sx={{
                    py: 0.25,
                    px: 0.45,
                    fontSize: '0.92rem',
                    fontFamily: 'monospace',
                  }}
                >
                  {formatValueWithUncertainty(
                    parameterValues[idx] ?? 0,
                    displayedUncertainties[idx] ?? 0
                  )}
                </TableCell>
                {hasExpandedUncertainties && (
                  <TableCell
                    sx={{
                      py: 0.25,
                      px: 0.45,
                      fontSize: '0.82rem',
                      fontFamily: 'monospace',
                      color: 'text.disabled',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    <Tooltip
                      title="Expanded uncertainty (U)"
                      placement="top"
                      arrow
                    >
                      <span>
                        (
                        {formatExpandedUncertainty(
                          parameterExpandedUncertainties[idx] ?? 0
                        )}
                        )
                      </span>
                    </Tooltip>
                  </TableCell>
                )}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </Box>

      {/* ── Fit quality ── */}
      <Box
        sx={{
          mb: 1.2,
          p: 1.25,
          borderRadius: 1.5,
          border: '1px solid rgba(148, 163, 184, 0.16)',
          background: 'rgba(255, 255, 255, 0.015)',
        }}
      >
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            mb: 0.6,
          }}
        >
          <Typography variant="body2" sx={{ color: 'text.secondary' }}>
            Fit quality
          </Typography>
          {assumptions && assumptions.length > 0 && (
            <Tooltip title="Assumptions & methodology" placement="top" arrow>
              <IconButton
                size="small"
                onClick={() => setAssumptionsOpen(true)}
                sx={{
                  p: 0.2,
                  color: 'text.disabled',
                  '&:hover': { color: 'primary.main' },
                }}
              >
                <HelpOutlineIcon sx={{ fontSize: '1rem' }} />
              </IconButton>
            </Tooltip>
          )}
        </Box>
        <Box
          sx={{
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: 0.75,
            fontSize: '0.9rem',
            fontFamily: 'monospace',
            color: 'text.secondary',
          }}
        >
          <Typography variant="body2" sx={{ fontFamily: 'inherit' }}>
            χ² = {formatScientific(chiSquared)}
          </Typography>
          <Typography variant="body2" sx={{ fontFamily: 'inherit' }}>
            χ²<sub>red</sub> = {formatScientific(chiSquaredReduced)}
          </Typography>
          <Typography variant="body2" sx={{ fontFamily: 'inherit' }}>
            χ²<sub>obs</sub> = {formatScientific(chiSquaredObservation)}
          </Typography>
          <Typography variant="body2" sx={{ fontFamily: 'inherit' }}>
            χ²<sub>obs,red</sub> ={' '}
            {formatScientific(chiSquaredObservationReduced)}
          </Typography>
          <Typography variant="body2" sx={{ fontFamily: 'inherit' }}>
            RMSE = {formatScientific(rmse)}
          </Typography>
          <Typography variant="body2" sx={{ fontFamily: 'inherit' }}>
            RSE = {formatScientific(residualStandardError)}
          </Typography>
          <Typography variant="body2" sx={{ fontFamily: 'inherit' }}>
            R² = {formatRSquared(rSquared)}
          </Typography>
          <Typography
            variant="body2"
            sx={{ fontFamily: 'inherit', color: 'text.disabled' }}
          >
            rank = {effectiveRank} / {parameterNames.length}
          </Typography>
          <Typography
            variant="body2"
            sx={{ fontFamily: 'inherit', color: 'text.disabled' }}
          >
            κ ={' '}
            {conditionNumber < 1e15 ? formatScientific(conditionNumber) : '∞'}
          </Typography>
          {typeof coverageDegreesOfFreedom === 'number' &&
            Number.isFinite(coverageDegreesOfFreedom) && (
              <Typography
                variant="body2"
                sx={{ fontFamily: 'inherit', color: 'text.disabled' }}
              >
                ν<sub>coverage</sub> ={' '}
                {formatScientific(coverageDegreesOfFreedom)}
              </Typography>
            )}
          {typeof welchSatterthwaiteDof === 'number' &&
            Number.isFinite(welchSatterthwaiteDof) && (
              <Typography
                variant="body2"
                sx={{ fontFamily: 'inherit', color: 'text.disabled' }}
              >
                ν<sub>WS</sub> = {formatScientific(welchSatterthwaiteDof)}
              </Typography>
            )}
        </Box>
      </Box>

      {/* ── Covariance matrix ── */}
      <Box
        sx={{
          p: 1.25,
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
            fontSize: '0.88rem',
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
          <Box
            sx={{ display: 'flex', alignItems: 'center', gap: 0.6, mt: 0.6 }}
          >
            <Button
              size="small"
              onClick={() => setMatrixKind('covariance')}
              sx={{
                minWidth: 0,
                px: 0.8,
                py: 0,
                lineHeight: 1.3,
                fontSize: '0.78rem',
                textTransform: 'none',
                color:
                  matrixKind === 'covariance'
                    ? 'primary.main'
                    : 'text.disabled',
              }}
            >
              covariance
            </Button>
            <Button
              size="small"
              onClick={() => setMatrixKind('correlation')}
              sx={{
                minWidth: 0,
                px: 0.8,
                py: 0,
                lineHeight: 1.3,
                fontSize: '0.78rem',
                textTransform: 'none',
                color:
                  matrixKind === 'correlation'
                    ? 'primary.main'
                    : 'text.disabled',
              }}
            >
              correlation
            </Button>
          </Box>
          <Typography
            variant="body2"
            color="text.secondary"
            sx={{ display: 'block', mt: 0.8 }}
          >
            {matrixKind === 'covariance'
              ? `Full covariance matrix from ODR fit (${uncertaintyBasis === 'raw' ? 'raw inv(N)' : 'scaled by χ²red'}).`
              : `Full correlation matrix from ODR fit (${uncertaintyBasis === 'raw' ? 'raw covariance' : 'scaled covariance'} basis).`}
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
                background: `${anafisTheme.colors.tabs.fitting.main}40`,
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
                        fontSize: '0.86rem',
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
                        fontSize: '0.86rem',
                        fontFamily: 'monospace',
                        whiteSpace: 'nowrap',
                        px: 0.8,
                        py: 0.5,
                      }}
                    >
                      {rowName}
                    </TableCell>
                    {parameterNames.map((_, colIdx) => {
                      const covValue = displayedMatrix?.[rowIdx]?.[colIdx];
                      const isDiagonal = rowIdx === colIdx;
                      const isValid =
                        covValue !== undefined && Number.isFinite(covValue);

                      return (
                        <TableCell
                          // biome-ignore lint/suspicious/noArrayIndexKey: Matrix positions are static
                          key={`matrix-cov-${rowName}-${colIdx}`}
                          align="center"
                          sx={{
                            fontSize: '0.82rem',
                            fontFamily: 'monospace',
                            whiteSpace: 'nowrap',
                            px: 0.8,
                            py: 0.5,
                            backgroundColor: isDiagonal
                              ? `${anafisTheme.colors.tabs.fitting.main}14`
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

      {/* ── Assumptions dialog ── */}
      <Dialog
        open={assumptionsOpen}
        onClose={() => setAssumptionsOpen(false)}
        maxWidth="md"
        fullWidth
        slotProps={{
          paper: {
            sx: {
              background:
                'linear-gradient(145deg, rgba(26, 26, 32, 0.98) 0%, rgba(18, 18, 22, 0.98) 100%)',
              border: '1px solid rgba(148, 163, 184, 0.2)',
              borderRadius: 2.5,
              backdropFilter: 'blur(16px)',
              maxWidth: 760,
            },
          },
        }}
      >
        <DialogTitle
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            pb: 0.5,
          }}
        >
          <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
            Assumptions & methodology
          </Typography>
          <IconButton
            size="small"
            onClick={() => setAssumptionsOpen(false)}
            sx={{ color: 'text.secondary' }}
          >
            <CloseIcon fontSize="small" />
          </IconButton>
        </DialogTitle>
        <DialogContent sx={{ pt: 0.5 }}>
          <Typography
            variant="caption"
            color="text.disabled"
            sx={{
              display: 'block',
              mb: 1.8,
              fontSize: '0.86rem',
              lineHeight: 1.45,
            }}
          >
            These describe the statistical model, solver strategy, and
            interpretation caveats for the reported results.
          </Typography>
          <Box
            component="ol"
            sx={{
              pl: 2.5,
              m: 0,
              '& li': {
                fontSize: '0.96rem',
                color: 'text.secondary',
                lineHeight: 1.7,
                mb: 1.05,
              },
            }}
          >
            {(assumptions ?? []).map((text) => (
              <li key={text}>{text}</li>
            ))}
          </Box>
        </DialogContent>
      </Dialog>
    </Box>
  );
}
