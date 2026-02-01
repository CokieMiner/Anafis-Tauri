import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  DataSequence,
  SequenceStatistics,
  SearchRequest,
  SequenceListResponse,
  UpdateSequenceRequest,
} from '@/core/types/dataLibrary';
import { type ErrorResponse, isErrorResponse, getErrorMessage } from '@/core/types/error';

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

  // Pagination state
  const [currentPage, setCurrentPage] = useState(0);
  const [pageSize, setPageSize] = useState(50);
  const [totalCount, setTotalCount] = useState(0);
  const [totalPages, setTotalPages] = useState(0);
  const [hasNextPage, setHasNextPage] = useState(false);
  const [hasPrevPage, setHasPrevPage] = useState(false);

  // Selection state for multi-select export
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  // Debounced search query for performance
  const debouncedSearchQuery = useDebounce(searchQuery, 300);

  // Memoized search request to prevent unnecessary API calls
  const searchRequest = useMemo((): SearchRequest => ({
    sort_by: sortBy,
    sort_order: sortOrder,
    page: currentPage,
    page_size: pageSize,
    ...(debouncedSearchQuery && { query: debouncedSearchQuery }),
    ...(selectedTags.length > 0 && { tags: selectedTags }),
  }), [debouncedSearchQuery, selectedTags, sortBy, sortOrder, currentPage, pageSize]);

  const loadSequences = useCallback(async () => {
    try {
      const response = await invoke<SequenceListResponse | ErrorResponse>('get_sequences', { search: searchRequest });
      if (isErrorResponse(response)) {
        setError(getErrorMessage(response));
        return;
      }
      setSequences(response.sequences);
      setTotalCount(response.total_count);
      setTotalPages(response.total_pages);
      setHasNextPage(response.has_next);
      setHasPrevPage(response.has_prev);
      setError(null);
    } catch (err) {
      console.error('Failed to load sequences:', err);
      setError(`Failed to load sequences: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [searchRequest]);

  const loadAllTags = useCallback(async () => {
    try {
      const response = await invoke<string[] | ErrorResponse>('get_all_tags');
      if (isErrorResponse(response)) {
        setError(getErrorMessage(response));
        return;
      }
      setAllTags(response);
      setError(null);
    } catch (err) {
      console.error('Failed to load tags:', err);
      setError(`Failed to load tags: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, []);

  const loadStatistics = useCallback(async (id: string) => {
    try {
      const response = await invoke<SequenceStatistics | null | ErrorResponse>('get_sequence_stats', { id });
      if (isErrorResponse(response)) {
        setError(getErrorMessage(response));
        return;
      }
      setSelectedStats(response);
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
      const response = await invoke<void | ErrorResponse>('pin_sequence', { id, isPinned: !isPinned });
      if (isErrorResponse(response)) {
        setError(getErrorMessage(response));
        return;
      }
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
      const response = await invoke<void | ErrorResponse>('delete_sequence', { id });
      if (isErrorResponse(response)) {
        setError(getErrorMessage(response));
        return;
      }
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
        idsArray.map(id => invoke<void | ErrorResponse>('delete_sequence', { id }))
      );

      // Separate successful and failed deletions
      const successfulIds: string[] = [];
      const failedDeletions: Array<{ id: string; error: string }> = [];

      deleteResults.forEach((result, index) => {
        const id = idsArray[index]!;
        if (result.status === 'fulfilled') {
          const response = result.value;
          if (isErrorResponse(response)) {
            failedDeletions.push({ id, error: getErrorMessage(response) });
          } else {
            successfulIds.push(id);
          }
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
      const response = await invoke<string | ErrorResponse>('duplicate_sequence', { id, newName });
      if (isErrorResponse(response)) {
        setError(getErrorMessage(response));
        return;
      }
      await loadSequences();
    } catch (err) {
      console.error('Failed to duplicate sequence:', err);
      setError(`Failed to duplicate sequence: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [loadSequences]);

  const handleUpdateSequence = useCallback(async (request: UpdateSequenceRequest) => {
    try {
      const response = await invoke<void | ErrorResponse>('update_sequence', { request });
      if (isErrorResponse(response)) {
        setError(getErrorMessage(response));
        return;
      }
      await loadSequences();

      // Update selected sequence
      if (selectedSequence?.id === request.id) {
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

    // Pagination state
    currentPage,
    pageSize,
    totalCount,
    totalPages,
    hasNextPage,
    hasPrevPage,

    // Pagination setters
    setCurrentPage,
    setPageSize,

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