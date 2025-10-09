// Data Library Models
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a data sequence stored in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSequence {
    pub id: String,           // UUID
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub unit: String,
    pub source: String,       // e.g., "Sheet1, A1:A100"
    pub data: Vec<f64>,       // Main data values
    pub uncertainties: Option<Vec<f64>>,  // Optional uncertainty values
    pub is_pinned: bool,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Statistics for a data sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceStatistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub has_uncertainties: bool,
}

/// Request to save a new sequence
#[derive(Debug, Clone, Deserialize)]
pub struct SaveSequenceRequest {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub unit: String,
    pub source: String,
    pub data: Vec<f64>,
    pub uncertainties: Option<Vec<f64>>,
    pub is_pinned: bool,
}

/// Request to update an existing sequence
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSequenceRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub unit: Option<String>,
    pub is_pinned: Option<bool>,
}

/// Request to search sequences
#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    pub query: Option<String>,        // Full-text search
    pub tags: Option<Vec<String>>,    // Filter by tags
    pub source: Option<String>,       // Filter by source
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Name,
    DateCreated,
    DateModified,
    Size,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: None,
            tags: None,
            source: None,
            sort_by: SortBy::DateModified,
            sort_order: SortOrder::Descending,
        }
    }
}

/// Response containing sequences and metadata
#[derive(Debug, Clone, Serialize)]
pub struct SequenceListResponse {
    pub sequences: Vec<DataSequence>,
    pub total_count: usize,
    pub pinned_count: usize,
}
