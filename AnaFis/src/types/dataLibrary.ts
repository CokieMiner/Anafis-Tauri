// Data Library TypeScript Types
// Matches Rust backend structures

export interface DataSequence {
  id: string;
  name: string;
  description: string;
  tags: string[];
  unit: string;
  source: string;  // e.g., "Sheet1, A1:A100"
  data: number[];
  uncertainties?: number[];
  is_pinned: boolean;
  created_at: string;  // ISO 8601 format
  modified_at: string; // ISO 8601 format
}

export interface SequenceStatistics {
  count: number;
  mean: number;
  std_dev: number;
  min: number;
  max: number;
  median: number;
  has_uncertainties: boolean;
}

export interface SaveSequenceRequest {
  name: string;
  description: string;
  tags: string[];
  unit: string;
  source: string;
  data: number[];
  uncertainties?: number[];
  is_pinned: boolean;
}

export interface UpdateSequenceRequest {
  id: string;
  name?: string;
  description?: string;
  tags?: string[];
  unit?: string;
  is_pinned?: boolean;
}

export type SortBy = 'name' | 'date_created' | 'date_modified' | 'size';
export type SortOrder = 'ascending' | 'descending';

export interface SearchRequest {
  query?: string;
  tags?: string[];
  source?: string;
  sort_by: SortBy;
  sort_order: SortOrder;
}

export interface SequenceListResponse {
  sequences: DataSequence[];
  total_count: number;
  pinned_count: number;
}
