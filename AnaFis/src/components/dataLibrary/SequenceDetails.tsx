import React, { memo } from 'react';
import {
  Box,
  Typography,
  Chip,
  IconButton,
  Card,
  CardContent,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Stack,
} from '@mui/material';
import {
  Star as StarIcon,
  StarBorder as StarBorderIcon,
  Edit as EditIcon,
  ContentCopy as CopyIcon,
  Delete as DeleteIcon,
} from '@mui/icons-material';
import SequenceChart from './SequenceChart';
import type { DataSequence, SequenceStatistics } from '@/types/dataLibrary';

interface SequenceDetailsProps {
  sequence: DataSequence | null;
  statistics: SequenceStatistics | null;
  onPin: (id: string, isPinned: boolean) => Promise<void>;
  onEdit: () => void;
  onDuplicate: () => void;
  onDelete: () => void;
}

const SequenceDetails: React.FC<SequenceDetailsProps> = memo(({
  sequence,
  statistics,
  onPin,
  onEdit,
  onDuplicate,
  onDelete,
}) => {
  if (!sequence) {
    return (
      <Box sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'center', 
        height: '100%',
        width: '100%',
        flex: 1,
        textAlign: 'center'
      }}>
        <Typography variant="h6" color="text.secondary">
          Select a sequence to view details
        </Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ flexGrow: 1, overflow: 'auto', p: 3 }}>
      {/* Sequence header */}
      <Box sx={{ mb: 3 }}>
        <Stack direction="row" justifyContent="space-between" alignItems="flex-start">
          <Box>
            <Typography variant="h5" gutterBottom>{sequence.name}</Typography>
            <Typography variant="body2" color="text.secondary">{sequence.description}</Typography>
            <Box sx={{ mt: 1, display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
              {sequence.tags.map(tag => (
                <Chip key={tag} label={tag} size="small" />
              ))}
            </Box>
            <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 1 }}>
              Source: {sequence.source} | Unit: {sequence.unit}
            </Typography>
            <Typography variant="caption" color="text.secondary">
              Modified: {new Date(sequence.modified_at).toLocaleString()}
            </Typography>
          </Box>
          
          <Stack direction="row" spacing={1}>
            <IconButton 
              onClick={() => void onPin(sequence.id, !sequence.is_pinned)} 
              size="small"
            >
              {sequence.is_pinned ? <StarIcon /> : <StarBorderIcon />}
            </IconButton>
            <IconButton onClick={onEdit} size="small">
              <EditIcon />
            </IconButton>
            <IconButton onClick={onDuplicate} size="small">
              <CopyIcon />
            </IconButton>
            <IconButton onClick={onDelete} size="small" color="error">
              <DeleteIcon />
            </IconButton>
          </Stack>
        </Stack>
      </Box>

      {/* Statistics */}
      {statistics && (
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>Quick Statistics</Typography>
            <Stack direction="row" spacing={4}>
              <Box>
                <Typography variant="caption" color="text.secondary">Count</Typography>
                <Typography variant="body1">{statistics.count}</Typography>
              </Box>
              <Box>
                <Typography variant="caption" color="text.secondary">Mean</Typography>
                <Typography variant="body1">{statistics.mean.toFixed(4)}</Typography>
              </Box>
              <Box>
                <Typography variant="caption" color="text.secondary">Std Dev</Typography>
                <Typography variant="body1">{statistics.std_dev.toFixed(4)}</Typography>
              </Box>
              <Box>
                <Typography variant="caption" color="text.secondary">Min</Typography>
                <Typography variant="body1">{statistics.min.toFixed(4)}</Typography>
              </Box>
              <Box>
                <Typography variant="caption" color="text.secondary">Max</Typography>
                <Typography variant="body1">{statistics.max.toFixed(4)}</Typography>
              </Box>
              <Box>
                <Typography variant="caption" color="text.secondary">Median</Typography>
                <Typography variant="body1">{statistics.median.toFixed(4)}</Typography>
              </Box>
            </Stack>
          </CardContent>
        </Card>
      )}

      {/* Data preview */}
      <Typography variant="h6" gutterBottom>Data Preview</Typography>
      <TableContainer component={Paper} sx={{ mb: 3, maxHeight: 300 }}>
        <Table stickyHeader size="small">
          <TableHead>
            <TableRow>
              <TableCell>Index</TableCell>
              <TableCell>Value</TableCell>
              {sequence.uncertainties && <TableCell>Uncertainty</TableCell>}
            </TableRow>
          </TableHead>
          <TableBody>
            {sequence.data.slice(0, 50).map((value, index) => (
              <TableRow key={index}>
                <TableCell>{index}</TableCell>
                <TableCell>{value}</TableCell>
                {sequence.uncertainties && (
                  <TableCell>{sequence.uncertainties[index] ?? '-'}</TableCell>
                )}
              </TableRow>
            ))}
            {sequence.data.length > 50 && (
              <TableRow>
                <TableCell colSpan={sequence.uncertainties ? 3 : 2} align="center">
                  <Typography variant="caption" color="text.secondary">
                    ... and {sequence.data.length - 50} more rows
                  </Typography>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </TableContainer>

      {/* Chart Preview */}
      <SequenceChart sequence={sequence} />
    </Box>
  );
});

SequenceDetails.displayName = 'SequenceDetails';

export default SequenceDetails;