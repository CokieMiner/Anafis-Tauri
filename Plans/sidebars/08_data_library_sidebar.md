# Data Library Sidebar 📚

**Status**: Planned  
**Priority**: CRITICAL (Core Infrastructure)  
**Complexity**: Medium  
**Dependencies**: Univer API, Data Library Window (Rust SQLite backend)

---

## Purpose

Quick export sidebar for saving measurement sequences FROM the spreadsheet TO the Data Library. Provides streamlined interface for capturing data with metadata, tags, and uncertainties. Uses same SQLite backend as Data Library Window.

**Architecture Note**: This sidebar focuses on SAVING data to library. For MANAGING stored sequences (browse, search, edit, delete), use the Data Library Window (see `10_data_library_window.md`). Each tab has its own import mechanism to load FROM library.

---

## Features

### Data Export from Spreadsheet (PRIMARY FOCUS)
- Select range from current sheet
- Name and describe the sequence
- Tag for organization
- Specify unit
- Include optional uncertainty column
- Preview statistics before saving
- **One-click save to SQLite database**

### Quick Access (Optional)
- View recently saved sequences
- Quick re-export confirmation
- Open Data Library Window for full management

### Backend Integration
- All data saved via Rust `save_sequence` command
- Statistics calculated in Rust before storage
- Persistent SQLite storage (shared with Data Library Window)
- Full-text search capabilities via FTS5

---

## UI Layout

**Simplified Export-Focused Interface:**

```
┌─────────────────────────────────────────┐
│ Data Library - Export              [X] │
├─────────────────────────────────────────┤
│ 📤 SAVE TO LIBRARY                      │
│                                         │
│ Sheet: [Sheet1 ▼]                      │
│ Range: [A1:A100_____] [Select]         │
│                                         │
│ Name: [Temperature Sensor A_____]      │
│                                         │
│ Description:                            │
│ ┌─────────────────────────────────┐   │
│ │ Calibration run after           │   │
│ │ maintenance on 2024-10-08       │   │
│ └─────────────────────────────────┘   │
│                                         │
│ Tags (comma-separated):                 │
│ [calibration, temp, sensor_a____]      │
│ [+ Common Tags ▼]                      │
│   • experiment_1                       │
│   • calibration                        │
│   • raw_data                           │
│                                         │
│ Unit: [°C___] [K___] [custom____]      │
│                                         │
│ Uncertainty Range (optional):          │
│ [B1:B100_____] [Select]                │
│ Type: [Absolute ▼] ±σ                  │
│                                         │
│ Preview (calculated in Rust):           │
│ ┌─────────────────────────────────┐   │
│ │ Points: 100                     │   │
│ │ Range: 23.5 - 35.2              │   │
│ │ Mean: 28.45 ± 1.23              │   │
│ │ With uncertainties: Yes         │   │
│ └─────────────────────────────────┘   │
│                                         │
│ [☐ Pin to library]                     │
│ [💾 Save to Library]                   │
│                                         │
│ ─────────────────────────────────      │
│                                         │
│ 📚 Recently Saved (3 last):             │
│ • Temperature Sensor A (just now)      │
│ • Time Values (5 min ago)              │
│ • Pressure Data (1 hour ago)           │
│                                         │
│ [� Open Library Manager...]           │
│   (Opens Data Library Window for        │
│    browsing, searching, editing)        │
└─────────────────────────────────────────┘
```

**Note**: For full library management (browse all, search, edit, delete), click "Open Library Manager" to open the Data Library Window.
│ Include:                     │
│ [✓] Column header            │
│     (Temperature Sensor A)   │
│ [✓] Uncertainties            │
│     (in adjacent column)     │
│ [✓] Unit in header           │
│     (Temperature Sensor A °C)│
│ [✓] Metadata as comment      │
│     (on header cell)         │
│                              │
│ Preview:                     │
│  D1: Temperature Sensor A °C │
│  D2: 23.5    E2: 0.1        │
│  D3: 24.1    E3: 0.1        │
│  ...                         │
│                              │
│ [Load] [Cancel]              │
└──────────────────────────────┘
```

---

## Data Flow Pattern

**Type**: Bidirectional Storage/Retrieval

1. **Export Path:**
   - User selects range in spreadsheet
   - Opens Data Library sidebar
   - Fills metadata (name, tags, unit)
   - Optionally selects uncertainty range
   - Preview shows statistics
   - Click "Save to Library"
   - Data stored in IndexedDB

2. **Import Path:**
   - User browses/searches library
   - Clicks "Load" on sequence
   - Chooses target location
   - Data inserted into spreadsheet

3. **Cross-Tab Usage:**
   - Graphs & Fitting tab reads from library
   - Monte Carlo tab reads inputs from library
   - Solver tab can use library data

---

## Technical Implementation

### TypeScript Interfaces

```typescript
// AnaFis/src/types/dataLibrary.ts

interface DataSequence {
  id: string;                    // UUID
  version: number;               // For schema migrations
  
  // User-provided metadata
  name: string;                  // "Temperature Sensor A"
  description?: string;          // Free text
  tags: string[];                // ['calibration', 'temperature']
  unit?: string;                 // 'K', 'm/s', 'Pa', etc.
  
  // The actual data
  values: number[];              // Main measurements
  uncertainties?: number[];      // ± values (same length)
  uncertaintyType?: 'absolute' | 'relative' | 'percentage';
  
  // Source information
  source: {
    type: 'spreadsheet' | 'fitting' | 'monte_carlo' | 'solver' | 'import';
    sheetName?: string;
    range?: string;
    tabName: string;
    timestamp: Date;
  };
  
  // Lifecycle
  createdAt: Date;
  modifiedAt: Date;
  lastAccessedAt: Date;
  usageCount: number;
  isPinned: boolean;
  
  // Pre-computed statistics (cached)
  stats: {
    count: number;
    mean: number;
    median: number;
    stdDev: number;
    min: number;
    max: number;
    meanUncertainty?: number;  // If uncertainties present
  };
}

interface DataLibrarySidebarProps {
  open: boolean;
  onClose: () => void;
  univerAPI: UniverAPI;
  currentSheet: string;
}
```

---

## Architecture: Rust Backend Integration

**This sidebar uses the same SQLite/Rust backend as the Data Library Window.** All data operations are handled via Tauri commands (see `10_data_library_window.md` for complete backend specification).

### Key Rust Commands Used

```typescript
// Save sequence to library (invokes Rust)
await invoke('save_sequence', {
  sequence: {
    id: uuid(),
    name, description, tags, unit,
    values, uncertainties,
    source: { type: 'spreadsheet', sheetName, range, timestamp }
  }
});

// Get recently saved sequences (for display)
const recent = await invoke('get_all_sequences', { limit: 3, sortBy: 'created_desc' });

// Get all tags (for autocomplete)
const tags = await invoke('get_all_tags');
```

**Note**: Statistics are calculated automatically in Rust when saving. See `10_data_library_window.md` for complete Rust implementation details (database schema, commands, etc.).

---

## TypeScript Implementation

### Sidebar Component

```typescript
// AnaFis/src/components/spreadsheet/DataLibrarySidebar.tsx

import { invoke } from '@tauri-apps/api/tauri';
import { v4 as uuid } from 'uuid';

// Helper Functions

```typescript
// Export sequence from spreadsheet (calls Rust backend)
async function saveToLibrary(
  univerAPI: UniverAPI,
  sheetId: string,
  range: string,
  metadata: {
    name: string;
    description?: string;
    tags: string[];
    unit?: string;
    isPinned?: boolean;
  },
  uncertaintyRange?: string
): Promise<string> {
  // Extract values from spreadsheet
  const values = await univerAPI.getRange(sheetId, range);
  const flatValues = values.flat().map(v => parseFloat(v)).filter(v => !isNaN(v));
  
  // Extract uncertainties if provided
  let uncertainties: number[] | undefined;
  if (uncertaintyRange) {
    const uncValues = await univerAPI.getRange(sheetId, uncertaintyRange);
    uncertainties = uncValues.flat().map(v => Math.abs(parseFloat(v))).filter(v => !isNaN(v));
    
    // Validate lengths match
    if (uncertainties.length !== flatValues.length) {
      throw new Error(
        `Data and uncertainty ranges have different lengths: ` +
        `${flatValues.length} vs ${uncertainties.length}`
      );
    }
  }
  
  // Create sequence object
  const sheetName = await univerAPI.getSheetName(sheetId);
  const sequence = {
    id: uuid(),
    name: metadata.name,
    description: metadata.description || '',
    tags: metadata.tags,
    unit: metadata.unit || '',
    values: flatValues,
    uncertainties: uncertainties || [],
    source: {
      type: 'spreadsheet' as const,
      sheetName,
      range,
      tabName: 'Spreadsheet',
      timestamp: new Date().toISOString()
    },
    isPinned: metadata.isPinned || false
  };
  
  // Save via Rust backend (statistics calculated there)
  await invoke('save_sequence', { sequence });
  
  return sequence.id;
}

// Open Data Library Window for management
async function openLibraryWindow(): Promise<void> {
  await invoke('open_data_library_window');
}
```

---

## Dependencies

**Rust Dependencies** (same as Data Library Window):
- rusqlite 0.31 with bundled SQLite and FTS5
- uuid 1.6
- chrono 0.4
- statrs 0.16
- serde_json

**TypeScript Dependencies**:
- @tauri-apps/api (Tauri invoke)
- uuid (for generating IDs)
- React, Material-UI (UI components)

**Note**: NO IndexedDB or `idb` package needed - all persistence handled by Rust/SQLite.

---

## File Location

- **Component**: `AnaFis/src/components/spreadsheet/DataLibrarySidebar.tsx`
- **Rust Backend**: `AnaFis/src-tauri/src/data_library/` (shared with Window)
- **Types**: `AnaFis/src/types/dataLibrary.ts` (TypeScript interfaces matching Rust structs)

---

## Success Criteria

- ✓ Can export spreadsheet range to library with metadata
- ✓ Data persists across app restarts (SQLite)
- ✓ Statistics calculated automatically in Rust
- ✓ Recently saved sequences displayed
- ✓ Can open Data Library Window for full management
- ✓ Tag autocomplete works with existing tags
- ✓ Validation prevents empty names, invalid ranges
- ✓ **All business logic in Rust, TypeScript for UI only**
- ✓ Quick and streamlined for fast data capture from spreadsheet

---

## Architecture Notes

**Rust-First Design**:
- All data operations (save, calculate stats) in Rust
- TypeScript only handles: UI rendering, user input, calling Tauri invoke()
- Benefits: Data integrity, performance, type safety, shared with Window

**Separation of Concerns**:
- **This Sidebar**: Quick save FROM spreadsheet
- **Data Library Window (10)**: Browse, search, edit, delete stored data
- **Each Tab**: Has own import mechanism to load FROM library

---

**Next Steps**: Implement after Data Library Window backend is complete (shares same Rust module)
