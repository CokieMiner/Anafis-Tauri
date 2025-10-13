import React, { useState, useEffect, useCallback } from 'react';
import ReactDOM from 'react-dom/client';
import { ThemeProvider, createTheme, CssBaseline } from '@mui/material';
import {
  Box,
  TextField,
  Button,
  Typography,
  Chip,
  MenuItem,
  Select,
  FormControl,
  InputLabel,
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
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  ListItemIcon,
  Divider,
  Stack,
  Alert,
} from '@mui/material';
import {
  Search as SearchIcon,
  Delete as DeleteIcon,
  Edit as EditIcon,
  Star as StarIcon,
  StarBorder as StarBorderIcon,
  ContentCopy as CopyIcon,
  Folder as FolderIcon,
  Label as LabelIcon,
  FileDownload as ExportIcon,
  CheckBox as CheckBoxIcon,
  CheckBoxOutlineBlank as CheckBoxOutlineBlankIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import * as echarts from 'echarts';
import CustomTitleBar from './components/CustomTitleBar';
import type {
  DataSequence,
  SequenceStatistics,
  SearchRequest,
  SequenceListResponse,
  UpdateSequenceRequest,
} from './types/dataLibrary';

export const DataLibraryWindowContent: React.FC = () => {
  // State management
  const [sequences, setSequences] = useState<DataSequence[]>([]);
  const [selectedSequence, setSelectedSequence] = useState<DataSequence | null>(null);
  const [selectedStats, setSelectedStats] = useState<SequenceStatistics | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [allTags, setAllTags] = useState<string[]>([]);
  const [sortBy, setSortBy] = useState<'name' | 'date_created' | 'date_modified' | 'size'>('date_modified');
  const [sortOrder, setSortOrder] = useState<'ascending' | 'descending'>('descending');
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Edit form state
  const [editName, setEditName] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [editUnit, setEditUnit] = useState('');
  const [editTags, setEditTags] = useState<string[]>([]);
  const [newTagInput, setNewTagInput] = useState('');
  
  // Selection state for multi-select export
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [exportFormat, setExportFormat] = useState<'csv' | 'json'>('csv');

  // Chart ref for ECharts preview
  const chartRef = React.useRef<HTMLDivElement>(null);
  const chartInstanceRef = React.useRef<echarts.ECharts | null>(null);

  // Define loadSequences before it's used in useEffect
  const loadSequences = useCallback(async () => {
    try {
      const searchRequest: SearchRequest = {
        query: searchQuery || undefined,
        tags: selectedTags.length > 0 ? selectedTags : undefined,
        sort_by: sortBy,
        sort_order: sortOrder,
      };
      
      const response = await invoke<SequenceListResponse>('get_sequences', { search: searchRequest });
      setSequences(response.sequences);
      setError(null);
    } catch (err) {
      console.error('Failed to load sequences:', err);
      setError(`Failed to load sequences: ${err}`);
    }
  }, [searchQuery, selectedTags, sortBy, sortOrder]);

  const loadAllTags = async () => {
    try {
      const tags = await invoke<string[]>('get_all_tags');
      setAllTags(tags);
    } catch (err) {
      console.error('Failed to load tags:', err);
    }
  };

  const loadStatistics = async (id: string) => {
    try {
      const stats = await invoke<SequenceStatistics | null>('get_sequence_stats', { id });
      setSelectedStats(stats);
    } catch (err) {
      console.error('Failed to load statistics:', err);
    }
  };

  // Load sequences on mount and when search/filter changes
  useEffect(() => {
    loadSequences();
    loadAllTags();
  }, [loadSequences]); // loadSequences now includes all dependencies via useCallback

  // Load statistics when a sequence is selected
  useEffect(() => {
    if (selectedSequence) {
      loadStatistics(selectedSequence.id);
    }
  }, [selectedSequence]);

  // Initialize and update ECharts preview
  useEffect(() => {
    if (!chartRef.current) return;

    // Initialize chart if not already done
    if (!chartInstanceRef.current) {
      chartInstanceRef.current = echarts.init(chartRef.current, null, {
        renderer: 'canvas',
        devicePixelRatio: 2,
      });
    }

    // Update chart with sequence data
    if (selectedSequence) {
      const xData = Array.from({ length: selectedSequence.data.length }, (_, i) => i);
      const yData = selectedSequence.data;
      const errorData = selectedSequence.uncertainties;

      const series: echarts.SeriesOption[] = [
        {
          name: selectedSequence.name,
          type: 'line',
          data: xData.map((x, i) => [x, yData[i]]),
          showSymbol: true,
          symbolSize: 4,
          itemStyle: { color: '#90caf9' },
          lineStyle: { color: '#90caf9', width: 2 },
        },
      ];

      // Add error bars if uncertainties exist
      if (errorData && errorData.length > 0) {
        series.push({
          name: 'Uncertainties',
          type: 'custom',
          renderItem: (_params: echarts.CustomSeriesRenderItemParams, api: echarts.CustomSeriesRenderItemAPI) => {
            const point = api.coord([api.value(0), api.value(1)]);
            const errorValue = api.value(2) as number;
            const yTop = api.coord([api.value(0), (api.value(1) as number) + errorValue]);
            const yBottom = api.coord([api.value(0), (api.value(1) as number) - errorValue]);
            
            return {
              type: 'group',
              children: [
                {
                  type: 'line',
                  shape: { x1: point[0], y1: yTop[1], x2: point[0], y2: yBottom[1] },
                  style: { stroke: '#f44336', lineWidth: 1.5 },
                },
                {
                  type: 'line',
                  shape: { x1: point[0] - 4, y1: yTop[1], x2: point[0] + 4, y2: yTop[1] },
                  style: { stroke: '#f44336', lineWidth: 1.5 },
                },
                {
                  type: 'line',
                  shape: { x1: point[0] - 4, y1: yBottom[1], x2: point[0] + 4, y2: yBottom[1] },
                  style: { stroke: '#f44336', lineWidth: 1.5 },
                },
              ],
            };
          },
          data: xData.map((x, i) => [x, yData[i], errorData[i]]),
          z: 1,
          silent: true,
        });
      }

      const option: echarts.EChartsOption = {
        backgroundColor: 'transparent',
        grid: {
          left: 60,
          right: 20,
          top: 20,
          bottom: 40,
          containLabel: false,
        },
        xAxis: {
          type: 'value',
          name: 'Index',
          nameLocation: 'middle',
          nameGap: 30,
          nameTextStyle: { color: '#ffffff', fontSize: 11 },
          axisLine: { lineStyle: { color: 'rgba(255,255,255,0.3)' } },
          axisLabel: { color: '#ffffff', fontSize: 10 },
          splitLine: { lineStyle: { color: 'rgba(255,255,255,0.1)' } },
        },
        yAxis: {
          type: 'value',
          name: `${selectedSequence.name} (${selectedSequence.unit})`,
          nameLocation: 'middle',
          nameGap: 45,
          nameTextStyle: { color: '#ffffff', fontSize: 11 },
          axisLine: { lineStyle: { color: 'rgba(255,255,255,0.3)' } },
          axisLabel: { color: '#ffffff', fontSize: 10 },
          splitLine: { lineStyle: { color: 'rgba(255,255,255,0.1)' } },
        },
        series,
        tooltip: {
          trigger: 'axis',
          axisPointer: { type: 'cross' },
          backgroundColor: 'rgba(0,0,0,0.8)',
          borderColor: '#90caf9',
          textStyle: { color: '#ffffff' },
        },
      };

      chartInstanceRef.current.setOption(option, true);
    } else {
      // Clear chart if no sequence selected
      chartInstanceRef.current.clear();
    }

    return () => {
      // Cleanup on unmount
      if (chartInstanceRef.current) {
        chartInstanceRef.current.dispose();
        chartInstanceRef.current = null;
      }
    };
  }, [selectedSequence]);

  const handlePinSequence = async (id: string, isPinned: boolean) => {
    try {
      await invoke('pin_sequence', { id, isPinned: !isPinned });
      await loadSequences();
      if (selectedSequence?.id === id) {
        setSelectedSequence({ ...selectedSequence, is_pinned: !isPinned });
      }
    } catch (err) {
      console.error('Failed to pin sequence:', err);
      setError(`Failed to pin sequence: ${err}`);
    }
  };

  const handleDeleteSequence = async () => {
    if (!selectedSequence) return;
    
    try {
      await invoke('delete_sequence', { id: selectedSequence.id });
      setDeleteDialogOpen(false);
      setSelectedSequence(null);
      setSelectedStats(null);
      await loadSequences();
    } catch (err) {
      console.error('Failed to delete sequence:', err);
      setError(`Failed to delete sequence: ${err}`);
    }
  };

  const handleDuplicateSequence = async () => {
    if (!selectedSequence) return;
    
    try {
      const newName = `${selectedSequence.name} (copy)`;
      await invoke('duplicate_sequence', { id: selectedSequence.id, newName });
      await loadSequences();
    } catch (err) {
      console.error('Failed to duplicate sequence:', err);
      setError(`Failed to duplicate sequence: ${err}`);
    }
  };

  const handleOpenEditDialog = () => {
    if (!selectedSequence) return;
    setEditName(selectedSequence.name);
    setEditDescription(selectedSequence.description);
    setEditUnit(selectedSequence.unit);
    setEditTags(selectedSequence.tags);
    setEditDialogOpen(true);
  };

  const handleSaveEdit = async () => {
    if (!selectedSequence) return;
    
    try {
      const request: UpdateSequenceRequest = {
        id: selectedSequence.id,
        name: editName !== selectedSequence.name ? editName : undefined,
        description: editDescription !== selectedSequence.description ? editDescription : undefined,
        unit: editUnit !== selectedSequence.unit ? editUnit : undefined,
        tags: JSON.stringify(editTags) !== JSON.stringify(selectedSequence.tags) ? editTags : undefined,
      };
      
      await invoke('update_sequence', { request });
      setEditDialogOpen(false);
      await loadSequences();
      
      // Update selected sequence
      const updatedSequence = { ...selectedSequence };
      if (request.name) updatedSequence.name = request.name;
      if (request.description) updatedSequence.description = request.description;
      if (request.unit) updatedSequence.unit = request.unit;
      if (request.tags) updatedSequence.tags = request.tags;
      setSelectedSequence(updatedSequence);
    } catch (err) {
      console.error('Failed to update sequence:', err);
      setError(`Failed to update sequence: ${err}`);
    }
  };

  const handleAddTag = () => {
    if (newTagInput.trim() && !editTags.includes(newTagInput.trim())) {
      setEditTags([...editTags, newTagInput.trim()]);
      setNewTagInput('');
    }
  };

  const handleRemoveTag = (tag: string) => {
    setEditTags(editTags.filter(t => t !== tag));
  };

  const handleToggleTagFilter = (tag: string) => {
    if (selectedTags.includes(tag)) {
      setSelectedTags(selectedTags.filter(t => t !== tag));
    } else {
      setSelectedTags([...selectedTags, tag]);
    }
  };
  
  // Selection handlers
  const handleToggleSelection = (id: string) => {
    const newSelected = new Set(selectedIds);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    setSelectedIds(newSelected);
  };
  
  const handleSelectAll = () => {
    setSelectedIds(new Set(sequences.map(s => s.id)));
  };
  
  const handleSelectNone = () => {
    setSelectedIds(new Set());
  };
  
  // Export handlers
  const handleExport = async () => {
    if (selectedIds.size === 0) {
      setError('Please select at least one sequence to export');
      return;
    }
    
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const extension = exportFormat === 'csv' ? '.csv' : '.json';
      const filePath = await save({
        filters: [{
          name: exportFormat.toUpperCase(),
          extensions: [exportFormat]
        }],
        defaultPath: `anafis_export${extension}`
      });
      
      if (filePath) {
        if (exportFormat === 'csv') {
          await invoke('export_sequences_csv', {
            sequenceIds: Array.from(selectedIds),
            filePath
          });
        } else {
          await invoke('export_sequences_json', {
            sequenceIds: Array.from(selectedIds),
            filePath
          });
        }
        
        setExportDialogOpen(false);
        setSelectedIds(new Set());
        setError(null);
      }
    } catch (err) {
      console.error('Export failed:', err);
      setError(`Export failed: ${err}`);
    }
  };

  // Group sequences by category
  const pinnedSequences = sequences.filter(s => s.is_pinned);
  const unpinnedSequences = sequences.filter(s => !s.is_pinned);

  return (
    <Box sx={{ width: '100%', height: '100vh', display: 'flex', flexDirection: 'column', bgcolor: 'background.default' }}>
      <CustomTitleBar title="Data Library" />
      
      {/* Toolbar */}
      <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
        <Stack direction="row" spacing={2} alignItems="center">
          <TextField
            size="small"
            placeholder="Search sequences..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            InputProps={{
              startAdornment: <SearchIcon sx={{ mr: 1, color: 'text.secondary' }} />,
            }}
            sx={{ flexGrow: 1 }}
          />
          
          <FormControl size="small" sx={{ minWidth: 150 }}>
            <InputLabel>Sort By</InputLabel>
            <Select
              value={sortBy}
              label="Sort By"
              onChange={(e) => setSortBy(e.target.value as 'name' | 'date_created' | 'date_modified' | 'size')}
            >
              <MenuItem value="name">Name</MenuItem>
              <MenuItem value="date_created">Date Created</MenuItem>
              <MenuItem value="date_modified">Date Modified</MenuItem>
              <MenuItem value="size">Size</MenuItem>
            </Select>
          </FormControl>
          
          <FormControl size="small" sx={{ minWidth: 120 }}>
            <InputLabel>Order</InputLabel>
            <Select
              value={sortOrder}
              label="Order"
              onChange={(e) => setSortOrder(e.target.value as 'ascending' | 'descending')}
            >
              <MenuItem value="ascending">Ascending</MenuItem>
              <MenuItem value="descending">Descending</MenuItem>
            </Select>
          </FormControl>
          
          {/* Export button */}
          <Button
            variant="contained"
            startIcon={<ExportIcon />}
            onClick={() => setExportDialogOpen(true)}
            disabled={selectedIds.size === 0}
          >
            Export ({selectedIds.size})
          </Button>
          
          {/* Selection controls */}
          {selectedIds.size > 0 && (
            <>
              <Button
                size="small"
                variant="outlined"
                onClick={handleSelectNone}
              >
                Clear Selection
              </Button>
            </>
          )}
          {selectedIds.size === 0 && sequences.length > 0 && (
            <Button
              size="small"
              variant="outlined"
              onClick={handleSelectAll}
            >
              Select All
            </Button>
          )}
        </Stack>
        
        {/* Tag filters */}
        {allTags.length > 0 && (
          <Box sx={{ mt: 1 }}>
            <Typography variant="caption" color="text.secondary">Filter by tags:</Typography>
            <Box sx={{ mt: 0.5, display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
              {allTags.map(tag => (
                <Chip
                  key={tag}
                  label={tag}
                  size="small"
                  color={selectedTags.includes(tag) ? 'primary' : 'default'}
                  onClick={() => handleToggleTagFilter(tag)}
                  icon={<LabelIcon />}
                />
              ))}
            </Box>
          </Box>
        )}
      </Box>

      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ m: 2 }}>
          {error}
        </Alert>
      )}

      {/* Main content */}
      <Box sx={{ display: 'flex', flexGrow: 1, overflow: 'hidden' }}>
        {/* Sidebar */}
        <Box sx={{ width: 250, borderRight: 1, borderColor: 'divider', overflow: 'auto' }}>
          <List dense>
            <ListItem>
              <Typography variant="subtitle2" color="text.secondary">
                ALL SEQUENCES ({sequences.length})
              </Typography>
            </ListItem>
            
            {pinnedSequences.length > 0 && (
              <>
                <Divider />
                <ListItem>
                  <Typography variant="caption" color="text.secondary">PINNED</Typography>
                </ListItem>
                {pinnedSequences.map(seq => (
                  <ListItemButton
                    key={seq.id}
                    selected={selectedSequence?.id === seq.id}
                    onClick={() => setSelectedSequence(seq)}
                  >
                    <IconButton
                      size="small"
                      edge="start"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleToggleSelection(seq.id);
                      }}
                      sx={{ mr: 1 }}
                    >
                      {selectedIds.has(seq.id) ? (
                        <CheckBoxIcon fontSize="small" color="primary" />
                      ) : (
                        <CheckBoxOutlineBlankIcon fontSize="small" />
                      )}
                    </IconButton>
                    <ListItemIcon>
                      <StarIcon fontSize="small" color="primary" />
                    </ListItemIcon>
                    <ListItemText primary={seq.name} secondary={`${seq.data.length} points`} />
                  </ListItemButton>
                ))}
              </>
            )}
            
            {unpinnedSequences.length > 0 && (
              <>
                <Divider />
                <ListItem>
                  <Typography variant="caption" color="text.secondary">OTHER</Typography>
                </ListItem>
                {unpinnedSequences.map(seq => (
                  <ListItemButton
                    key={seq.id}
                    selected={selectedSequence?.id === seq.id}
                    onClick={() => setSelectedSequence(seq)}
                  >
                    <IconButton
                      size="small"
                      edge="start"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleToggleSelection(seq.id);
                      }}
                      sx={{ mr: 1 }}
                    >
                      {selectedIds.has(seq.id) ? (
                        <CheckBoxIcon fontSize="small" color="primary" />
                      ) : (
                        <CheckBoxOutlineBlankIcon fontSize="small" />
                      )}
                    </IconButton>
                    <ListItemIcon>
                      <FolderIcon fontSize="small" />
                    </ListItemIcon>
                    <ListItemText primary={seq.name} secondary={`${seq.data.length} points`} />
                  </ListItemButton>
                ))}
              </>
            )}
          </List>
        </Box>

        {/* Detail panel */}
        <Box sx={{ flexGrow: 1, overflow: 'auto', p: 3 }}>
          {selectedSequence ? (
            <>
              {/* Sequence header */}
              <Box sx={{ mb: 3 }}>
                <Stack direction="row" justifyContent="space-between" alignItems="flex-start">
                  <Box>
                    <Typography variant="h5" gutterBottom>{selectedSequence.name}</Typography>
                    <Typography variant="body2" color="text.secondary">{selectedSequence.description}</Typography>
                    <Box sx={{ mt: 1, display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
                      {selectedSequence.tags.map(tag => (
                        <Chip key={tag} label={tag} size="small" />
                      ))}
                    </Box>
                    <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 1 }}>
                      Source: {selectedSequence.source} | Unit: {selectedSequence.unit}
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                      Modified: {new Date(selectedSequence.modified_at).toLocaleString()}
                    </Typography>
                  </Box>
                  
                  <Stack direction="row" spacing={1}>
                    <IconButton onClick={() => handlePinSequence(selectedSequence.id, selectedSequence.is_pinned)} size="small">
                      {selectedSequence.is_pinned ? <StarIcon /> : <StarBorderIcon />}
                    </IconButton>
                    <IconButton onClick={handleOpenEditDialog} size="small">
                      <EditIcon />
                    </IconButton>
                    <IconButton onClick={handleDuplicateSequence} size="small">
                      <CopyIcon />
                    </IconButton>
                    <IconButton onClick={() => setDeleteDialogOpen(true)} size="small" color="error">
                      <DeleteIcon />
                    </IconButton>
                  </Stack>
                </Stack>
              </Box>

              {/* Statistics */}
              {selectedStats && (
                <Card sx={{ mb: 3 }}>
                  <CardContent>
                    <Typography variant="h6" gutterBottom>Quick Statistics</Typography>
                    <Stack direction="row" spacing={4}>
                      <Box>
                        <Typography variant="caption" color="text.secondary">Count</Typography>
                        <Typography variant="body1">{selectedStats.count}</Typography>
                      </Box>
                      <Box>
                        <Typography variant="caption" color="text.secondary">Mean</Typography>
                        <Typography variant="body1">{selectedStats.mean.toFixed(4)}</Typography>
                      </Box>
                      <Box>
                        <Typography variant="caption" color="text.secondary">Std Dev</Typography>
                        <Typography variant="body1">{selectedStats.std_dev.toFixed(4)}</Typography>
                      </Box>
                      <Box>
                        <Typography variant="caption" color="text.secondary">Min</Typography>
                        <Typography variant="body1">{selectedStats.min.toFixed(4)}</Typography>
                      </Box>
                      <Box>
                        <Typography variant="caption" color="text.secondary">Max</Typography>
                        <Typography variant="body1">{selectedStats.max.toFixed(4)}</Typography>
                      </Box>
                      <Box>
                        <Typography variant="caption" color="text.secondary">Median</Typography>
                        <Typography variant="body1">{selectedStats.median.toFixed(4)}</Typography>
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
                      {selectedSequence.uncertainties && <TableCell>Uncertainty</TableCell>}
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {selectedSequence.data.slice(0, 50).map((value, index) => (
                      <TableRow key={index}>
                        <TableCell>{index}</TableCell>
                        <TableCell>{value}</TableCell>
                        {selectedSequence.uncertainties && (
                          <TableCell>{selectedSequence.uncertainties[index]}</TableCell>
                        )}
                      </TableRow>
                    ))}
                    {selectedSequence.data.length > 50 && (
                      <TableRow>
                        <TableCell colSpan={selectedSequence.uncertainties ? 3 : 2} align="center">
                          <Typography variant="caption" color="text.secondary">
                            ... and {selectedSequence.data.length - 50} more rows
                          </Typography>
                        </TableCell>
                      </TableRow>
                    )}
                  </TableBody>
                </Table>
              </TableContainer>

              {/* Mini chart */}
              <Typography variant="h6" gutterBottom>Chart Preview</Typography>
              <Box sx={{ bgcolor: 'background.paper', p: 2, borderRadius: 1 }}>
                <Box
                  ref={chartRef}
                  sx={{
                    width: '100%',
                    height: 300,
                  }}
                />
              </Box>
            </>
          ) : (
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
              <Typography variant="h6" color="text.secondary">
                Select a sequence to view details
              </Typography>
            </Box>
          )}
        </Box>
      </Box>

      {/* Edit Dialog */}
      <Dialog open={editDialogOpen} onClose={() => setEditDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Edit Sequence</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Name"
            value={editName}
            onChange={(e) => setEditName(e.target.value)}
            margin="normal"
          />
          <TextField
            fullWidth
            multiline
            rows={3}
            label="Description"
            value={editDescription}
            onChange={(e) => setEditDescription(e.target.value)}
            margin="normal"
          />
          <TextField
            fullWidth
            label="Unit"
            value={editUnit}
            onChange={(e) => setEditUnit(e.target.value)}
            margin="normal"
          />
          <Box sx={{ mt: 2 }}>
            <Typography variant="subtitle2" gutterBottom>Tags</Typography>
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mb: 1 }}>
              {editTags.map(tag => (
                <Chip
                  key={tag}
                  label={tag}
                  size="small"
                  onDelete={() => handleRemoveTag(tag)}
                />
              ))}
            </Box>
            <Stack direction="row" spacing={1}>
              <TextField
                size="small"
                placeholder="Add tag"
                value={newTagInput}
                onChange={(e) => setNewTagInput(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && handleAddTag()}
              />
              <Button onClick={handleAddTag}>Add</Button>
            </Stack>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setEditDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleSaveEdit} variant="contained">Save</Button>
        </DialogActions>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog open={deleteDialogOpen} onClose={() => setDeleteDialogOpen(false)}>
        <DialogTitle>Confirm Delete</DialogTitle>
        <DialogContent>
          <Typography>
            Are you sure you want to delete "{selectedSequence?.name}"? This action cannot be undone.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleDeleteSequence} color="error" variant="contained">Delete</Button>
        </DialogActions>
      </Dialog>
      
      {/* Export Dialog */}
      <Dialog open={exportDialogOpen} onClose={() => setExportDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Export Sequences</DialogTitle>
        <DialogContent>
          <Typography gutterBottom>
            Export {selectedIds.size} sequence{selectedIds.size > 1 ? 's' : ''} to a universal format
          </Typography>
          
          <FormControl fullWidth sx={{ mt: 2 }}>
            <InputLabel>Export Format</InputLabel>
            <Select
              value={exportFormat}
              label="Export Format"
              onChange={(e) => setExportFormat(e.target.value as 'csv' | 'json')}
            >
              <MenuItem value="csv">
                <Box>
                  <Typography variant="body1">CSV (Comma-Separated Values)</Typography>
                  <Typography variant="caption" color="text.secondary">
                    Universal format - Excel, MATLAB, Python, R, Origin, Igor Pro
                  </Typography>
                </Box>
              </MenuItem>
              <MenuItem value="json">
                <Box>
                  <Typography variant="body1">JSON (JavaScript Object Notation)</Typography>
                  <Typography variant="caption" color="text.secondary">
                    Web-friendly format with full metadata - Python, JavaScript, Julia
                  </Typography>
                </Box>
              </MenuItem>
            </Select>
          </FormControl>
          
          <Alert severity="info" sx={{ mt: 2 }}>
            {exportFormat === 'csv' 
              ? 'CSV format: Each sequence becomes a column with headers. Uncertainties included as separate columns.'
              : 'JSON format: Preserves all metadata including tags, descriptions, units, and timestamps.'}
          </Alert>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setExportDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleExport} variant="contained" startIcon={<ExportIcon />}>
            Export
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

const darkTheme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: '#90caf9',
    },
    secondary: {
      main: '#f48fb1',
    },
    background: {
      default: '#121212',
      paper: '#1e1e1e',
    },
  },
});

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <DataLibraryWindowContent />
    </ThemeProvider>
  </React.StrictMode>
);
