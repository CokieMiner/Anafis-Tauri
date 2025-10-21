// Database operations for Data Library
use rusqlite::{params, Connection, Result as SqliteResult};
use std::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;

use super::models::*;

pub struct DataLibraryDatabase {
    conn: Mutex<Connection>,
}

impl DataLibraryDatabase {
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
    pub fn save_sequence(&self, request: SaveSequenceRequest) -> SqliteResult<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Serialize data and uncertainties to JSON
        let data_json = serde_json::to_string(&request.data)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let uncertainties_json = request.uncertainties.as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let tags_json = serde_json::to_string(&request.tags)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
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
                request.is_pinned as i32,
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
        
        Ok(id)
    }
    
    /// Get all sequences with optional filtering and sorting
    pub fn get_sequences(&self, search: &SearchRequest) -> SqliteResult<Vec<DataSequence>> {
        let conn = self.conn.lock().unwrap();
        
        let mut query = String::from("SELECT id, name, description, tags, unit, source, data, uncertainties, is_pinned, created_at, modified_at FROM sequences");
        let mut where_clauses = Vec::new();
        
        // Add full-text search if query provided
        if let Some(q) = &search.query {
            if !q.is_empty() {
                query = "SELECT s.id, s.name, s.description, s.tags, s.unit, s.source, s.data, s.uncertainties, s.is_pinned, s.created_at, s.modified_at 
                     FROM sequences s
                     JOIN sequences_fts fts ON s.id = fts.id
                     WHERE sequences_fts MATCH ?1".to_string();
            }
        }
        
        // Add tag filter
        if let Some(tags) = &search.tags {
            if !tags.is_empty() {
                let tags_conditions: Vec<String> = tags.iter()
                    .map(|_| "tags LIKE ?".to_string())
                    .collect();
                where_clauses.push(format!("({})", tags_conditions.join(" OR ")));
            }
        }
        
        // Add source filter
        if let Some(source) = &search.source {
            if !source.is_empty() {
                where_clauses.push("source = ?".to_string());
            }
        }
        
        // Add WHERE clause if needed
        if !where_clauses.is_empty() && search.query.is_none() {
            query.push_str(&format!(" WHERE {}", where_clauses.join(" AND ")));
        }
        
        // Add ORDER BY
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
        query.push_str(&format!(" ORDER BY is_pinned DESC, {} {}", order_col, order_dir));
        
        let mut stmt = conn.prepare(&query)?;
        
        // Execute query with parameters
        let mut rows = if let Some(q) = &search.query {
            if !q.is_empty() {
                stmt.query(params![q])?
            } else {
                stmt.query([])?
            }
        } else {
            stmt.query([])?
        };
        
        let mut sequences = Vec::new();
        while let Some(row) = rows.next()? {
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
            
            // Deserialize JSON fields
            let tags: Vec<String> = serde_json::from_str(&tags_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    3, rusqlite::types::Type::Text, Box::new(e)
                ))?;
            let data: Vec<f64> = serde_json::from_str(&data_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    6, rusqlite::types::Type::Text, Box::new(e)
                ))?;
            let uncertainties: Option<Vec<f64>> = uncertainties_json
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    7, rusqlite::types::Type::Text, Box::new(e)
                ))?;
            
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    9, rusqlite::types::Type::Text, Box::new(e)
                ))?
                .with_timezone(&Utc);
            let modified_at = chrono::DateTime::parse_from_rfc3339(&modified_at_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    10, rusqlite::types::Type::Text, Box::new(e)
                ))?
                .with_timezone(&Utc);
            
            sequences.push(DataSequence {
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
            });
        }
        
        Ok(sequences)
    }
    
    /// Get a single sequence by ID
    pub fn get_sequence(&self, id: &str) -> SqliteResult<Option<DataSequence>> {
        let sequences = self.get_sequences(&SearchRequest::default())?;
        Ok(sequences.into_iter().find(|s| s.id == id))
    }
    
    /// Update an existing sequence
    pub fn update_sequence(&self, request: UpdateSequenceRequest) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
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
            params.push(Box::new(is_pinned as i32));
        }
        
        updates.push("modified_at = ?");
        params.push(Box::new(now.to_rfc3339()));
        
        params.push(Box::new(request.id.clone()));
        
        let query = format!(
            "UPDATE sequences SET {} WHERE id = ?",
            updates.join(", ")
        );
        
        conn.execute(&query, rusqlite::params_from_iter(params.iter()))?;
        
        // Update FTS table if name, description, or tags changed
        if request.name.is_some() || request.description.is_some() || request.tags.is_some() {
            // Get current values for FTS update
            let mut stmt = conn.prepare("SELECT name, description, tags, source FROM sequences WHERE id = ?")?;
            let mut rows = stmt.query(params![request.id])?;
            
            if let Some(row) = rows.next()? {
                let name: String = row.get(0)?;
                let description: String = row.get(1)?;
                let tags_json: String = row.get(2)?;
                let source: String = row.get(3)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        2, rusqlite::types::Type::Text, Box::new(e)
                    ))?;
                
                conn.execute(
                    "UPDATE sequences_fts SET name = ?1, description = ?2, tags = ?3, source = ?4 WHERE id = ?5",
                    params![name, description, tags.join(" "), source, request.id],
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Delete a sequence
    pub fn delete_sequence(&self, id: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sequences WHERE id = ?", params![id])?;
        conn.execute("DELETE FROM sequences_fts WHERE id = ?", params![id])?;
        Ok(())
    }
    
    /// Get all unique tags
    pub fn get_all_tags(&self) -> SqliteResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT DISTINCT tags FROM sequences")?;
        let mut rows = stmt.query([])?;
        
        let mut all_tags = std::collections::HashSet::new();
        while let Some(row) = rows.next()? {
            let tags_json: String = row.get(0)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0, rusqlite::types::Type::Text, Box::new(e)
                ))?;
            all_tags.extend(tags);
        }
        
        let mut tags: Vec<String> = all_tags.into_iter().collect();
        tags.sort();
        Ok(tags)
    }
    
    /// Duplicate a sequence
    pub fn duplicate_sequence(&self, id: &str, new_name: &str) -> SqliteResult<String> {
        let sequence = self.get_sequence(id)?
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
        
        self.save_sequence(request)
    }
    
    /// Export sequences to CSV format
    /// Format: Each sequence becomes a column pair (name, name_uncertainty)
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
        let max_len = sequences.iter()
            .map(|s| s.data.len())
            .max()
            .unwrap_or(0);
        
        let mut file = File::create(file_path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        // Write header row with embedded metadata
        // Format: [name,unit,description,tags]
        let mut header = Vec::new();
        for seq in &sequences {
            // Escape commas and quotes in metadata fields
            let name = seq.name.replace(",", ";").replace("\"", "'");
            let unit = seq.unit.replace(",", ";").replace("\"", "'");
            let description = seq.description.replace(",", ";").replace("\"", "'");
            let tags = seq.tags.join("|").replace(",", ";").replace("\"", "'");
            
            // Data column with full metadata
            header.push(format!("[{},{},{},{}]", name, unit, description, tags));
            
            // Uncertainty column if present
            if seq.uncertainties.is_some() {
                header.push(format!("[{}_uncertainty,{},Uncertainty values for {},{}]", 
                    name, unit, name, tags));
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
    
    /// Export sequences to JSON format with full metadata
    pub fn export_to_json(&self, sequence_ids: &[String], file_path: &str) -> SqliteResult<()> {
        use std::fs::File;
        
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
        
        // Create export structure
        let export_data = serde_json::json!({
            "format": "AnaFis Data Export",
            "version": "1.0",
            "exported_at": Utc::now().to_rfc3339(),
            "sequences": sequences
        });
        
        let file = File::create(file_path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        serde_json::to_writer_pretty(file, &export_data)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        Ok(())
    }
}
