import {
  Box,
  Button,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  type SelectChangeEvent,
  TextField,
  Typography,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import type React from 'react';
import { useCallback, useEffect, useState } from 'react';
import { anafisColors } from '@/shared/theme';
import SidebarCard from '@/tabs/spreadsheet/components/sidebar/SidebarCard';
import { sidebarStyles } from '@/tabs/spreadsheet/components/sidebar/utils/sidebarStyles';
import type {
  CellValue,
  SpreadsheetRef,
} from '@/tabs/spreadsheet/types/SpreadsheetInterface';

const CONFIDENCE_PRESETS = [
  { label: '1σ (68.27%)', value: 68.27 },
  { label: '90%', value: 90 },
  { label: '95%', value: 95 },
  { label: '2σ (95.45%)', value: 95.45 },
  { label: '99%', value: 99 },
  { label: '3σ (99.73%)', value: 99.73 },
  { label: 'Custom', value: -1 },
] as const;

interface ConfidenceConversionSectionProps {
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  selection: string;
}

export default function ConfidenceConversionSection({
  spreadsheetRef,
  selection,
}: ConfidenceConversionSectionProps) {
  const [sourceConfidence, setSourceConfidence] = useState<number>(68.27);
  const [sourceCustom, setSourceCustom] = useState<string>('');
  const [targetConfidence, setTargetConfidence] = useState<number>(95);
  const [targetCustom, setTargetCustom] = useState<string>('');
  const [factor, setFactor] = useState<number | null>(null);
  const [isApplying, setIsApplying] = useState(false);
  const [status, setStatus] = useState<string>('');
  const [statusError, setStatusError] = useState(false);

  const effectiveSource =
    sourceConfidence === -1 ? parseFloat(sourceCustom) : sourceConfidence;
  const effectiveTarget =
    targetConfidence === -1 ? parseFloat(targetCustom) : targetConfidence;

  // Compute conversion factor when confidence levels change
  useEffect(() => {
    let cancelled = false;

    async function compute() {
      if (Number.isNaN(effectiveSource) || Number.isNaN(effectiveTarget)) {
        setFactor(null);
        return;
      }
      if (effectiveSource <= 0 || effectiveSource >= 100) {
        setFactor(null);
        return;
      }
      if (effectiveTarget <= 0 || effectiveTarget >= 100) {
        setFactor(null);
        return;
      }
      try {
        const sourceSigma = (await invoke('convert_confidence_to_sigma', {
          confidencePercent: effectiveSource,
        })) as number;
        const targetSigma = (await invoke('convert_confidence_to_sigma', {
          confidencePercent: effectiveTarget,
        })) as number;
        if (!cancelled) {
          setFactor(targetSigma / sourceSigma);
        }
      } catch {
        if (!cancelled) setFactor(null);
      }
    }

    void compute();
    return () => {
      cancelled = true;
    };
  }, [effectiveSource, effectiveTarget]);

  const handleApply = useCallback(async () => {
    if (!spreadsheetRef.current) return;
    if (!selection || !factor) return;

    setIsApplying(true);
    setStatus('');
    setStatusError(false);

    try {
      const cellData = await spreadsheetRef.current.getRangeFull(selection);
      let convertedCount = 0;
      let totalCellCount = 0;

      const updated = cellData.map((row) =>
        row.map((cell) => {
          if (!cell) return cell;
          totalCellCount++;

          const u = (
            cell.meta?.customFields as Record<string, unknown> | undefined
          )?.uncertainty as
            | {
                upperBound: number;
                lowerBound?: number;
                upperType: string;
                lowerType?: string;
              }
            | undefined;

          if (!u || typeof u.upperBound !== 'number') return cell;

          const newUpper = u.upperBound * factor;
          const updatedUncertainty = {
            ...u,
            upperBound: newUpper,
            ...(typeof u.lowerBound === 'number'
              ? { lowerBound: u.lowerBound * factor }
              : {}),
          };

          convertedCount++;
          const result: CellValue = {};
          if (cell.v !== undefined && cell.v !== null) {
            result.v = cell.v;
          }
          if (cell.f) result.f = cell.f;
          if (cell.meta) {
            result.meta = {
              ...cell.meta,
              customFields: {
                ...(cell.meta.customFields as Record<string, unknown>),
                uncertainty: updatedUncertainty,
              },
            };
          } else {
            result.meta = {
              customFields: {
                uncertainty: updatedUncertainty,
              },
            };
          }
          return result;
        })
      );

      if (convertedCount === 0) {
        setStatus('No uncertainty cells found in selection');
        setStatusError(true);
        return;
      }

      await spreadsheetRef.current.updateRange(selection, updated);
      setStatus(`Converted ${convertedCount} of ${totalCellCount} cells`);
    } catch (err: unknown) {
      setStatus(String(err));
      setStatusError(true);
    } finally {
      setIsApplying(false);
    }
  }, [spreadsheetRef, selection, factor]);

  const handleSourceChange = (e: SelectChangeEvent<number>) => {
    setSourceConfidence(Number(e.target.value));
  };

  const handleTargetChange = (e: SelectChangeEvent<number>) => {
    setTargetConfidence(Number(e.target.value));
  };

  const isValid =
    factor !== null &&
    !Number.isNaN(factor) &&
    selection.length > 0 &&
    !Number.isNaN(effectiveSource) &&
    !Number.isNaN(effectiveTarget);

  return (
    <Box sx={{ px: 1, pb: 2 }}>
      <SidebarCard title="Confidence Conversion" sx={{ mx: 0.5 }}>
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
          {/* Selection display */}
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              gap: 1,
            }}
          >
            <Typography sx={sidebarStyles.text.label}>Selection:</Typography>
            <Typography
              sx={{
                fontFamily: 'monospace',
                fontSize: 12,
                color: anafisColors.spreadsheet,
                fontWeight: 600,
                flex: 1,
              }}
            >
              {selection || 'None'}
            </Typography>
          </Box>

          {/* Source confidence */}
          <FormControl size="small" fullWidth>
            <InputLabel sx={{ color: 'rgba(255,255,255,0.7)', fontSize: 12 }}>
              From Confidence
            </InputLabel>
            <Select
              value={sourceConfidence}
              onChange={handleSourceChange}
              label="From Confidence"
              sx={{
                ...sidebarStyles.input,
                fontFamily: 'monospace',
                fontSize: 12,
                color: 'white',
              }}
            >
              {CONFIDENCE_PRESETS.map((p) => (
                <MenuItem key={`src-${p.value}`} value={p.value}>
                  {p.label}
                </MenuItem>
              ))}
            </Select>
          </FormControl>

          {sourceConfidence === -1 && (
            <TextField
              label="Custom Source (%)"
              type="number"
              value={sourceCustom}
              onChange={(e) => setSourceCustom(e.target.value)}
              placeholder="68.27"
              size="small"
              fullWidth
              sx={sidebarStyles.input}
              slotProps={{
                input: {
                  style: {
                    color: 'white',
                    fontFamily: 'monospace',
                    fontSize: 12,
                  },
                },
                inputLabel: {
                  style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 },
                },
                htmlInput: { min: 0, max: 100, step: 0.01 },
              }}
            />
          )}

          {/* Target confidence */}
          <FormControl size="small" fullWidth>
            <InputLabel sx={{ color: 'rgba(255,255,255,0.7)', fontSize: 12 }}>
              To Confidence
            </InputLabel>
            <Select
              value={targetConfidence}
              onChange={handleTargetChange}
              label="To Confidence"
              sx={{
                ...sidebarStyles.input,
                fontFamily: 'monospace',
                fontSize: 12,
                color: 'white',
              }}
            >
              {CONFIDENCE_PRESETS.map((p) => (
                <MenuItem key={`tgt-${p.value}`} value={p.value}>
                  {p.label}
                </MenuItem>
              ))}
            </Select>
          </FormControl>

          {targetConfidence === -1 && (
            <TextField
              label="Custom Target (%)"
              type="number"
              value={targetCustom}
              onChange={(e) => setTargetCustom(e.target.value)}
              placeholder="95"
              size="small"
              fullWidth
              sx={sidebarStyles.input}
              slotProps={{
                input: {
                  style: {
                    color: 'white',
                    fontFamily: 'monospace',
                    fontSize: 12,
                  },
                },
                inputLabel: {
                  style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 },
                },
                htmlInput: { min: 0, max: 100, step: 0.01 },
              }}
            />
          )}

          {/* Factor preview */}
          {factor !== null && !Number.isNaN(factor) && (
            <Typography
              sx={{
                fontSize: 11,
                fontFamily: 'monospace',
                color: 'rgba(255,255,255,0.5)',
                textAlign: 'center',
              }}
            >
              Conversion factor: {factor.toFixed(4)}x
            </Typography>
          )}

          {/* Apply button */}
          <Button
            fullWidth
            variant="contained"
            onClick={() => void handleApply()}
            disabled={!isValid || isApplying}
            sx={sidebarStyles.button.primary}
          >
            {isApplying ? 'Converting...' : 'Apply Conversion'}
          </Button>

          {/* Status */}
          {status && (
            <Typography
              sx={{
                fontSize: 11,
                textAlign: 'center',
                color: statusError ? '#f44336' : 'rgba(255,255,255,0.6)',
              }}
            >
              {status}
            </Typography>
          )}
        </Box>
      </SidebarCard>
    </Box>
  );
}
