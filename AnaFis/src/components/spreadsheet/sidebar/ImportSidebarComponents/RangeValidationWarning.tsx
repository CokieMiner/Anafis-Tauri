import React from 'react';
import { Box, Typography } from '@mui/material';
import type { ImportResult } from '@/types/import';

interface RangeValidationWarningProps {
  validation: ImportResult['rangeValidation'];
}

/**
 * Reusable component for displaying range validation warnings
 */
export const RangeValidationWarning: React.FC<RangeValidationWarningProps> = ({ validation }) => {
  if (!validation) {
    return null;
  }

  return (
    <Box
      sx={{
        mt: 2,
        p: 1.5,
        bgcolor: 'rgba(33, 150, 243, 0.05)',
        borderRadius: '6px',
        border: '1px solid rgba(33, 150, 243, 0.2)',
      }}
    >
      {/* Show selected range dimensions if range is equal or bigger than file data */}
      {!validation.willTruncate && validation.selectedRange && (
        <Typography
          sx={{
            color: 'rgba(255, 255, 255, 0.9)',
            fontSize: 14,
            fontWeight: 500,
            textAlign: 'center',
          }}
        >
          {validation.selectedRange.rows} × {validation.selectedRange.columns}
        </Typography>
      )}

      {/* Validation warnings */}
      {validation.warnings.length > 0 && (
        <Box>
          {validation.warnings.map((warning: string, index: number) => (
            <Typography
              key={index}
              sx={{
                color: validation.willTruncate ? '#ffb74d' : 'rgba(255, 255, 255, 0.7)',
                fontSize: 10,
                mb: 0.5,
                display: 'flex',
                alignItems: 'flex-start',
                gap: 0.5,
                mt: !validation.willTruncate && validation.selectedRange ? 0.5 : 0,
              }}
            >
              <span style={{ fontSize: '8px', marginTop: '1px' }}>
                {validation.willTruncate ? '⚠️' : 'ℹ️'}
              </span>
              {warning}
            </Typography>
          ))}
        </Box>
      )}
    </Box>
  );
};
