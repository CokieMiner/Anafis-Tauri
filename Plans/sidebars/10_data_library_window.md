# Data Library Window

**Status**: Planned  
**Priority**: CRITICAL (Core Infrastructure)  
**Complexity**: High  
**Dependencies**: SQLite (rusqlite), Tauri window management, Tauri State management

---

## Purpose

The Data Library Window is a standalone window for managing persistent data sequences. It provides a centralized interface to save, organize, search, and export data from spreadsheets. Data sequences can be tagged, searched, filtered, and loaded back into any spreadsheet tab or used in other components (Graphs & Fitting, Monte Carlo, Solver).

**Architecture**: All data storage, search, filtering, sorting, and statistics are handled in Rust backend using SQLite. TypeScript frontend only handles UI rendering and user interactions.

---

## Features

### Core Functionality

1. **Data Sequence Management**
   - Save data from spreadsheet selections
   - View all saved sequences in a searchable list
   - Edit sequence metadata (name, tags, units)
   - Delete sequences (with confirmation)
   - Pin favorite sequences to top
   - Duplicate sequences

2. **Organization & Search**
   - Tag-based categorization
   - Full-text search (name, tags, source)
   - Filter by tags (multi-select)
   - Sort by: name, date created, date modified, size
   - Group by: tags, source spreadsheet, date

3. **Data Operations**
   - Preview data (table + mini chart)
   - Load data to active spreadsheet
   - Export sequences (JSON, CSV, Excel)
   - Import sequences from files
   - Batch operations (delete, tag, export)

4. **Statistics & Analysis**
   - Quick stats display (mean, std dev, min, max, count)
   - Distribution histogram
   - Correlation matrix between sequences
   - Data quality indicators

---

## UI Layout

### Window Specifications
- **Size**: 1000x700px (default)
- **Min Size**: 800x500px
- **Resizable**: Yes
- **Position**: Centered on screen
- **Title**: "Data Library"

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│  Data Library                                    [─][□][×] │
├────────────────────────────────────────────────────────────┤
│  [Search...]  [🔍]  [Tags ▾]  [Sort ▾]  [+ New]  [Import] │
├──────────────┬─────────────────────────────────────────────┤
│              │  Sequence Name          📊 Quick Stats      │
│  PINNED      │  Tags: physics, lab1    ┌──────────────┐   │
│  ⭐ Voltage  │  Source: Sheet 1        │  Mean: 5.2   │   │
│  ⭐ Current  │  200 points             │  Std: 0.8    │   │
│              │  Modified: 2h ago       │  Min: 3.1    │   │
│  ALL (45)    │                         │  Max: 7.9    │   │
│  🏷️ physics  │  [Load] [Edit] [Delete] └──────────────┘   │
│  🏷️ lab1     ├─────────────────────────────────────────────┤
│  🏷️ lab2     │  📈 Data Preview                            │
│  📁 Sheet 1  │  ┌────────┬──────────┬──────────┐          │
│  📁 Sheet 2  │  │  X     │  Y       │  σY      │          │
│              │  │  1.0   │  5.2±0.1 │          │          │
│  RECENT      │  │  2.0   │  5.8±0.1 │          │          │
│  Yesterday   │  │  3.0   │  6.1±0.2 │          │          │
│  Last Week   │  │  ...   │  ...     │          │          │
│  Older       │  └────────┴──────────┴──────────┘          │
│              │  [Mini Chart: Line/Scatter with error bars] │
└──────────────┴─────────────────────────────────────────────┘
```

---

## Detailed UI Components

### 1. Toolbar (Top)

#### Search Bar
- **Width**: Flexible (40% of window width)
- **Placeholder**: "Search sequences..."
- **Behavior**: Live search as user types (debounced 300ms)
- **Searches**: Name, tags, source, description

#### Tag Filter Dropdown
- **Button**: "Tags ▾"
- **Content**: Multi-select checkboxes for all tags
- **Actions**: 
  - Select/deselect tags
  - "Clear All" button
  - Shows count of selected tags

#### Sort Dropdown
- **Button**: "Sort ▾"
- **Options**:
  - Name (A-Z)
  - Name (Z-A)
  - Date Created (Newest)
  - Date Created (Oldest)
  - Date Modified (Newest)
  - Date Modified (Oldest)
  - Size (Largest)
  - Size (Smallest)

#### Action Buttons
- **[+ New]**: Save data from active spreadsheet selection
- **[Import]**: Import sequences from file (JSON, CSV, Excel)

---

### 2. Sidebar (Left, 200px)

#### Pinned Section
- Shows pinned sequences (⭐ icon)
- Click to select and show in main panel
- Drag to reorder

#### All Sequences Counter
- Shows total number of sequences
- Always visible

#### Tag List
- Groups sequences by tags
- Shows count per tag
- Click to filter by tag
- 🏷️ icon for each tag
- Color-coded tags (optional)

#### Source Spreadsheet List
- Groups by source spreadsheet/tab
- Shows count per source
- 📁 icon for each source

#### Recent Section
- Time-based grouping:
  - Today
  - Yesterday
  - Last Week
  - Older
- Shows count per group

---

### 3. Main Panel (Center-Right)

#### Sequence Card (when sequence selected)

**Header Section**:
- **Name**: Large, editable on click
- **Tags**: Clickable chips (click to filter, ×  to remove)
- **Source**: "Sheet 1, Column A:B" (read-only)
- **Info**: Point count, uncertainties present, creation/modification dates

**Quick Stats Box**:
- Mean ± Standard Deviation
- Minimum
- Maximum
- Count
- Median (optional)
- Displayed in compact card on right side

**Action Buttons**:
- **[Load to Spreadsheet]**: Opens dialog to select target tab/range
- **[Edit]**: Open edit dialog (change name, tags, units, description)
- **[Export]**: Export this sequence (JSON/CSV/Excel)
- **[Duplicate]**: Create a copy
- **[⭐ Pin/Unpin]**: Toggle pinned status
- **[Delete]**: Delete with confirmation

**Data Preview Section**:
- **Table View** (default):
  - First 50 rows shown
  - Columns: Index, X, Y, σY (if present)
  - Scrollable
  - Copy-to-clipboard button
  
- **Chart View** (toggle):
  - Mini Plotly.js chart
  - Line or scatter plot with error bars
  - Zoom/pan disabled (preview only)
  - Click to open in Graphs & Fitting

---

### 4. List View (when no sequence selected)

**Grid of Sequence Cards**:
- **Card Size**: 250x150px
- **Layout**: Grid (3-4 columns depending on window width)
- **Each Card Shows**:
  - ⭐ Pin icon (top-left, if pinned)
  - Sequence name (truncated)
  - First tag (if present)
  - Point count
  - Mini sparkline chart
  - Last modified date
- **Hover**: Highlights card, shows "View Details" overlay
- **Click**: Opens detailed view

---

## Data Model

### DataSequence Structure (Rust)

```rust
// src-tauri/src/data_library/models.rs
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSequence {
    pub id: String,                    // UUID
    pub name: String,                  // User-defined name
    pub description: Option<String>,   // Optional description
    
    // Data (stored as JSON blobs in SQLite)
    pub values: Vec<f64>,              // Y values (or single column)
    pub x_values: Option<Vec<f64>>,    // X values (optional, for XY pairs)
    pub uncertainties: Option<Vec<f64>>, // Uncertainties for Y values
    
    // Metadata
    pub tags: Vec<String>,             // User-defined tags
    pub unit: Option<String>,          // Unit of measurement
    pub x_unit: Option<String>,        // X unit (if x_values present)
    pub source: SequenceSource,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    
    // Organization
    pub is_pinned: bool,
    pub color: Option<String>,         // Optional color tag
    
    // Computed statistics (cached in database)
    pub stats: SequenceStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceSource {
    pub spreadsheet_id: String,        // Tab ID
    pub spreadsheet_name: String,      // Tab name
    pub range: String,                 // e.g., "A1:B100"
    pub formula: Option<String>,       // If from calculated column
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

// TypeScript interface (for frontend only - no logic)
interface DataSequence {
  id: string;
  name: string;
  description?: string;
  values: number[];
  xValues?: number[];
  uncertainties?: number[];
  tags: string[];
  unit?: string;
  xUnit?: string;
  source: {
    spreadsheetId: string;
    spreadsheetName: string;
    range: string;
    formula?: string;
  };
  createdAt: string;  // ISO string
  modifiedAt: string; // ISO string
  isPinned: boolean;
  color?: string;
  stats: {
    count: number;
    mean: number;
    median: number;
    stdDev: number;
    min: number;
    max: number;
  };
}
```

---

## User Interactions

### 1. Saving Data from Spreadsheet

**From Main App**:
1. User selects range in spreadsheet
2. Clicks "Data Library" button in toolbar
3. Window opens with "Save Selection" dialog pre-filled:
   - **Name**: Auto-generated ("Data from Sheet 1") - editable
   - **Range**: Pre-filled from selection
   - **Tags**: Empty - user can add
   - **Description**: Optional
4. Click "Save" → Data saved to IndexedDB
5. Confirmation toast: "Saved 'Data Name' to library"

**From Data Library Window**:
1. Click "[+ New]" button
2. Dialog opens: "Save Data from Spreadsheet"
3. User selects:
   - Source tab (dropdown)
   - Range (text input or selection mode)
   - Name, tags, description
4. Click "Save"

### 2. Searching & Filtering

**Search**:
- Type in search bar → Results update live
- Searches: name, tags, description, source
- Shows "X results" counter

**Tag Filtering**:
- Click "Tags ▾" dropdown
- Select one or more tags
- Results show sequences with ANY selected tag (OR logic)
- Clear button to reset

**Combined**:
- Search + tag filters work together (AND logic)

### 3. Loading Data to Spreadsheet

**Method 1**: From sequence detail view
1. View sequence details
2. Click "[Load to Spreadsheet]"
3. Dialog opens:
   - Select target tab (dropdown)
   - Select target range (e.g., "C1:D100")
   - Options: 
     - [ ] Include headers
     - [ ] Include uncertainties
     - [ ] Overwrite existing data
4. Click "Load" → Data written to spreadsheet
5. Window stays open (or closes, user preference)

**Method 2**: Drag & drop (future)
- Drag sequence card from library
- Drop onto spreadsheet tab
- Data inserted at active cell

### 4. Editing Sequence

1. Click "[Edit]" button
2. Dialog opens with current values:
   - Name (text input)
   - Description (text area)
   - Tags (chip input with autocomplete)
   - Unit (text input)
3. Cannot edit: values, source, timestamps
4. Click "Save" → Updates in SQLite database

### 5. Deleting Sequence

1. Click "[Delete]" button
2. Confirmation dialog: "Delete 'Sequence Name'? This cannot be undone."
3. Options: [Cancel] [Delete]
4. On delete: Remove from SQLite database, close detail view, return to list

### 6. Batch Operations

1. Enable "Select Mode" (checkbox toggle at top)
2. Click checkboxes on sequence cards
3. Batch action bar appears at bottom:
   - "X selected"
   - [Tag] [Export] [Delete]
4. Perform action on all selected sequences

---

## Technical Implementation

### Architecture Overview

**Rust Backend (All Logic)**:
- SQLite database for persistent storage
- All CRUD operations
- Search and filter logic
- Sorting algorithms
- Statistics calculations
- Export/import file operations
- Data validation

**TypeScript Frontend (UI Only)**:
- React components for rendering
- User input handling
- Call Tauri commands
- Display data received from backend
- No business logic or calculations

---

### Frontend (React + TypeScript)

#### Component Structure

```
DataLibraryWindow/
├── DataLibraryWindow.tsx         # Main window component (UI only)
├── components/
│   ├── Toolbar.tsx               # Search input, filter dropdowns
│   ├── Sidebar.tsx               # Display pinned, tags, sources
│   ├── SequenceList.tsx          # Grid of sequence cards
│   ├── SequenceCard.tsx          # Individual card
│   ├── SequenceDetail.tsx        # Detailed view
│   ├── DataPreview.tsx           # Table/chart preview
│   ├── QuickStats.tsx            # Statistics display
│   └── dialogs/
│       ├── SaveDataDialog.tsx    # UI for save dialog
│       ├── EditDialog.tsx        # UI for edit dialog
│       ├── LoadDialog.tsx        # UI for load dialog
│       └── ImportDialog.tsx      # UI for import dialog
└── styles/
    └── DataLibrary.module.css
```

#### Main Window Component (Simplified - No Logic)

```typescript
// DataLibraryWindow.tsx
import React, { useState, useEffect } from 'react';
import { Box } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { Toolbar } from './components/Toolbar';
import { Sidebar } from './components/Sidebar';
import { SequenceList } from './components/SequenceList';
import { SequenceDetail } from './components/SequenceDetail';

interface DataSequence {
  // Same as Rust struct but with camelCase
  id: string;
  name: string;
  // ... other fields
}

export const DataLibraryWindow: React.FC = () => {
  const [sequences, setSequences] = useState<DataSequence[]>([]);
  const [selectedSequenceId, setSelectedSequenceId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // Load all sequences from Rust backend
  useEffect(() => {
    loadSequences();
  }, []);

  const loadSequences = async () => {
    setLoading(true);
    try {
      const result = await invoke<DataSequence[]>('get_all_sequences');
      setSequences(result);
    } catch (error) {
      console.error('Failed to load sequences:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = async (query: string) => {
    try {
      const result = await invoke<DataSequence[]>('search_sequences', { query });
      setSequences(result);
    } catch (error) {
      console.error('Search failed:', error);
    }
  };

  const handleFilterByTags = async (tags: string[]) => {
    try {
      const result = await invoke<DataSequence[]>('filter_sequences_by_tags', { tags });
      setSequences(result);
    } catch (error) {
      console.error('Filter failed:', error);
    }
  };

  const handleSort = async (sortBy: string) => {
    try {
      const result = await invoke<DataSequence[]>('sort_sequences', { sortBy });
      setSequences(result);
    } catch (error) {
      console.error('Sort failed:', error);
    }
  };

  const handleSaveSequence = async (sequenceData: any) => {
    try {
      await invoke('save_sequence', { sequenceData });
      await loadSequences(); // Reload list
    } catch (error) {
      console.error('Save failed:', error);
    }
  };

  const handleUpdateSequence = async (id: string, updates: any) => {
    try {
      await invoke('update_sequence', { id, updates });
      await loadSequences(); // Reload list
    } catch (error) {
      console.error('Update failed:', error);
    }
  };

  const handleDeleteSequence = async (id: string) => {
    try {
      await invoke('delete_sequence', { id });
      await loadSequences(); // Reload list
      setSelectedSequenceId(null);
    } catch (error) {
      console.error('Delete failed:', error);
    }
  };

  const selectedSequence = sequences.find(s => s.id === selectedSequenceId);

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      <Toolbar
        onSearch={handleSearch}
        onFilterByTags={handleFilterByTags}
        onSort={handleSort}
        onNewSequence={() => {/* Open save dialog */}}
        onImport={() => {/* Open import dialog */}}
      />

      <Box sx={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <Sidebar
          sequences={sequences}
          onSequenceClick={setSelectedSequenceId}
        />

        <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
          {selectedSequence ? (
            <SequenceDetail
              sequence={selectedSequence}
              onEdit={handleUpdateSequence}
              onDelete={handleDeleteSequence}
              onClose={() => setSelectedSequenceId(null)}
            />
          ) : (
            <SequenceList
              sequences={sequences}
              onSequenceClick={setSelectedSequenceId}
            />
          )}
        </Box>
      </Box>
    </Box>
  );
};
```

---

### Backend (Rust + Tauri)

#### Database Schema (SQLite)

```rust
// src-tauri/src/data_library/database.rs
use rusqlite::{Connection, Result};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sequences (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                values TEXT NOT NULL,        -- JSON array
                x_values TEXT,               -- JSON array (nullable)
                uncertainties TEXT,          -- JSON array (nullable)
                tags TEXT NOT NULL,          -- JSON array
                unit TEXT,
                x_unit TEXT,
                source_spreadsheet_id TEXT NOT NULL,
                source_spreadsheet_name TEXT NOT NULL,
                source_range TEXT NOT NULL,
                source_formula TEXT,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                is_pinned INTEGER NOT NULL DEFAULT 0,
                color TEXT,
                stats_count INTEGER NOT NULL,
                stats_mean REAL NOT NULL,
                stats_median REAL NOT NULL,
                stats_std_dev REAL NOT NULL,
                stats_min REAL NOT NULL,
                stats_max REAL NOT NULL
            )",
            [],
        )?;

        // Create indexes for fast queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_name ON sequences(name)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON sequences(created_at)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_modified_at ON sequences(modified_at)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_is_pinned ON sequences(is_pinned)",
            [],
        )?;

        // Full-text search virtual table
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS sequences_fts USING fts5(
                id UNINDEXED,
                name,
                description,
                tags,
                source_spreadsheet_name
            )",
            [],
        )?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
}
```

#### Statistics Calculation (Rust)

```rust
// src-tauri/src/data_library/stats.rs
use crate::data_library::models::SequenceStats;

pub fn calculate_stats(values: &[f64]) -> SequenceStats {
    if values.is_empty() {
        return SequenceStats {
            count: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
        };
    }

    let count = values.len();
    let sum: f64 = values.iter().sum();
    let mean = sum / count as f64;

    // Calculate standard deviation
    let variance: f64 = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / count as f64;
    let std_dev = variance.sqrt();

    // Calculate median
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if count % 2 == 0 {
        (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
    } else {
        sorted[count / 2]
    };

    let min = sorted.first().copied().unwrap_or(0.0);
    let max = sorted.last().copied().unwrap_or(0.0);

    SequenceStats {
        count,
        mean,
        median,
        std_dev,
        min,
        max,
    }
}
```

#### CRUD Operations (Rust)

```rust
// src-tauri/src/data_library/commands.rs
use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use crate::data_library::database::Database;
use crate::data_library::models::*;
use crate::data_library::stats::calculate_stats;

#[tauri::command]
pub async fn get_all_sequences(db: State<'_, Database>) -> Result<Vec<DataSequence>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT * FROM sequences ORDER BY is_pinned DESC, modified_at DESC"
    ).map_err(|e| e.to_string())?;
    
    let sequences = stmt.query_map([], |row| {
        Ok(DataSequence {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            values: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
            x_values: row.get::<_, Option<String>>(4)?
                .map(|s| serde_json::from_str(&s).unwrap()),
            uncertainties: row.get::<_, Option<String>>(5)?
                .map(|s| serde_json::from_str(&s).unwrap()),
            tags: serde_json::from_str(&row.get::<_, String>(6)?).unwrap(),
            unit: row.get(7)?,
            x_unit: row.get(8)?,
            source: SequenceSource {
                spreadsheet_id: row.get(9)?,
                spreadsheet_name: row.get(10)?,
                range: row.get(11)?,
                formula: row.get(12)?,
            },
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?)
                .unwrap().with_timezone(&Utc),
            modified_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(14)?)
                .unwrap().with_timezone(&Utc),
            is_pinned: row.get::<_, i32>(15)? != 0,
            color: row.get(16)?,
            stats: SequenceStats {
                count: row.get::<_, i32>(17)? as usize,
                mean: row.get(18)?,
                median: row.get(19)?,
                std_dev: row.get(20)?,
                min: row.get(21)?,
                max: row.get(22)?,
            },
        })
    }).map_err(|e| e.to_string())?;
    
    sequences.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_sequence(
    db: State<'_, Database>,
    sequence_data: SaveSequenceInput,
) -> Result<DataSequence, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let stats = calculate_stats(&sequence_data.values);
    
    let sequence = DataSequence {
        id: id.clone(),
        name: sequence_data.name,
        description: sequence_data.description,
        values: sequence_data.values,
        x_values: sequence_data.x_values,
        uncertainties: sequence_data.uncertainties,
        tags: sequence_data.tags,
        unit: sequence_data.unit,
        x_unit: sequence_data.x_unit,
        source: sequence_data.source,
        created_at: now,
        modified_at: now,
        is_pinned: false,
        color: None,
        stats,
    };
    
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO sequences VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &sequence.id,
            &sequence.name,
            &sequence.description,
            &serde_json::to_string(&sequence.values).unwrap(),
            &sequence.x_values.as_ref().map(|v| serde_json::to_string(v).unwrap()),
            &sequence.uncertainties.as_ref().map(|v| serde_json::to_string(v).unwrap()),
            &serde_json::to_string(&sequence.tags).unwrap(),
            &sequence.unit,
            &sequence.x_unit,
            &sequence.source.spreadsheet_id,
            &sequence.source.spreadsheet_name,
            &sequence.source.range,
            &sequence.source.formula,
            &sequence.created_at.to_rfc3339(),
            &sequence.modified_at.to_rfc3339(),
            &(sequence.is_pinned as i32),
            &sequence.color,
            &(sequence.stats.count as i32),
            &sequence.stats.mean,
            &sequence.stats.median,
            &sequence.stats.std_dev,
            &sequence.stats.min,
            &sequence.stats.max,
        ],
    ).map_err(|e| e.to_string())?;
    
    // Update FTS index
    conn.execute(
        "INSERT INTO sequences_fts VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![
            &sequence.id,
            &sequence.name,
            &sequence.description.as_ref().unwrap_or(&String::new()),
            &sequence.tags.join(" "),
            &sequence.source.spreadsheet_name,
        ],
    ).map_err(|e| e.to_string())?;
    
    Ok(sequence)
}

#[tauri::command]
pub async fn update_sequence(
    db: State<'_, Database>,
    id: String,
    updates: UpdateSequenceInput,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now();
    
    conn.execute(
        "UPDATE sequences SET 
            name = COALESCE(?, name),
            description = COALESCE(?, description),
            tags = COALESCE(?, tags),
            unit = COALESCE(?, unit),
            is_pinned = COALESCE(?, is_pinned),
            color = COALESCE(?, color),
            modified_at = ?
        WHERE id = ?",
        rusqlite::params![
            &updates.name,
            &updates.description,
            &updates.tags.as_ref().map(|t| serde_json::to_string(t).unwrap()),
            &updates.unit,
            &updates.is_pinned.map(|b| b as i32),
            &updates.color,
            &now.to_rfc3339(),
            &id,
        ],
    ).map_err(|e| e.to_string())?;
    
    // Update FTS index if name/description/tags changed
    if updates.name.is_some() || updates.description.is_some() || updates.tags.is_some() {
        // Fetch updated data
        let mut stmt = conn.prepare(
            "SELECT name, description, tags, source_spreadsheet_name FROM sequences WHERE id = ?"
        ).map_err(|e| e.to_string())?;
        
        let (name, desc, tags, source_name): (String, Option<String>, String, String) = 
            stmt.query_row([&id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                ))
            }).map_err(|e| e.to_string())?;
        
        let tags_vec: Vec<String> = serde_json::from_str(&tags).unwrap();
        
        conn.execute(
            "UPDATE sequences_fts SET name = ?, description = ?, tags = ?, source_spreadsheet_name = ? WHERE id = ?",
            rusqlite::params![
                &name,
                &desc.unwrap_or_default(),
                &tags_vec.join(" "),
                &source_name,
                &id,
            ],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn delete_sequence(db: State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM sequences WHERE id = ?", [&id])
        .map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM sequences_fts WHERE id = ?", [&id])
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct SaveSequenceInput {
    pub name: String,
    pub description: Option<String>,
    pub values: Vec<f64>,
    pub x_values: Option<Vec<f64>>,
    pub uncertainties: Option<Vec<f64>>,
    pub tags: Vec<String>,
    pub unit: Option<String>,
    pub x_unit: Option<String>,
    pub source: SequenceSource,
}

#[derive(serde::Deserialize)]
pub struct UpdateSequenceInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub unit: Option<String>,
    pub is_pinned: Option<bool>,
    pub color: Option<String>,
}
```

#### Search and Filter Commands (Rust)

```rust
// src-tauri/src/data_library/search.rs
use tauri::State;
use crate::data_library::database::Database;
use crate::data_library::models::DataSequence;

#[tauri::command]
pub async fn search_sequences(
    db: State<'_, Database>,
    query: String,
) -> Result<Vec<DataSequence>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    // Use FTS5 for full-text search
    let mut stmt = conn.prepare(
        "SELECT s.* FROM sequences s
         JOIN sequences_fts fts ON s.id = fts.id
         WHERE sequences_fts MATCH ?
         ORDER BY s.is_pinned DESC, s.modified_at DESC"
    ).map_err(|e| e.to_string())?;
    
    let sequences = stmt.query_map([&query], |row| {
        // Same mapping as get_all_sequences
        // ... (omitted for brevity)
        Ok(DataSequence { /* ... */ })
    }).map_err(|e| e.to_string())?;
    
    sequences.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn filter_sequences_by_tags(
    db: State<'_, Database>,
    tags: Vec<String>,
) -> Result<Vec<DataSequence>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT * FROM sequences 
         WHERE tags LIKE ?
         ORDER BY is_pinned DESC, modified_at DESC"
    ).map_err(|e| e.to_string())?;
    
    // Build LIKE pattern for tag matching
    let tag_patterns: Vec<String> = tags.iter()
        .map(|t| format!("%\"{}\"", t))
        .collect();
    
    let pattern = tag_patterns.join("%");
    
    let sequences = stmt.query_map([&pattern], |row| {
        // Same mapping
        Ok(DataSequence { /* ... */ })
    }).map_err(|e| e.to_string())?;
    
    sequences.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sort_sequences(
    db: State<'_, Database>,
    sort_by: String,
) -> Result<Vec<DataSequence>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let order_clause = match sort_by.as_str() {
        "name_asc" => "name ASC",
        "name_desc" => "name DESC",
        "created_asc" => "created_at ASC",
        "created_desc" => "created_at DESC",
        "modified_asc" => "modified_at ASC",
        "modified_desc" => "modified_at DESC",
        "size_asc" => "stats_count ASC",
        "size_desc" => "stats_count DESC",
        _ => "modified_at DESC", // default
    };
    
    let query = format!(
        "SELECT * FROM sequences ORDER BY is_pinned DESC, {}",
        order_clause
    );
    
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    
    let sequences = stmt.query_map([], |row| {
        // Same mapping
        Ok(DataSequence { /* ... */ })
    }).map_err(|e| e.to_string())?;
    
    sequences.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
```

#### Export/Import Commands

```rust
// src-tauri/src/data_library/export.rs
use std::fs::File;
use std::io::Write;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct DataSequence {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub values: Vec<f64>,
    pub x_values: Option<Vec<f64>>,
    pub uncertainties: Option<Vec<f64>>,
    pub tags: Vec<String>,
    pub unit: Option<String>,
    // ... other fields
}

#[tauri::command]
pub async fn export_sequence_json(sequence: DataSequence, path: String) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&sequence)
        .map_err(|e| e.to_string())?;
    
    let mut file = File::create(path)
        .map_err(|e| e.to_string())?;
    
    file.write_all(json.as_bytes())
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn export_sequence_csv(sequence: DataSequence, path: String) -> Result<(), String> {
    use csv::Writer;
    
    let mut wtr = Writer::from_path(path)
        .map_err(|e| e.to_string())?;
    
    // Write header
    if sequence.x_values.is_some() {
        wtr.write_record(&["X", "Y", "Uncertainty"])
            .map_err(|e| e.to_string())?;
    } else {
        wtr.write_record(&["Value", "Uncertainty"])
            .map_err(|e| e.to_string())?;
    }
    
    // Write data
    for (i, &value) in sequence.values.iter().enumerate() {
        if let Some(ref x_vals) = sequence.x_values {
            let x = x_vals.get(i).map(|v| v.to_string()).unwrap_or_default();
            let unc = sequence.uncertainties.as_ref()
                .and_then(|u| u.get(i))
                .map(|v| v.to_string())
                .unwrap_or_default();
            wtr.write_record(&[x, value.to_string(), unc])
                .map_err(|e| e.to_string())?;
        } else {
            let unc = sequence.uncertainties.as_ref()
                .and_then(|u| u.get(i))
                .map(|v| v.to_string())
                .unwrap_or_default();
            wtr.write_record(&[value.to_string(), unc])
                .map_err(|e| e.to_string())?;
        }
    }
    
    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn import_sequence_json(path: String) -> Result<DataSequence, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| e.to_string())?;
    
    let sequence: DataSequence = serde_json::from_str(&content)
        .map_err(|e| e.to_string())?;
    
    Ok(sequence)
}
```

---

## Data Flow

### Saving Data from Spreadsheet
```
User selects range in spreadsheet
        ↓
Clicks "Data Library" button
        ↓
Data Library window opens
        ↓
"Save Selection" dialog pre-filled
        ↓
User edits name/tags/description
        ↓
Click "Save"
        ↓
Frontend: invoke('save_sequence')
        ↓
Rust Backend: Calculate stats
        ↓
SQLite: INSERT INTO sequences
        ↓
Return DataSequence to frontend
        ↓
Update sequences list in UI
        ↓
Show confirmation toast
```

### Loading Data to Spreadsheet
```
User views sequence in Data Library
        ↓
Click "[Load to Spreadsheet]"
        ↓
Dialog: Select target tab + range
        ↓
Click "Load"
        ↓
Backend: Emit event to main window
        ↓
Main window: Receive event with data
        ↓
Active tab: Write data to Univer
        ↓
Show confirmation toast
```

### Search & Filter
```
User types in search box
        ↓
useSequenceSearch hook (debounced)
        ↓
Filter sequences by query
        ↓
Apply tag filters (if any)
        ↓
Sort filtered results
        ↓
Update UI with filtered list
```

---

## Success Criteria

### Functional Requirements
- ✅ Save data sequences from spreadsheet selections
- ✅ Search sequences by name, tags, description, source
- ✅ Filter sequences by tags (multi-select)
- ✅ Sort sequences by various criteria
- ✅ View detailed sequence information with preview
- ✅ Edit sequence metadata (name, tags, description, unit)
- ✅ Delete sequences with confirmation
- ✅ Pin/unpin sequences to top
- ✅ Load sequences back to spreadsheet
- ✅ Export sequences (JSON, CSV, Excel)
- ✅ Import sequences from files
- ✅ Batch operations (tag, export, delete)

### Visual Requirements
- ✅ Clean, organized layout with sidebar navigation
- ✅ Responsive grid layout for sequence cards
- ✅ Clear visual hierarchy (pinned → all → recent)
- ✅ Readable statistics and data preview
- ✅ Smooth animations and transitions
- ✅ Consistent with main app design language

### Performance Requirements
- ✅ Search results update within 100ms (FTS5)
- ✅ Load 1000+ sequences without lag (<500ms)
- ✅ Smooth scrolling in list view
- ✅ Fast SQLite queries (<50ms average)
- ✅ Efficient re-rendering (React.memo, useMemo)
- ✅ Minimal data serialization overhead

### UX Requirements
- ✅ Intuitive navigation and organization
- ✅ Clear feedback for all actions
- ✅ Undo support for delete operations (trash/archive)
- ✅ Keyboard shortcuts for common actions
- ✅ Accessible (ARIA labels, keyboard navigation)
- ✅ Error handling with user-friendly messages


---

## Future Enhancements

### Phase 2
- Data sequence versioning (track changes)
- Collaborative sharing (export/import with permissions)
- Cloud sync (optional backend integration)
- Advanced filtering (date ranges, value ranges, statistical criteria)

### Phase 3
- Data transformations (normalize, scale, offset)
- Sequence arithmetic (add, subtract, multiply sequences)
- Automatic outlier detection and highlighting
- Data quality scoring

### Phase 4
- Machine learning on sequences (clustering, anomaly detection)
- Automatic tagging suggestions (based on source, patterns)
- Integration with external databases
- Real-time collaboration

---

## Related Components

1. **[Main Toolbar](./00_main_toolbar.md)** - Opens Data Library window
2. **[Export System](./07_export_system.md)** - Exports data to library
3. **[Graphs & Fitting Tab](./09_graphs_and_fitting_tab.md)** - Uses sequences for plotting
4. **[Monte Carlo Tab](#)** - Uses sequences for simulations
5. **[Solver Tab](#)** - Uses sequences as constraints

---

## Dependencies

### NPM Packages
```bash
npm install plotly.js              # Mini preview charts (UI only)
npm install react-plotly.js        # React bindings
npm install @mui/material          # UI components
npm install @mui/icons-material    # Icons
npm install date-fns               # Date formatting (UI only)
```

### Rust Crates
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
csv = "1.3"
tauri = { version = "2.8", features = ["dialog-all"] }
```

---

## Implementation Notes

**Rust-First Architecture**:
- All data stored in SQLite database (Rust manages database)
- All business logic in Rust (CRUD, search, filter, sort, statistics)
- TypeScript only renders UI and handles user interactions
- No IndexedDB - all persistence via Tauri commands to Rust backend
- Statistics calculated in Rust using proper algorithms
- Search uses SQLite FTS5 (Full-Text Search) for performance
- Tag filtering uses SQL queries
- Each data sequence is independent (no links to original spreadsheet data)
- Export/import handled entirely in Rust
- Window can stay open while working in main app (non-modal)

**Performance Benefits**:
- SQLite much faster than IndexedDB for complex queries
- FTS5 provides instant full-text search
- Statistics calculated once on save (Rust) and cached in DB
- Sorting done in SQL (faster than JavaScript)
- Less data serialization between frontend/backend

**Database Location**:
- SQLite file stored in app data directory
- Path: `~/.local/share/anafis/data_library.db` (Linux)
- Cross-platform using Tauri's `app_data_dir()`

---

## Testing Checklist

### Unit Tests (Rust)
- [ ] SQLite database operations (create, read, update, delete)
- [ ] Statistics calculations (mean, median, std dev, min, max)
- [ ] FTS5 search functionality
- [ ] Tag filtering queries
- [ ] Sorting algorithms
- [ ] Data validation (sequence structure, value ranges)
- [ ] Export/import functions (JSON, CSV)

### Unit Tests (TypeScript)
- [ ] Component rendering
- [ ] User input handling
- [ ] Tauri command invocation
- [ ] Error display

### Integration Tests
- [ ] Save from spreadsheet → Load to spreadsheet (round-trip)
- [ ] Export to file → Import from file
- [ ] Window communication (main ↔ data library)
- [ ] Concurrent operations (multiple saves/deletes)
- [ ] Database migrations
- [ ] Large dataset handling (10,000+ sequences)

### UI Tests
- [ ] Search responsiveness
- [ ] Tag filtering
- [ ] Sequence card interactions
- [ ] Detail view navigation
- [ ] Dialog flows (save, edit, load, delete)
- [ ] Keyboard shortcuts

### Performance Tests
- [ ] Load 1000+ sequences (<500ms)
- [ ] Search with large dataset (<100ms)
- [ ] Batch operations on many sequences
- [ ] Memory usage monitoring
- [ ] SQLite query performance
- [ ] Database size optimization

---

This specification provides a comprehensive foundation for implementing the Data Library window as the core infrastructure for persistent data management in AnaFis, with all logic in Rust and TypeScript handling only UI rendering.
