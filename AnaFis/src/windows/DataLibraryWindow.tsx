import React, { useState, memo } from 'react';
import ReactDOM from 'react-dom/client';
import { createAnafisTheme } from '@/tabs/spreadsheet/components/sidebar/themes';
import {
  ThemeProvider,
  CssBaseline,
  Box,
  TextField,
  Button,
  Typography,
  Chip,
  MenuItem,
  Select,
  SelectChangeEvent,
  FormControl,
  InputLabel,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Stack,
  Alert,
  Divider,
  IconButton,
} from '@mui/material';
import {
  Search as SearchIcon,
  Delete as DeleteIcon,
  Label as LabelIcon,
  FileDownload as ExportIcon,
  ChevronLeft as ChevronLeftIcon,
  ChevronRight as ChevronRightIcon,
  FirstPage as FirstPageIcon,
  LastPage as LastPageIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import CustomTitleBar from '@/shared/components/CustomTitleBar';
import SequenceList from '@/shared/dataLibrary/components/SequenceList';
import SequenceDetails from '@/shared/dataLibrary/components/SequenceDetails';
import { useDataLibrary } from '@/shared/dataLibrary/managers/useDataLibraryManager';
import type { UpdateSequenceRequest, DataSequence } from '@/core/types/dataLibrary';

// Toolbar props interface
interface DataLibraryToolbarProps {
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  sortBy: 'name' | 'date_created' | 'date_modified' | 'size';
  setSortBy: (sortBy: 'name' | 'date_created' | 'date_modified' | 'size') => void;
  sortOrder: 'ascending' | 'descending';
  setSortOrder: (sortOrder: 'ascending' | 'descending') => void;
  selectedIds: Set<string>;
  sequences: DataSequence[];
  onExport: () => void;
  onBulkDelete: () => void;
  onSelectAll: () => void;
  onSelectNone: () => void;
}

// Toolbar component for better organization
const DataLibraryToolbar = memo(({
  searchQuery,
  setSearchQuery,
  sortBy,
  setSortBy,
  sortOrder,
  setSortOrder,
  selectedIds,
  sequences,
  onExport,
  onBulkDelete,
  onSelectAll,
  onSelectNone
}: DataLibraryToolbarProps) => (
  <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
    <Stack direction="row" spacing={2} alignItems="center">
      <TextField
        size="small"
        placeholder="Search sequences..."
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
        slotProps={{
          input: {
            startAdornment: <SearchIcon sx={{ mr: 1, color: 'text.secondary' }} />,
          }
        }}
        sx={{ flexGrow: 1 }}
      />

      <FormControl size="small" sx={{ minWidth: 150 }}>
        <InputLabel>Sort By</InputLabel>
        <Select
          value={sortBy}
          label="Sort By"
          onChange={(e: SelectChangeEvent) => setSortBy(e.target.value as 'name' | 'date_created' | 'date_modified' | 'size')}
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
          onChange={(e: SelectChangeEvent) => setSortOrder(e.target.value as 'ascending' | 'descending')}
        >
          <MenuItem value="ascending">Ascending</MenuItem>
          <MenuItem value="descending">Descending</MenuItem>
        </Select>
      </FormControl>

      <Button
        variant="contained"
        startIcon={<ExportIcon />}
        onClick={onExport}
        disabled={selectedIds.size === 0}
      >
        Export ({selectedIds.size})
      </Button>

      <Button
        variant="contained"
        color="error"
        startIcon={<DeleteIcon />}
        onClick={onBulkDelete}
        disabled={selectedIds.size === 0}
      >
        Delete ({selectedIds.size})
      </Button>

      {selectedIds.size > 0 ? (
        <Button size="small" variant="outlined" onClick={onSelectNone}>
          Clear Selection
        </Button>
      ) : sequences.length > 0 ? (
        <Button size="small" variant="outlined" onClick={onSelectAll}>
          Select All
        </Button>
      ) : null}
    </Stack>
  </Box>
));

export const DataLibraryWindowContent: React.FC = () => {
  const {
    sequences,
    selectedSequence,
    selectedStats,
    searchQuery,
    selectedTags,
    allTags,
    sortBy,
    sortOrder,
    error,
    selectedIds,
    currentPage,
    pageSize,
    totalCount,
    totalPages,
    hasNextPage,
    hasPrevPage,
    setCurrentPage,
    setPageSize,
    setSelectedSequence,
    setSearchQuery,
    setSortBy,
    setSortOrder,
    setError,
    handlePinSequence,
    handleDeleteSequence,
    handleBulkDelete,
    handleDuplicateSequence,
    handleUpdateSequence,
    handleToggleTagFilter,
    handleToggleSelection,
    handleSelectAll,
    handleSelectNone,
  } = useDataLibrary();

  // Dialog states
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [exportFormat, setExportFormat] = useState<'csv' | 'json'>('csv');
  const [bulkDeleteDialogOpen, setBulkDeleteDialogOpen] = useState(false);

  // Loading states for delete operations
  const [isDeleting, setIsDeleting] = useState(false);
  const [isBulkDeleting, setIsBulkDeleting] = useState(false);

  // Edit form state
  const [editName, setEditName] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [editUnit, setEditUnit] = useState('');
  const [editTags, setEditTags] = useState<string[]>([]);
  const [newTagInput, setNewTagInput] = useState('');

  // Simplified handlers using the custom hook
  const handleDelete = async () => {
    if (!selectedSequence) {return;}

    setIsDeleting(true);
    try {
      await handleDeleteSequence(selectedSequence.id);
      setDeleteDialogOpen(false);
    } catch (err) {
      console.error('Failed to delete sequence:', err);
      // Error is already handled by the hook, just keep dialog open
    } finally {
      setIsDeleting(false);
    }
  };

  const handleBulkDeleteConfirm = async () => {
    setIsBulkDeleting(true);
    try {
      await handleBulkDelete(selectedIds);
      setBulkDeleteDialogOpen(false);
    } catch (err) {
      console.error('Failed to bulk delete sequences:', err);
      // Error is already handled by the hook, just keep dialog open
    } finally {
      setIsBulkDeleting(false);
    }
  };

  const handleDuplicate = () => {
    if (!selectedSequence) {return;}
    void handleDuplicateSequence(selectedSequence.id, selectedSequence.name);
  };

  const handleOpenEditDialog = () => {
    if (!selectedSequence) {return;}
    setEditName(selectedSequence.name);
    setEditDescription(selectedSequence.description);
    setEditUnit(selectedSequence.unit);
    setEditTags(selectedSequence.tags);
    setEditDialogOpen(true);
  };

  const handleSaveEdit = async () => {
    if (!selectedSequence) {return;}

    const request: UpdateSequenceRequest = {
      id: selectedSequence.id,
    };

    // Only add properties that have changed
    if (editName !== selectedSequence.name) {
      request.name = editName;
    }
    if (editDescription !== selectedSequence.description) {
      request.description = editDescription;
    }
    if (editUnit !== selectedSequence.unit) {
      request.unit = editUnit;
    }

    // Compare tags using Sets for order-insensitive comparison
    const originalTagsSet = new Set(selectedSequence.tags);
    const editTagsSet = new Set(editTags);
    const tagsChanged = originalTagsSet.size !== editTagsSet.size ||
      [...originalTagsSet].some(tag => !editTagsSet.has(tag));

    if (tagsChanged) {
      request.tags = editTags;
    }

    try {
      await handleUpdateSequence(request);
      setEditDialogOpen(false);
    } catch (err) {
      console.error('Failed to update sequence:', err);
      setError(`Failed to update sequence: ${err instanceof Error ? err.message : String(err)}`);
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
        handleSelectNone();
        setError(null);
      }
    } catch (err) {
      console.error('Export failed:', err);
      setError(`Export failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  return (
    <Box sx={{ width: '100%', height: '100vh', display: 'flex', flexDirection: 'column', bgcolor: 'background.default' }}>
      <CustomTitleBar title="Data Library" />

      {/* Toolbar */}
      <DataLibraryToolbar
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
        sortBy={sortBy}
        setSortBy={setSortBy}
        sortOrder={sortOrder}
        setSortOrder={setSortOrder}
        selectedIds={selectedIds}
        sequences={sequences}
        onExport={() => setExportDialogOpen(true)}
        onBulkDelete={() => setBulkDeleteDialogOpen(true)}
        onSelectAll={handleSelectAll}
        onSelectNone={handleSelectNone}
      />

      {/* Tag filters */}
      {allTags.length > 0 && (
        <Box sx={{ px: 2, pb: 1 }}>
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

      {/* Divider line */}
      {allTags.length > 0 && <Divider />}

      {/* Pagination controls */}
      {totalCount > 0 && (
        <Box sx={{ px: 2, py: 1, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <Typography variant="body2" color="text.secondary">
            Showing {sequences.length} of {totalCount} sequences
          </Typography>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <FormControl size="small" sx={{ minWidth: 80 }}>
              <Select
                value={pageSize}
                onChange={(e) => setPageSize(Number(e.target.value))}
              >
                <MenuItem value={25}>25</MenuItem>
                <MenuItem value={50}>50</MenuItem>
                <MenuItem value={100}>100</MenuItem>
              </Select>
            </FormControl>
            <Typography variant="body2" color="text.secondary">per page</Typography>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
              <IconButton
                size="small"
                onClick={() => setCurrentPage(0)}
                disabled={!hasPrevPage}
              >
                <FirstPageIcon />
              </IconButton>
              <IconButton
                size="small"
                onClick={() => setCurrentPage(currentPage - 1)}
                disabled={!hasPrevPage}
              >
                <ChevronLeftIcon />
              </IconButton>
              <Typography variant="body2" sx={{ mx: 1 }}>
                Page {currentPage + 1} of {totalPages}
              </Typography>
              <IconButton
                size="small"
                onClick={() => setCurrentPage(currentPage + 1)}
                disabled={!hasNextPage}
              >
                <ChevronRightIcon />
              </IconButton>
              <IconButton
                size="small"
                onClick={() => setCurrentPage(totalPages - 1)}
                disabled={!hasNextPage}
              >
                <LastPageIcon />
              </IconButton>
            </Box>
          </Box>
        </Box>
      )}

      {/* Divider line */}
      {totalCount > 0 && <Divider />}

      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ m: 2 }}>
          {error}
        </Alert>
      )}

      {/* Main content */}
      <Box sx={{ display: 'flex', flexGrow: 1, overflow: 'hidden' }}>
        <SequenceList
          sequences={sequences}
          selectedSequence={selectedSequence}
          selectedIds={selectedIds}
          onSequenceSelect={setSelectedSequence}
          onToggleSelection={handleToggleSelection}
        />

        <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          <SequenceDetails
            sequence={selectedSequence}
            statistics={selectedStats}
            onPin={handlePinSequence}
            onEdit={handleOpenEditDialog}
            onDuplicate={handleDuplicate}
            onDelete={() => setDeleteDialogOpen(true)}
          />
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
                onKeyDown={(e) => e.key === 'Enter' && handleAddTag()}
              />
              <Button onClick={handleAddTag}>Add</Button>
            </Stack>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setEditDialogOpen(false)}>Cancel</Button>
          <Button onClick={() => void handleSaveEdit()} variant="contained">Save</Button>
        </DialogActions>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog open={deleteDialogOpen} onClose={() => !isDeleting && setDeleteDialogOpen(false)}>
        <DialogTitle>Confirm Delete</DialogTitle>
        <DialogContent>
          <Typography>
            Are you sure you want to delete "{selectedSequence?.name}"? This action cannot be undone.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteDialogOpen(false)} disabled={isDeleting}>Cancel</Button>
          <Button onClick={() => void handleDelete()} color="error" variant="contained" disabled={isDeleting}>
            {isDeleting ? 'Deleting...' : 'Delete'}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Bulk Delete Confirmation Dialog */}
      <Dialog open={bulkDeleteDialogOpen} onClose={() => !isBulkDeleting && setBulkDeleteDialogOpen(false)}>
        <DialogTitle>Confirm Bulk Delete</DialogTitle>
        <DialogContent>
          <Typography>
            Are you sure you want to delete {selectedIds.size} sequence{selectedIds.size > 1 ? 's' : ''}? This action cannot be undone.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setBulkDeleteDialogOpen(false)} disabled={isBulkDeleting}>Cancel</Button>
          <Button onClick={() => void handleBulkDeleteConfirm()} color="error" variant="contained" disabled={isBulkDeleting}>
            {isBulkDeleting ? 'Deleting...' : `Delete ${selectedIds.size} Sequence${selectedIds.size > 1 ? 's' : ''}`}
          </Button>
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
              onChange={(e: SelectChangeEvent) => setExportFormat(e.target.value as 'csv' | 'json')}
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
          <Button onClick={() => void handleExport()} variant="contained" startIcon={<ExportIcon />}>
            Export
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

const theme = createAnafisTheme();

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <DataLibraryWindowContent />
    </ThemeProvider>
  </React.StrictMode>
);
