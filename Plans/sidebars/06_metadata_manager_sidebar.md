# Metadata Manager Sidebar 📝

**Status**: Planned  
**Priority**: Low  
**Complexity**: Medium  
**Dependencies**: SQLite (rusqlite), chrono, serde_json (Rust backend)

---

## Purpose

Manage metadata associated with cells, ranges, and the entire spreadsheet. Store contextual information like measurement conditions, instrument details, calibration data, and notes. All metadata persists in SQLite database with full-text search capabilities.

**Architecture**: Rust handles all storage, search, and data operations. TypeScript only for UI rendering and user input.anager Sidebar �

**Status**: Planned  
**Priority**: Low  
**Complexity**: Low  
**Dependencies**: None (pure frontend with localStorage)

---

## Purpose

Manage metadata associated with cells, ranges, and the entire spreadsheet. Store contextual information like measurement conditions, instrument details, calibration data, and notes.

---

## Features

### Metadata Categories
- **Instrument Info**: Name, model, serial number, calibration date
- **Measurement Conditions**: Temperature, pressure, humidity, date/time
- **Data Source**: File name, import date, original format
- **Calibration Data**: Calibration curve, uncertainty, standards used
- **Operator Info**: Name, lab, notes
- **Cell-Specific Notes**: Comments, tags, quality flags
- **Units**: Associated units for numeric values

### Storage Levels
- **Cell-level**: Metadata for individual cells
- **Range-level**: Metadata for ranges (e.g., column represents temperature)
- **Sheet-level**: Metadata for entire worksheet
- **Workbook-level**: Global metadata

### Search & Filter
- Full-text search across all metadata fields (SQLite FTS5)
- Filter by category, date, operator, quality
- Find cells with specific metadata
- Tag-based filtering

### Export/Import
- Export metadata as JSON
- Import metadata from file
- Automatically included in .anafis export
- Backup and restore via database

---

## UI Layout

```
┌─────────────────────────────────────┐
│ Metadata Manager                [X] │
├─────────────────────────────────────┤
│ Selection: [A1:A100]      [Change]  │
│                                     │
│ Scope: [Cell-level ▼]              │
│  • Cell-level                      │
│  • Range-level                     │
│  • Sheet-level                     │
│  • Workbook-level                  │
│                                     │
│ ┌───────────────────────────────┐ │
│ │ Metadata Fields               │ │
│ │                               │ │
│ │ ┌─ Instrument ──────────────┐ │ │
│ │ │ Name:   [Thermometer____] │ │ │
│ │ │ Model:  [TH-1000________] │ │ │
│ │ │ Serial: [123456_________] │ │ │
│ │ │ Cal. Date: [2024-01-15] 📅│ │ │
│ │ └──────────────────────────┘ │ │
│ │                               │ │
│ │ ┌─ Measurement Conditions ──┐ │ │
│ │ │ Temperature: [23.5°C____] │ │ │
│ │ │ Pressure:    [101.3 kPa_] │ │ │
│ │ │ Humidity:    [45%_______] │ │ │
│ │ │ Date/Time: [2024-03-20]📅│ │ │
│ │ │            [14:30:00]⏰   │ │ │
│ │ └──────────────────────────┘ │ │
│ │                               │ │
│ │ ┌─ Data Source ─────────────┐ │ │
│ │ │ File: [experiment_01.csv] │ │ │
│ │ │ Import: [2024-03-20]      │ │ │
│ │ │ Format: [CSV]             │ │ │
│ │ └──────────────────────────┘ │ │
│ │                               │ │
│ │ ┌─ Operator Info ───────────┐ │ │
│ │ │ Name: [John Doe_________] │ │ │
│ │ │ Lab:  [Physics Lab A____] │ │ │
│ │ │ Email: [j.doe@lab.com__] │ │ │
│ │ └──────────────────────────┘ │ │
│ │                               │ │
│ │ ┌─ Notes ───────────────────┐ │ │
│ │ │ [Repeated measurement     │ │ │
│ │ │  after instrument         │ │ │
│ │ │  recalibration. Results   │ │ │
│ │ │  show improved accuracy.] │ │ │
│ │ └──────────────────────────┘ │ │
│ │                               │ │
│ │ ┌─ Units ───────────────────┐ │ │
│ │ │ Unit: [°C ▼]              │ │ │
│ │ │ Uncertainty: [±0.1_____]  │ │ │
│ │ └──────────────────────────┘ │ │
│ │                               │ │
│ │ Tags: [#calibration] [#temp] │ │
│ │       [+ Add Tag]            │ │ │
│ └───────────────────────────────┘ │
│                                     │
│ [Save Metadata] [Clear]            │
│                                     │
│ ┌─ Search Metadata ──────────────┐ │
│ │ Search: [thermometer_________] │ │
│ │ [Search]                       │ │
│ │                                │ │
│ │ Results: 3 cells found         │ │
│ │  • A1:A100 - Instrument: ...   │ │
│ │  • B5 - Note mentions ...      │ │
│ │  • C12 - Tag: #thermometer     │ │
│ └────────────────────────────────┘ │
│                                     │
│ [Export Metadata (JSON)]           │
│ [Import Metadata]                  │
└─────────────────────────────────────┘
```

---

## Data Flow Pattern

**Type**: UI → Rust → SQLite (Rust-First Architecture)

1. User selects cell/range
2. **Rust**: Query metadata from SQLite database via `get_metadata` command
3. TypeScript: Display in sidebar form
4. User edits metadata fields
5. TypeScript: Collect form data, call `save_metadata` command
6. **Rust**: Validate and save to SQLite database
7. TypeScript: Update UI with confirmation
8. Metadata automatically included in .anafis export via Rust serialization

**All business logic in Rust**: Storage, validation, search, export/import

---

## Technical Implementation

### TypeScript Interfaces

```typescript
interface MetadataManagerSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef: UniverSpreadsheetRef;
  onSelectionChange: (cellRef: string) => void;
}

type MetadataScope = 'cell' | 'range' | 'sheet' | 'workbook';

interface InstrumentInfo {
  name: string;
  model: string;
  serialNumber: string;
  calibrationDate: Date | null;
  calibrationCertificate?: string; // File reference
}

interface MeasurementConditions {
  temperature?: number; // °C
  temperatureUnit?: string;
  pressure?: number; // kPa
  pressureUnit?: string;
  humidity?: number; // %
  dateTime?: Date;
  location?: string;
}

interface DataSource {
  fileName?: string;
  importDate?: Date;
  originalFormat?: string; // CSV, XLSX, etc.
  originalPath?: string;
}

interface OperatorInfo {
  name: string;
  lab?: string;
  email?: string;
  orcid?: string; // ORCID researcher ID
}

interface UnitInfo {
  unit: string;
  uncertainty?: number;
  uncertaintyUnit?: string;
}

interface CellMetadata {
  scope: MetadataScope;
  target: string; // Cell ref (A1) or range (A1:B10)
  
  instrument?: InstrumentInfo;
  conditions?: MeasurementConditions;
  source?: DataSource;
  operator?: OperatorInfo;
  units?: UnitInfo;
  
  notes?: string;
  tags?: string[];
  
  quality?: 'good' | 'questionable' | 'bad';
  
  createdAt: Date;
  modifiedAt: Date;
}
```

---

## Rust Backend Implementation

### Database Schema

```sql
-- Metadata table with full-text search
CREATE TABLE IF NOT EXISTS metadata (
    id TEXT PRIMARY KEY,
    target TEXT NOT NULL,           -- Cell/range reference (e.g., "A1:A100", "Sheet1")
    scope TEXT NOT NULL,            -- 'cell', 'range', 'sheet', 'workbook'
    
    -- Instrument info
    instrument_name TEXT,
    instrument_model TEXT,
    instrument_serial TEXT,
    calibration_date TEXT,
    
    -- Measurement conditions
    temperature TEXT,
    pressure TEXT,
    humidity TEXT,
    measurement_datetime TEXT,
    
    -- Data source
    source_file TEXT,
    import_date TEXT,
    original_format TEXT,
    
    -- Calibration data
    calibration_curve TEXT,
    uncertainty TEXT,
    standards_used TEXT,
    
    -- Operator info
    operator_name TEXT,
    lab TEXT,
    operator_email TEXT,
    
    -- Units
    unit TEXT,
    unit_uncertainty TEXT,
    
    -- Notes and quality
    notes TEXT,
    quality TEXT,                   -- 'good', 'questionable', 'bad'
    
    -- Tags (JSON array)
    tags TEXT,
    
    -- Timestamps
    created_at TEXT NOT NULL,
    modified_at TEXT NOT NULL
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_metadata_target ON metadata(target);
CREATE INDEX IF NOT EXISTS idx_metadata_scope ON metadata(scope);
CREATE INDEX IF NOT EXISTS idx_metadata_quality ON metadata(quality);
CREATE INDEX IF NOT EXISTS idx_metadata_modified ON metadata(modified_at);

-- Full-text search virtual table
CREATE VIRTUAL TABLE IF NOT EXISTS metadata_fts USING fts5(
    target,
    instrument_name,
    instrument_model,
    operator_name,
    lab,
    notes,
    tags,
    content='metadata',
    content_rowid='rowid'
);

-- Triggers to keep FTS table in sync
CREATE TRIGGER IF NOT EXISTS metadata_fts_insert AFTER INSERT ON metadata BEGIN
    INSERT INTO metadata_fts(rowid, target, instrument_name, instrument_model, operator_name, lab, notes, tags)
    VALUES (new.rowid, new.target, new.instrument_name, new.instrument_model, new.operator_name, new.lab, new.notes, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS metadata_fts_delete AFTER DELETE ON metadata BEGIN
    DELETE FROM metadata_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER IF NOT EXISTS metadata_fts_update AFTER UPDATE ON metadata BEGIN
    UPDATE metadata_fts SET
        target = new.target,
        instrument_name = new.instrument_name,
        instrument_model = new.instrument_model,
        operator_name = new.operator_name,
        lab = new.lab,
        notes = new.notes,
        tags = new.tags
    WHERE rowid = new.rowid;
END;
```

### Rust Commands

```rust
// AnaFis/src-tauri/src/metadata/mod.rs

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub id: String,
    pub target: String,
    pub scope: String,
    
    // Instrument
    pub instrument_name: Option<String>,
    pub instrument_model: Option<String>,
    pub instrument_serial: Option<String>,
    pub calibration_date: Option<String>,
    
    // Measurement conditions
    pub temperature: Option<String>,
    pub pressure: Option<String>,
    pub humidity: Option<String>,
    pub measurement_datetime: Option<String>,
    
    // Data source
    pub source_file: Option<String>,
    pub import_date: Option<String>,
    pub original_format: Option<String>,
    
    // Calibration
    pub calibration_curve: Option<String>,
    pub uncertainty: Option<String>,
    pub standards_used: Option<String>,
    
    // Operator
    pub operator_name: Option<String>,
    pub lab: Option<String>,
    pub operator_email: Option<String>,
    
    // Units
    pub unit: Option<String>,
    pub unit_uncertainty: Option<String>,
    
    // Notes and quality
    pub notes: Option<String>,
    pub quality: Option<String>,
    
    // Tags
    pub tags: Vec<String>,
    
    // Timestamps
    pub created_at: String,
    pub modified_at: String,
}

// Save or update metadata
#[tauri::command]
pub async fn save_metadata(metadata: Metadata) -> Result<(), String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let tags_json = serde_json::to_string(&metadata.tags)
        .map_err(|e| format!("Failed to serialize tags: {}", e))?;
    
    let now = Utc::now().to_rfc3339();
    
    // Check if metadata exists
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM metadata WHERE target = ? AND scope = ?)",
        params![&metadata.target, &metadata.scope],
        |row| row.get(0)
    ).map_err(|e| format!("Query failed: {}", e))?;
    
    if exists {
        // Update existing
        conn.execute(
            "UPDATE metadata SET
                instrument_name = ?, instrument_model = ?, instrument_serial = ?, calibration_date = ?,
                temperature = ?, pressure = ?, humidity = ?, measurement_datetime = ?,
                source_file = ?, import_date = ?, original_format = ?,
                calibration_curve = ?, uncertainty = ?, standards_used = ?,
                operator_name = ?, lab = ?, operator_email = ?,
                unit = ?, unit_uncertainty = ?,
                notes = ?, quality = ?, tags = ?,
                modified_at = ?
            WHERE target = ? AND scope = ?",
            params![
                metadata.instrument_name, metadata.instrument_model, metadata.instrument_serial, metadata.calibration_date,
                metadata.temperature, metadata.pressure, metadata.humidity, metadata.measurement_datetime,
                metadata.source_file, metadata.import_date, metadata.original_format,
                metadata.calibration_curve, metadata.uncertainty, metadata.standards_used,
                metadata.operator_name, metadata.lab, metadata.operator_email,
                metadata.unit, metadata.unit_uncertainty,
                metadata.notes, metadata.quality, tags_json,
                now,
                metadata.target, metadata.scope
            ]
        ).map_err(|e| format!("Update failed: {}", e))?;
    } else {
        // Insert new
        let id = metadata.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        
        conn.execute(
            "INSERT INTO metadata (
                id, target, scope,
                instrument_name, instrument_model, instrument_serial, calibration_date,
                temperature, pressure, humidity, measurement_datetime,
                source_file, import_date, original_format,
                calibration_curve, uncertainty, standards_used,
                operator_name, lab, operator_email,
                unit, unit_uncertainty,
                notes, quality, tags,
                created_at, modified_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id, metadata.target, metadata.scope,
                metadata.instrument_name, metadata.instrument_model, metadata.instrument_serial, metadata.calibration_date,
                metadata.temperature, metadata.pressure, metadata.humidity, metadata.measurement_datetime,
                metadata.source_file, metadata.import_date, metadata.original_format,
                metadata.calibration_curve, metadata.uncertainty, metadata.standards_used,
                metadata.operator_name, metadata.lab, metadata.operator_email,
                metadata.unit, metadata.unit_uncertainty,
                metadata.notes, metadata.quality, tags_json,
                now, now
            ]
        ).map_err(|e| format!("Insert failed: {}", e))?;
    }
    
    Ok(())
}

// Get metadata for target
#[tauri::command]
pub async fn get_metadata(target: String, scope: String) -> Result<Option<Metadata>, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut stmt = conn.prepare(
        "SELECT * FROM metadata WHERE target = ? AND scope = ?"
    ).map_err(|e| format!("Prepare failed: {}", e))?;
    
    let metadata = stmt.query_row(params![target, scope], |row| {
        let tags_str: String = row.get(24)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        
        Ok(Metadata {
            id: row.get(0)?,
            target: row.get(1)?,
            scope: row.get(2)?,
            instrument_name: row.get(3)?,
            instrument_model: row.get(4)?,
            instrument_serial: row.get(5)?,
            calibration_date: row.get(6)?,
            temperature: row.get(7)?,
            pressure: row.get(8)?,
            humidity: row.get(9)?,
            measurement_datetime: row.get(10)?,
            source_file: row.get(11)?,
            import_date: row.get(12)?,
            original_format: row.get(13)?,
            calibration_curve: row.get(14)?,
            uncertainty: row.get(15)?,
            standards_used: row.get(16)?,
            operator_name: row.get(17)?,
            lab: row.get(18)?,
            operator_email: row.get(19)?,
            unit: row.get(20)?,
            unit_uncertainty: row.get(21)?,
            notes: row.get(22)?,
            quality: row.get(23)?,
            tags,
            created_at: row.get(25)?,
            modified_at: row.get(26)?,
        })
    }).optional()
    .map_err(|e| format!("Query failed: {}", e))?;
    
    Ok(metadata)
}

// Delete metadata
#[tauri::command]
pub async fn delete_metadata(target: String, scope: String) -> Result<(), String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    conn.execute(
        "DELETE FROM metadata WHERE target = ? AND scope = ?",
        params![target, scope]
    ).map_err(|e| format!("Delete failed: {}", e))?;
    
    Ok(())
}

// Search metadata using FTS5
#[tauri::command]
pub async fn search_metadata(query: String) -> Result<Vec<Metadata>, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut stmt = conn.prepare(
        "SELECT m.* FROM metadata m
         JOIN metadata_fts fts ON m.rowid = fts.rowid
         WHERE metadata_fts MATCH ?
         ORDER BY rank"
    ).map_err(|e| format!("Prepare failed: {}", e))?;
    
    let results = stmt.query_map(params![query], |row| {
        let tags_str: String = row.get(24)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        
        Ok(Metadata {
            id: row.get(0)?,
            target: row.get(1)?,
            scope: row.get(2)?,
            instrument_name: row.get(3)?,
            instrument_model: row.get(4)?,
            instrument_serial: row.get(5)?,
            calibration_date: row.get(6)?,
            temperature: row.get(7)?,
            pressure: row.get(8)?,
            humidity: row.get(9)?,
            measurement_datetime: row.get(10)?,
            source_file: row.get(11)?,
            import_date: row.get(12)?,
            original_format: row.get(13)?,
            calibration_curve: row.get(14)?,
            uncertainty: row.get(15)?,
            standards_used: row.get(16)?,
            operator_name: row.get(17)?,
            lab: row.get(18)?,
            operator_email: row.get(19)?,
            unit: row.get(20)?,
            unit_uncertainty: row.get(21)?,
            notes: row.get(22)?,
            quality: row.get(23)?,
            tags,
            created_at: row.get(25)?,
            modified_at: row.get(26)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?;
    
    results.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Collection failed: {}", e))
}

// Get all tags
#[tauri::command]
pub async fn get_all_metadata_tags() -> Result<Vec<String>, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut stmt = conn.prepare("SELECT DISTINCT tags FROM metadata WHERE tags IS NOT NULL")
        .map_err(|e| format!("Prepare failed: {}", e))?;
    
    let tag_sets = stmt.query_map([], |row| {
        let tags_str: String = row.get(0)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        Ok(tags)
    }).map_err(|e| format!("Query failed: {}", e))?;
    
    let mut all_tags = std::collections::HashSet::new();
    for tag_set in tag_sets {
        if let Ok(tags) = tag_set {
            for tag in tags {
                all_tags.insert(tag);
            }
        }
    }
    
    Ok(all_tags.into_iter().collect())
}

// Export all metadata as JSON
#[tauri::command]
pub async fn export_metadata() -> Result<String, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut stmt = conn.prepare("SELECT * FROM metadata ORDER BY created_at")
        .map_err(|e| format!("Prepare failed: {}", e))?;
    
    let results = stmt.query_map([], |row| {
        let tags_str: String = row.get(24)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        
        Ok(Metadata {
            id: row.get(0)?,
            target: row.get(1)?,
            scope: row.get(2)?,
            instrument_name: row.get(3)?,
            instrument_model: row.get(4)?,
            instrument_serial: row.get(5)?,
            calibration_date: row.get(6)?,
            temperature: row.get(7)?,
            pressure: row.get(8)?,
            humidity: row.get(9)?,
            measurement_datetime: row.get(10)?,
            source_file: row.get(11)?,
            import_date: row.get(12)?,
            original_format: row.get(13)?,
            calibration_curve: row.get(14)?,
            uncertainty: row.get(15)?,
            standards_used: row.get(16)?,
            operator_name: row.get(17)?,
            lab: row.get(18)?,
            operator_email: row.get(19)?,
            unit: row.get(20)?,
            unit_uncertainty: row.get(21)?,
            notes: row.get(22)?,
            quality: row.get(23)?,
            tags,
            created_at: row.get(25)?,
            modified_at: row.get(26)?,
        })
    }).map_err(|e| format!("Query failed: {}", e))?;
    
    let metadata: Vec<Metadata> = results.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Collection failed: {}", e))?;
    
    serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Serialization failed: {}", e))
}

// Import metadata from JSON
#[tauri::command]
pub async fn import_metadata(json: String) -> Result<usize, String> {
    let metadata_list: Vec<Metadata> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut count = 0;
    for metadata in metadata_list {
        if save_metadata(metadata).await.is_ok() {
            count += 1;
        }
    }
    
    Ok(count)
}

fn get_db_path() -> Result<std::path::PathBuf, String> {
    let app_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .ok_or_else(|| "Failed to get app data directory".to_string())?;
    Ok(app_dir.join("anafis_metadata.db"))
}
```

---

## TypeScript Implementation (UI Only)

### Metadata Store Class (Removed - Now uses Rust)

```typescript
// REMOVED: MetadataStore class
// All storage operations now via Tauri commands:

import { invoke } from '@tauri-apps/api/tauri';

// Save metadata (calls Rust)
async function saveMetadata(metadata: CellMetadata): Promise<void> {
  await invoke('save_metadata', { metadata });
}

// Get metadata (calls Rust)
async function getMetadata(target: string, scope: MetadataScope): Promise<CellMetadata | null> {
  return await invoke('get_metadata', { target, scope });
}

// Delete metadata (calls Rust)
async function deleteMetadata(target: string, scope: MetadataScope): Promise<void> {
  await invoke('delete_metadata', { target, scope });
}

// Search metadata (calls Rust with FTS5)
async function searchMetadata(query: string): Promise<CellMetadata[]> {
  return await invoke('search_metadata', { query });
}

// Get all tags (calls Rust)
async function getAllTags(): Promise<string[]> {
  return await invoke('get_all_metadata_tags');
}

// Export all metadata (calls Rust)
async function exportMetadata(): Promise<string> {
  return await invoke('export_metadata');
}

// Import metadata (calls Rust)
async function importMetadata(json: string): Promise<number> {
  return await invoke('import_metadata', { json });
}
```

---

## Integration with .anafis Format

When exporting to .anafis format, include metadata:

```typescript
interface AnaFisFormat {
  version: string;
  created: Date;
  
  sheets: Array<{
    name: string;
    data: any[][];
    formulas: Record<string, string>;
    formatting: Record<string, any>;
    metadata: Record<string, CellMetadata>; // ← Include here
  }>;
  
  globalMetadata: CellMetadata[]; // Workbook-level metadata
}
```

---

## Dependencies

**Rust Dependencies**:
- rusqlite 0.31 with bundled SQLite and FTS5
- serde, serde_json
- chrono 0.4
- uuid 1.6
- tauri 1.x

**TypeScript Dependencies**:
- @tauri-apps/api (Tauri invoke)
- React, Material-UI (UI components)

**Note**: NO localStorage - all persistence in SQLite via Rust backend

---

## File Location

- **Component**: `AnaFis/src/components/spreadsheet/MetadataManagerSidebar.tsx`
- **Rust Backend**: `AnaFis/src-tauri/src/metadata/mod.rs`
- **Types**: `AnaFis/src/types/metadata.ts` (TypeScript interfaces matching Rust structs)
- **Database**: `~/.local/share/anafis/anafis_metadata.db` (separate from Data Library)

---

## Success Criteria

- ✓ All metadata fields can be stored and retrieved
- ✓ Full-text search via SQLite FTS5 works correctly
- ✓ Metadata persists across sessions (SQLite)
- ✓ Export/import works correctly
- ✓ Metadata included in .anafis exports
- ✓ **All business logic in Rust, TypeScript for UI only**
- ✓ Performance: Handle 10,000+ metadata entries with fast search (<50ms)
- ✓ Database queries are type-safe and validated

---

## Architecture Notes

**Rust-First Design**:
- All storage, search, validation in Rust
- SQLite with FTS5 for fast full-text search
- TypeScript only handles: UI rendering, form input, calling invoke()
- Benefits: Data integrity, persistent storage, fast search, consistent with app architecture

**Database Strategy**:
- Separate database from Data Library (different purpose)
- FTS5 virtual table for instant search across all fields
- Triggers keep FTS in sync automatically
- Indexes for fast target/scope lookups

---

**Next Steps**: Implement after Quick Plot Sidebar (low priority)
