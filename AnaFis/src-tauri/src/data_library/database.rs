//! Data Library Database Operations
//!
//! This module handles all database operations for the data library functionality.
//! It provides CRUD operations for storing and retrieving data sequences with
//! associated metadata and statistics.

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Result as SqliteResult, Row, params};
use std::sync::Mutex;
use uuid::Uuid;
use std::fmt::Write;
use super::models::{SearchRequest, SortBy, SortOrder, DataSequence, SaveSequenceRequest, SequenceListResponse, UpdateSequenceRequest, BatchImportRequest, BatchImportResponse, BatchImportError};

pub struct DataLibraryDatabase {
    conn: Mutex<Connection>,
}

impl DataLibraryDatabase {
    fn build_fts_query(search: &str) -> Option<String> {
        let terms: Vec<String> = search
            .split_whitespace()
            .map(|token| {
                token
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>()
            })
            .filter(|token| !token.is_empty())
            .map(|token| format!("{token}*"))
            .collect();

        if terms.is_empty() {
            None
        } else {
            Some(terms.join(" AND "))
        }
    }

    fn build_where_clause(search: &SearchRequest) -> (String, Vec<String>) {
        let mut params = Vec::new();
        let mut where_clauses = Vec::new();

        if let Some(query_text) = &search.query {
            let query_text = query_text.trim();
            if !query_text.is_empty() {
                if let Some(fts_query) = Self::build_fts_query(query_text) {
                    params.push(fts_query);
                    let fts_idx = params.len();

                    params.push(format!("%{query_text}%"));
                    let like_idx = params.len();

                    where_clauses.push(format!(
                        "(id IN (SELECT id FROM sequences_fts WHERE sequences_fts MATCH ?{fts_idx}) OR name LIKE ?{like_idx})"
                    ));
                } else {
                    params.push(format!("%{query_text}%"));
                    where_clauses.push(format!("name LIKE ?{}", params.len()));
                }
            }
        }

        if let Some(tags) = &search.tags
            && !tags.is_empty()
        {
            let tags_conditions: Vec<String> = tags
                .iter()
                .map(|tag| {
                    params.push(format!("%{tag}%"));
                    format!("tags LIKE ?{}", params.len())
                })
                .collect();
            where_clauses.push(format!("({})", tags_conditions.join(" AND ")));
        }

        if let Some(source) = &search.source
            && !source.is_empty()
        {
            params.push(source.clone());
            where_clauses.push(format!("source = ?{}", params.len()));
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_clauses.join(" AND "))
        };

        (where_sql, params)
    }

    fn order_by_clause(search: &SearchRequest) -> String {
        let order_col = match search.sort_by {
            SortBy::Name => "name",
            SortBy::DateCreated => "created_at",
            SortBy::DateModified => "modified_at",
            SortBy::Size => "length(data)",
        };
        let order_dir = match search.sort_order {
            SortOrder::Ascending => "ASC",
            SortOrder::Descending => "DESC",
        };
        format!(" ORDER BY is_pinned DESC, {order_col} {order_dir}")
    }

    fn query_count(
        &self,
        base_sql: &str,
        where_sql: &str,
        params: &[String],
    ) -> SqliteResult<usize> {
        let sql = format!("{base_sql}{where_sql}");
        let count: i64 = if params.is_empty() {
            self.conn
                .lock()
                .unwrap()
                .prepare(&sql)?
                .query_row([], |row| row.get(0))?
        } else {
            let param_refs: Vec<&dyn rusqlite::ToSql> =
                params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
            self.conn
                .lock()
                .unwrap()
                .prepare(&sql)?
                .query_row(&param_refs[..], |row| row.get(0))?
        };

        usize::try_from(count).map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Integer,
                "Count value out of range for usize".into(),
            )
        })
    }

    fn query_sequences_with_limit(
        &self,
        search: &SearchRequest,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> SqliteResult<Vec<DataSequence>> {
        let (where_sql, mut params) = Self::build_where_clause(search);
        let mut query = "SELECT id, name, description, tags, unit, source, data, uncertainties, is_pinned, created_at, modified_at FROM sequences".to_string();
        query.push_str(&where_sql);
        query.push_str(&Self::order_by_clause(search));

        if let Some(limit) = limit {
            params.push(limit.to_string());
            let limit_idx = params.len();
            let _ = write!(query, " LIMIT ?{limit_idx}");
        }
        if let Some(offset) = offset {
            params.push(offset.to_string());
            let offset_idx = params.len();
            let _ = write!(query, " OFFSET ?{offset_idx}");
        }

        let sequences = if params.is_empty() {
            self.conn
                .lock()
                .unwrap()
                .prepare(&query)?
                .query_map([], Self::row_to_sequence)?
                .collect::<SqliteResult<Vec<_>>>()?
        } else {
            let param_refs: Vec<&dyn rusqlite::ToSql> =
                params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
            self.conn
                .lock()
                .unwrap()
                .prepare(&query)?
                .query_map(&param_refs[..], Self::row_to_sequence)?
                .collect::<SqliteResult<Vec<_>>>()?
        };

        Ok(sequences)
    }

    fn row_to_sequence(row: &Row<'_>) -> SqliteResult<DataSequence> {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let description: String = row.get(2)?;
        let tags_json: String = row.get(3)?;
        let unit: String = row.get(4)?;
        let source: String = row.get(5)?;
        let data_json: String = row.get(6)?;
        let uncertainties_json: Option<String> = row.get(7)?;
        let is_pinned: i32 = row.get(8)?;
        let created_at_str: String = row.get(9)?;
        let modified_at_str: String = row.get(10)?;

        let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
        })?;
        let data: Vec<f64> = serde_json::from_str(&data_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e))
        })?;
        let uncertainties: Option<Vec<f64>> = uncertainties_json
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    9,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);
        let modified_at = chrono::DateTime::parse_from_rfc3339(&modified_at_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        Ok(DataSequence {
            id,
            name,
            description,
            tags,
            unit,
            source,
            data,
            uncertainties,
            is_pinned: is_pinned != 0,
            created_at,
            modified_at,
        })
    }

    /// Initialize the database and create tables if they don't exist
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;

        // Create sequences table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sequences (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                tags TEXT,  -- JSON array
                unit TEXT,
                source TEXT,
                data BLOB NOT NULL,  -- Serialized Vec<f64>
                uncertainties BLOB,  -- Optional serialized Vec<f64>
                is_pinned INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create FTS5 virtual table for full-text search
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS sequences_fts USING fts5(
                id UNINDEXED,
                name,
                description,
                tags,
                source
            )",
            [],
        )?;

        // Create indexes for better query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sequences_name ON sequences(name)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sequences_pinned ON sequences(is_pinned)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sequences_created ON sequences(created_at)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sequences_modified ON sequences(modified_at)",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Save a new sequence to the database
    pub fn save_sequence(&self, request: &SaveSequenceRequest) -> SqliteResult<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Serialize data and uncertainties to JSON
        let data_json = serde_json::to_string(&request.data)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let uncertainties_json = request
            .uncertainties
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let tags_json = serde_json::to_string(&request.tags)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        {
            let conn = self.conn.lock().unwrap();

            // Insert into main table
            conn.execute(
                "INSERT INTO sequences (id, name, description, tags, unit, source, data, uncertainties, is_pinned, created_at, modified_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    id,
                    request.name,
                    request.description,
                    tags_json,
                    request.unit,
                    request.source,
                    data_json,
                    uncertainties_json,
                    i32::from(request.is_pinned),
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                ],
            )?;

            // Insert into FTS table
            conn.execute(
                "INSERT INTO sequences_fts (id, name, description, tags, source)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    id,
                    request.name,
                    request.description,
                    request.tags.join(" "),
                    request.source,
                ],
            )?;
        }

        Ok(id)
    }

    /// Get paginated sequences with optional filtering and sorting
    pub fn get_sequences_paginated(
        &self,
        search: &SearchRequest,
    ) -> SqliteResult<SequenceListResponse> {
        let page_size = search.page_size.unwrap_or(50).max(1);
        let requested_page = search.page.unwrap_or(0);
        let (where_sql, where_params) = Self::build_where_clause(search);

        let total_count =
            self.query_count("SELECT COUNT(*) FROM sequences", &where_sql, &where_params)?;
        let total_pages = total_count.div_ceil(page_size);
        let page = if total_pages == 0 {
            0
        } else {
            requested_page.min(total_pages - 1)
        };
        let offset = page.saturating_mul(page_size);

        let pinned_where_sql = if where_sql.is_empty() {
            " WHERE is_pinned = 1".to_string()
        } else {
            where_sql.replacen(" WHERE ", " WHERE is_pinned = 1 AND ", 1)
        };
        let pinned_count = self.query_count(
            "SELECT COUNT(*) FROM sequences",
            &pinned_where_sql,
            &where_params,
        )?;

        let sequences = self.query_sequences_with_limit(search, Some(page_size), Some(offset))?;

        Ok(SequenceListResponse {
            version: crate::error::API_VERSION.to_string(),
            sequences,
            total_count,
            pinned_count,
            page,
            page_size,
            total_pages,
            has_next: page + 1 < total_pages,
            has_prev: page > 0,
        })
    }

    pub fn get_sequence(&self, id: &str) -> SqliteResult<Option<DataSequence>> {
        self.conn
            .lock()
            .unwrap()
            .prepare(
                "SELECT id, name, description, tags, unit, source, data, uncertainties, is_pinned, created_at, modified_at
                 FROM sequences
                 WHERE id = ?1",
            )?
            .query_row(params![id], Self::row_to_sequence)
            .optional()
    }

    pub fn update_sequence(&self, request: &UpdateSequenceRequest) -> SqliteResult<()> {
        let now = Utc::now();

        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(name) = &request.name {
            updates.push("name = ?");
            params.push(Box::new(name.clone()));
        }
        if let Some(description) = &request.description {
            updates.push("description = ?");
            params.push(Box::new(description.clone()));
        }
        if let Some(tags) = &request.tags {
            updates.push("tags = ?");
            let tags_json = serde_json::to_string(tags)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            params.push(Box::new(tags_json));
        }
        if let Some(unit) = &request.unit {
            updates.push("unit = ?");
            params.push(Box::new(unit.clone()));
        }
        if let Some(is_pinned) = request.is_pinned {
            updates.push("is_pinned = ?");
            params.push(Box::new(i32::from(is_pinned)));
        }

        updates.push("modified_at = ?");
        params.push(Box::new(now.to_rfc3339()));

        params.push(Box::new(request.id.clone()));

        let query = format!("UPDATE sequences SET {} WHERE id = ?", updates.join(", "));

        {
            let conn = self.conn.lock().unwrap();
            conn.execute(&query, rusqlite::params_from_iter(params.iter()))?;

            // Update FTS table if name, description, or tags changed
            if request.name.is_some() || request.description.is_some() || request.tags.is_some() {
                // Get current values for FTS update
                let mut stmt =
                    conn.prepare("SELECT name, description, tags, source FROM sequences WHERE id = ?")?;
                let mut rows = stmt.query(params![request.id])?;

                if let Some(row) = rows.next()? {
                    let name: String = row.get(0)?;
                    let description: String = row.get(1)?;
                    let tags_json: String = row.get(2)?;
                    let source: String = row.get(3)?;
                    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                    conn.execute(
                        "UPDATE sequences_fts SET name = ?1, description = ?2, tags = ?3, source = ?4 WHERE id = ?5",
                        params![name, description, tags.join(" "), source, request.id],
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Delete a sequence
    pub fn delete_sequence(&self, id: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sequences WHERE id = ?", params![id])?;
        conn.execute("DELETE FROM sequences_fts WHERE id = ?", params![id])?;
        drop(conn);
        Ok(())
    }

    /// Get all unique tags
    pub fn get_all_tags(&self) -> SqliteResult<Vec<String>> {
        let mut all_tags = std::collections::HashSet::new();
        let tags_json_list: Vec<String> = self.conn
            .lock()
            .unwrap()
            .prepare("SELECT DISTINCT tags FROM sequences")?
            .query_map([], |row| row.get(0))?
            .collect::<SqliteResult<Vec<String>>>()?;

        for tags_json in tags_json_list {
            let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            all_tags.extend(tags);
        }

        let mut tags: Vec<String> = all_tags.into_iter().collect();
        tags.sort();
        Ok(tags)
    }

    /// Duplicate a sequence
    pub fn duplicate_sequence(&self, id: &str, new_name: &str) -> SqliteResult<String> {
        let sequence = self
            .get_sequence(id)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

        let request = SaveSequenceRequest {
            name: new_name.to_string(),
            description: format!("{} (copy)", sequence.description),
            tags: sequence.tags,
            unit: sequence.unit,
            source: sequence.source,
            data: sequence.data,
            uncertainties: sequence.uncertainties,
            is_pinned: false,
        };

        self.save_sequence(&request)
    }

    /// Export sequences to CSV format
    /// Format: Each sequence becomes a column pair (name, `name_uncertainty`)
    /// First row: column headers
    /// Subsequent rows: data values
    pub fn export_to_csv(&self, sequence_ids: &[String], file_path: &str) -> SqliteResult<()> {
        use std::fs::File;
        use std::io::Write;

        // Fetch all sequences
        let mut sequences = Vec::new();
        for id in sequence_ids {
            if let Some(seq) = self.get_sequence(id)? {
                sequences.push(seq);
            }
        }

        if sequences.is_empty() {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        // Find the maximum length
        let max_len = sequences.iter().map(|s| s.data.len()).max().unwrap_or(0);

        let mut file = File::create(file_path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // Write header row with embedded metadata
        // Format: [name,unit,description,tags]
        let mut header = Vec::new();
        for seq in &sequences {
            // Escape commas and quotes in metadata fields
            let name = seq.name.replace(',', ";").replace('"', "'");
            let unit = seq.unit.replace(',', ";").replace('"', "'");
            let description = seq.description.replace(',', ";").replace('"', "'");
            let tags = seq.tags.join("|").replace(',', ";").replace('"', "'");

            // Data column with full metadata
            header.push(format!("[{name},{unit},{description},{tags}]"));

            // Uncertainty column if present
            if seq.uncertainties.is_some() {
                header.push(format!(
                    "[{name}_uncertainty,{unit},Uncertainty values for {name},{tags}]"
                ));
            }
        }
        writeln!(file, "{}", header.join(","))
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // Write data rows
        for i in 0..max_len {
            let mut row = Vec::new();
            for seq in &sequences {
                if i < seq.data.len() {
                    row.push(seq.data[i].to_string());
                    if let Some(ref uncertainties) = seq.uncertainties {
                        if i < uncertainties.len() {
                            row.push(uncertainties[i].to_string());
                        } else {
                            row.push(String::new());
                        }
                    }
                } else {
                    row.push(String::new());
                    if seq.uncertainties.is_some() {
                        row.push(String::new());
                    }
                }
            }
            writeln!(file, "{}", row.join(","))
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        }

        Ok(())
    }

    /// Batch import multiple sequences
    /// Returns results for each sequence, continuing on errors
    pub fn batch_import_sequences(
        &self,
        request: BatchImportRequest,
    ) -> BatchImportResponse {
        let mut successful_imports = 0;
        let mut failed_imports = 0;
        let mut errors = Vec::new();
        let mut imported_ids = Vec::new();

        for (index, sequence_request) in request.sequences.into_iter().enumerate() {
            match self.save_sequence(&sequence_request) {
                Ok(id) => {
                    successful_imports += 1;
                    imported_ids.push(id);
                }
                Err(e) => {
                    failed_imports += 1;
                    errors.push(BatchImportError {
                        index,
                        sequence_name: sequence_request.name,
                        error: e.to_string(),
                    });
                }
            }
        }

        BatchImportResponse {
            version: crate::error::API_VERSION.to_string(),
            successful_imports,
            failed_imports,
            errors,
            imported_ids,
        }
    }
}
