import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  DataSequence,
  SequenceStatistics,
  SearchRequest,
  SequenceListResponse,
  UpdateSequenceRequest,
} from '../types/dataLibrary';

// Debounce utility for search operations
const useDebounce = <T>(value: T, delay: number): T => {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
};

export const useDataLibrary = () => {
  // State management
  const [sequences, setSequences] = useState<DataSequence[]>([]);
  const [selectedSequence, setSelectedSequence] = useState<DataSequence | null>(null);
  const [selectedStats, setSelectedStats] = useState<SequenceStatistics | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [allTags, setAllTags] = useState<string[]>([]);
  const [sortBy, setSortBy] = useState<'name' | 'date_created' | 'date_modified' | 'size'>('date_modified');
  const [sortOrder, setSortOrder] = useState<'ascending' | 'descending'>('descending');
  const [error, setError] = useState<string | null>(null);

  // Selection state for multi-select export
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  // Debounced search query for performance
  const debouncedSearchQuery = useDebounce(searchQuery, 300);

  // Memoized search request to prevent unnecessary API calls
  const searchRequest = useMemo((): SearchRequest => ({
    sort_by: sortBy,
    sort_order: sortOrder,
    ...(debouncedSearchQuery && { query: debouncedSearchQuery }),
    ...(selectedTags.length > 0 && { tags: selectedTags }),
  }), [debouncedSearchQuery, selectedTags, sortBy, sortOrder]);

  const loadSequences = useCallback(async () => {
    try {
      const response = await invoke<SequenceListResponse>('get_sequences', { search: searchRequest });
      setSequences(response.sequences);
      setError(null);
    } catch (err) {
      console.error('Failed to load sequences:', err);
      setError(`Failed to load sequences: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [searchRequest]);

  const loadAllTags = useCallback(async () => {
    try {
      const tags = await invoke<string[]>('get_all_tags');
      setAllTags(tags);
      setError(null);
    } catch (err) {
      console.error('Failed to load tags:', err);
      setError(`Failed to load tags: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, []);

  const loadStatistics = useCallback(async (id: string) => {
    try {
      const stats = await invoke<SequenceStatistics | null>('get_sequence_stats', { id });
      setSelectedStats(stats);
      setError(null);
    } catch (err) {
      console.error('Failed to load statistics:', err);
      setError(`Failed to load statistics: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, []);

  // Load data on mount
  useEffect(() => {
    const load = async () => {
      await loadSequences();
      await loadAllTags();
    };
    void load();
  }, [loadSequences, loadAllTags]);

  // Load statistics when a sequence is selected
  useEffect(() => {
    const load = async () => {
      if (selectedSequence) {
        await loadStatistics(selectedSequence.id);
      }
    };
    void load();
  }, [selectedSequence, loadStatistics]);

  const handlePinSequence = useCallback(async (id: string, isPinned: boolean) => {
    try {
      await invoke('pin_sequence', { id, isPinned: !isPinned });
      await loadSequences();
      if (selectedSequence?.id === id) {
        setSelectedSequence({ ...selectedSequence, is_pinned: !isPinned });
      }
    } catch (err) {
      console.error('Failed to pin sequence:', err);
      setError(`Failed to pin sequence: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [selectedSequence, loadSequences]);

  const handleDeleteSequence = useCallback(async (id: string) => {
    try {
      await invoke('delete_sequence', { id });
      if (selectedSequence?.id === id) {
        setSelectedSequence(null);
        setSelectedStats(null);
      }
      await loadSequences();
    } catch (err) {
      console.error('Failed to delete sequence:', err);
      setError(`Failed to delete sequence: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [selectedSequence, loadSequences]);

  const handleBulkDelete = useCallback(async (ids: Set<string>) => {
    if (ids.size === 0) {return;}

    try {
      const idsArray = Array.from(ids);

      // Use Promise.allSettled to handle partial failures gracefully
      const deleteResults = await Promise.allSettled(
        idsArray.map(id => invoke('delete_sequence', { id }))
      );

      // Separate successful and failed deletions
      const successfulIds: string[] = [];
      const failedDeletions: Array<{ id: string; error: string }> = [];

      deleteResults.forEach((result, index) => {
        const id = idsArray[index]!;
        if (result.status === 'fulfilled') {
          successfulIds.push(id);
        } else {
          const error = result.reason instanceof Error ? result.reason.message : String(result.reason);
          failedDeletions.push({ id, error });
          console.error(`Failed to delete sequence ${id}:`, error);
        }
      });

      // Update selected IDs to only remove successfully deleted items
      if (successfulIds.length > 0) {
        setSelectedIds(prev => {
          const newSelected = new Set(prev);
          successfulIds.forEach(id => newSelected.delete(id));
          return newSelected;
        });

        // Clear selected sequence/stats only if they were successfully deleted
        if (selectedSequence && successfulIds.includes(selectedSequence.id)) {
          setSelectedSequence(null);
          setSelectedStats(null);
        }
      }

      // Always reload sequences to reflect current state
      await loadSequences();

      // Handle errors - show user-friendly message for failed deletions
      if (failedDeletions.length > 0) {
        const errorMessage = failedDeletions.length === 1
          ? `Failed to delete sequence: ${failedDeletions[0]!.error}`
          : `Failed to delete ${failedDeletions.length} of ${ids.size} sequences. Check console for details.`;

        setError(errorMessage);

        // Log detailed per-ID errors for debugging
        console.error('Bulk delete failures:', failedDeletions);
      } else if (successfulIds.length > 0) {
        // Clear any previous errors if all deletions succeeded
        setError(null);
      }
    } catch (err) {
      // This should rarely happen since we're using allSettled, but handle unexpected errors
      console.error('Unexpected error during bulk delete:', err);
      setError(`Unexpected error during bulk delete: ${err instanceof Error ? err.message : String(err)}`);
      // Still reload sequences to be safe
      await loadSequences();
    }
  }, [selectedSequence, loadSequences]);

  const handleDuplicateSequence = useCallback(async (id: string, name: string) => {
    try {
      const newName = `${name} (copy)`;
      await invoke('duplicate_sequence', { id, newName });
      await loadSequences();
    } catch (err) {
      console.error('Failed to duplicate sequence:', err);
      setError(`Failed to duplicate sequence: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [loadSequences]);

  const handleUpdateSequence = useCallback(async (request: UpdateSequenceRequest) => {
    try {
      await invoke('update_sequence', { request });
      await loadSequences();

      // Update selected sequence
      if (selectedSequence && selectedSequence.id === request.id) {
        const updatedSequence = { ...selectedSequence };
        if (request.name !== undefined) {updatedSequence.name = request.name;}
        if (request.description !== undefined) {updatedSequence.description = request.description;}
        if (request.unit !== undefined) {updatedSequence.unit = request.unit;}
        if (request.tags !== undefined) {updatedSequence.tags = request.tags;}
        setSelectedSequence(updatedSequence);
      }
    } catch (err) {
      console.error('Failed to update sequence:', err);
      setError(`Failed to update sequence: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [selectedSequence, loadSequences]);

  const handleToggleTagFilter = useCallback((tag: string) => {
    if (selectedTags.includes(tag)) {
      setSelectedTags(selectedTags.filter(t => t !== tag));
    } else {
      setSelectedTags([...selectedTags, tag]);
    }
  }, [selectedTags]);

  // Selection handlers
  const handleToggleSelection = useCallback((id: string) => {
    const newSelected = new Set(selectedIds);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    setSelectedIds(newSelected);
  }, [selectedIds]);

  const handleSelectAll = useCallback(() => {
    setSelectedIds(new Set(sequences.map(s => s.id)));
  }, [sequences]);

  const handleSelectNone = useCallback(() => {
    setSelectedIds(new Set());
  }, []);

  return {
    // State
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

    // Setters
    setSelectedSequence,
    setSearchQuery,
    setSortBy,
    setSortOrder,
    setError,

    // Actions
    handlePinSequence,
    handleDeleteSequence,
    handleBulkDelete,
    handleDuplicateSequence,
    handleUpdateSequence,
    handleToggleTagFilter,
    handleToggleSelection,
    handleSelectAll,
    handleSelectNone,

    // Loaders
    loadSequences,
    loadAllTags,
    loadStatistics,
  };
};