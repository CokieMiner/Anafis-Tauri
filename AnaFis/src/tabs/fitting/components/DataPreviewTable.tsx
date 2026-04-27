import { Box, Typography } from '@mui/material';
import type {
  ImportedColumn,
  ImportedData,
} from '@/tabs/fitting/types/fittingTypes';

interface DataPreviewTableProps {
  importedData: ImportedData;
  maxPreviewRows?: number;
}

export default function DataPreviewTable({
  importedData,
  maxPreviewRows = 5,
}: DataPreviewTableProps) {
  if (importedData.columns.length === 0) {
    return null;
  }

  const previewRows = Math.min(maxPreviewRows, importedData.rowCount);

  return (
    <Box sx={{ mt: 1 }}>
      <Typography
        variant="caption"
        color="text.secondary"
        sx={{ mb: 0.5, display: 'block' }}
      >
        {importedData.rowCount} rows · {importedData.columns.length} cols (
        {importedData.sourceName})
      </Typography>

      <Box
        sx={{
          overflow: 'auto',
          maxHeight: 140,
          borderRadius: 1,
          border: '1px solid rgba(148, 163, 184, 0.12)',
          background: 'rgba(0,0,0,0.2)',
          '&::-webkit-scrollbar': { width: 4, height: 4 },
          '&::-webkit-scrollbar-thumb': {
            background: 'rgba(255,179,0,0.25)',
            borderRadius: 2,
          },
        }}
      >
        <Box
          component="table"
          sx={{
            width: '100%',
            borderCollapse: 'collapse',
            fontSize: '0.7rem',
            fontFamily: 'monospace',
            tableLayout: 'fixed',
          }}
        >
          <Box component="thead">
            <Box component="tr">
              {importedData.columns.map((col: ImportedColumn) => (
                <Box
                  component="th"
                  key={`th-${col.name}`}
                  sx={{
                    px: 0.75,
                    py: 0.4,
                    textAlign: 'left',
                    fontWeight: 700,
                    fontSize: '0.65rem',
                    color: col.name.startsWith('σ')
                      ? 'warning.dark'
                      : 'text.secondary',
                    borderBottom: '1px solid rgba(148,163,184,0.15)',
                    whiteSpace: 'nowrap',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    maxWidth: 90,
                  }}
                  title={col.name}
                >
                  {col.name}
                </Box>
              ))}
            </Box>
          </Box>

          <Box component="tbody">
            {Array.from({ length: previewRows }, (_, rowIdx) => (
              <Box
                component="tr"
                // biome-ignore lint/suspicious/noArrayIndexKey: Table row order is static
                key={`preview-tr-${rowIdx}`}
                sx={{
                  '&:hover': { background: 'rgba(255,179,0,0.04)' },
                }}
              >
                {importedData.columns.map((col: ImportedColumn) => (
                  <Box
                    component="td"
                    key={`td-${col.name}`}
                    sx={{
                      px: 0.75,
                      py: 0.25,
                      color: 'text.secondary',
                      borderBottom: '1px solid rgba(255,255,255,0.03)',
                      whiteSpace: 'nowrap',
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      maxWidth: 90,
                    }}
                  >
                    {col.data[rowIdx] !== undefined
                      ? Number(col.data[rowIdx]).toPrecision(5)
                      : '—'}
                  </Box>
                ))}
              </Box>
            ))}
          </Box>
        </Box>
      </Box>
    </Box>
  );
}
