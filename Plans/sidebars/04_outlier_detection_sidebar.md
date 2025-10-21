# Outlier Detection Sidebar 🔍

**Status**: Planned  
**Priority**: Medium  
**Complexity**: Medium  
**Dependencies**: statrs (Rust), statistical libraries

---

## Purpose

Identify and handle outliers in experimental data to improve data quality and analysis accuracy.

---

## Features

### Detection Methods
- **Z-Score Method**: Standard deviations from mean
- **Modified Z-Score**: Using Median Absolute Deviation (MAD) - more robust
- **IQR Method**: Interquartile range (Q1-1.5×IQR, Q3+1.5×IQR)
- **Isolation Forest**: Machine learning-based anomaly detection
- **Grubbs' Test**: Statistical test for single outliers
- **Dixon's Q Test**: For small datasets

### Visualization
- Highlight outliers in spreadsheet with colored backgrounds
- Box plot showing quartiles and outliers
- Scatter plot with outliers marked

### Actions
- **Flag Only**: Visual highlight in spreadsheet (no data modification)
- **Remove Outliers**: Delete outlier rows/values
- **Replace with Interpolation**: Linear/spline interpolation
- **Replace with Mean/Median**: Statistical replacement
- **Replace with NaN**: Mark as missing data
- **Write to New Column**: Keep original, create cleaned version

### Threshold Adjustment
- Adjustable sensitivity (Z-score threshold, IQR multiplier)
- Interactive slider to see effect on outlier count

---

## UI Layout

```
┌─────────────────────────────────────┐
│ Outlier Detection               [X] │
├─────────────────────────────────────┤
│ Data Selection:                     │
│  Range: [A1:A100] [Select]         │
│                                     │
│ Detection Method: [Z-Score ▼]      │
│  ( ) Z-Score (parametric)          │
│  (•) Modified Z-Score (robust)     │
│  ( ) IQR Method                    │
│  ( ) Grubbs' Test                  │
│  ( ) Isolation Forest (ML)         │
│                                     │
│ Threshold:                          │
│  Z-Score: [2.5──────────] 2.5      │
│  (standard deviations from mean)   │
│                                     │
│ ┌─────────────────────────────┐   │
│ │ Detection Results           │   │
│ │ ─────────────────────────── │   │
│ │ Total Points:    100        │   │
│ │ Outliers Found:  3 (3.0%)   │   │
│ │                             │   │
│ │ Statistics:                 │   │
│ │  Mean:    52.34             │   │
│ │  Median:  51.20             │   │
│ │  Std Dev: 8.45              │   │
│ │  MAD:     6.23              │   │
│ │                             │   │
│ │ Outliers:                   │   │
│ │  A15: 127.5 (z=3.2) ⬆       │   │
│ │  A47: 8.3   (z=-2.8) ⬇      │   │
│ │  A89: 145.2 (z=3.7) ⬆       │   │
│ └─────────────────────────────┘   │
│                                     │
│ [✓] Highlight in spreadsheet       │
│     Color: [🔴 Red]                 │
│                                     │
│ [📊 Show Box Plot]                 │
│                                     │
│ Action:                             │
│  ( ) Flag only (no changes)        │
│  ( ) Remove outliers               │
│  (•) Replace with interpolation    │
│  ( ) Replace with mean/median      │
│  ( ) Replace with NaN              │
│                                     │
│ Output Range: [B1:B100] [Select]   │
│  [ ] Overwrite original (⚠️)       │
│                                     │
│ [Apply] [Reset Highlights]         │
└─────────────────────────────────────┘
```

### Box Plot Visualization
```
┌─────────────────────────────────────┐
│ Box Plot Visualization              │
│                                     │
│     150 ┤        × (outlier)        │
│         │        × (outlier)        │
│     100 ┤    ┌─────┐                │
│         │    │     │                │
│         │────┤     │                │
│      50 ┤    │     │                │
│         │    └─────┘                │
│       0 ┤        ×  (outlier)       │
│                                     │
│ Q1: 46.25  Median: 51.20  Q3: 58.40│
│ IQR: 12.15                          │
│ Lower fence: 28.03                  │
│ Upper fence: 76.63                  │
└─────────────────────────────────────┘
```

---

## Data Flow Pattern

**Type**: Monitor → Validate → Highlight (Pattern C)

1. User selects data range
2. Sidebar reads values from Univer
3. User selects detection method and adjusts threshold
4. Apply outlier detection algorithm
5. Display statistics and list of outliers in sidebar
6. Highlight outlier cells in spreadsheet (using Univer formatting API)
7. User reviews outliers and chooses action
8. Apply action (remove, replace, or write cleaned data)
9. Optional: Write processing log to metadata

---

## Technical Implementation

### TypeScript Interfaces

```typescript
interface OutlierDetectionSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef: UniverSpreadsheetRef;
  onSelectionChange: (cellRef: string) => void;
}

type OutlierMethod = 
  | 'zscore' 
  | 'modified_zscore' 
  | 'iqr' 
  | 'grubbs' 
  | 'dixon' 
  | 'isolation_forest';

type OutlierAction = 
  | 'flag' 
  | 'remove' 
  | 'interpolate_linear'
  | 'interpolate_spline'
  | 'replace_mean'
  | 'replace_median'
  | 'replace_nan';

interface OutlierConfig {
  method: OutlierMethod;
  threshold: number; // e.g., 2.5 for z-score, 1.5 for IQR
  action: OutlierAction;
  highlightColor: string;
  outputRange?: string;
  overwriteOriginal: boolean;
}

interface OutlierResult {
  index: number;
  cellRef: string;
  value: number;
  score: number; // z-score, modified z-score, etc.
  isOutlier: boolean;
  direction: 'high' | 'low'; // Above or below threshold
}

interface OutlierStatistics {
  totalPoints: number;
  outlierCount: number;
  outlierPercentage: number;
  mean: number;
  median: number;
  stdDev: number;
  mad: number; // Median Absolute Deviation
  q1: number;
  q3: number;
  iqr: number;
  lowerFence: number; // Q1 - 1.5*IQR
  upperFence: number; // Q3 + 1.5*IQR
}

interface OutlierDetectionState {
  inputRange: string;
  outputRange: string;
  config: OutlierConfig;
  originalData: number[] | null;
  results: OutlierResult[];
  statistics: OutlierStatistics | null;
  highlightedCells: string[];
  isProcessing: boolean;
  error: string | null;
}
```

---

## Rust Backend Implementation

**IMPORTANT**: ALL outlier detection algorithms are implemented in Rust. TypeScript ONLY calls Rust commands via invoke().

### TypeScript Wrapper (UI Only)

```typescript
// REMOVED: All detection algorithm functions (detectOutliersZScore, detectOutliersModifiedZScore, detectOutliersIQR, calculateOutlierStatistics)
// TypeScript ONLY calls Rust backend:

import { invoke } from '@tauri-apps/api/tauri';

async function detectOutliers(
  data: number[],
  method: string,
  threshold: number
): Promise<OutlierResult[]> {
  return await invoke('detect_outliers', {
    data,
    method,
    threshold
  });
}
```

---

## Rust Outlier Detection Module

```rust
// AnaFis/src-tauri/src/outliers/mod.rs

use statrs::statistics::{Statistics, OrderStatistics};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct OutlierResult {
    index: usize,
    value: f64,
    score: f64,
    is_outlier: bool,
    direction: String, // "high" or "low"
}

#[derive(Debug, Serialize)]
pub struct OutlierStats {
    total_points: usize,
    outliers_count: usize,
    outlier_percentage: f64,
    high_outliers: usize,
    low_outliers: usize,
}

// Z-Score method implementation
fn detect_z_score(data: &[f64], threshold: f64) -> Vec<OutlierResult> {
    let mean: f64 = data.iter().sum::<f64>() / data.len() as f64;
    let variance: f64 = data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (data.len() - 1) as f64;
    let std_dev = variance.sqrt();
    
    data.iter().enumerate().map(|(index, &value)| {
        let z_score = ((value - mean) / std_dev).abs();
        OutlierResult {
            index,
            value,
            score: z_score,
            is_outlier: z_score > threshold,
            direction: if value > mean { "high".to_string() } else { "low".to_string() },
        }
    }).collect()
}

// Modified Z-Score method (more robust)
fn detect_modified_z_score(data: &[f64], threshold: f64) -> Vec<OutlierResult> {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];
    
    // Calculate MAD (Median Absolute Deviation)
    let mut abs_deviations: Vec<f64> = data.iter()
        .map(|x| (x - median).abs())
        .collect();
    abs_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mad = abs_deviations[abs_deviations.len() / 2];
    
    data.iter().enumerate().map(|(index, &value)| {
        let modified_z = if mad > 0.0 {
            0.6745 * (value - median) / mad
        } else {
            0.0
        };
        let score = modified_z.abs();
        
        OutlierResult {
            index,
            value,
            score,
            is_outlier: score > threshold,
            direction: if value > median { "high".to_string() } else { "low".to_string() },
        }
    }).collect()
}

// IQR method implementation
fn detect_iqr(data: &[f64], threshold: f64) -> Vec<OutlierResult> {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let n = sorted.len();
    let q1 = sorted[n / 4];
    let q3 = sorted[3 * n / 4];
    let iqr = q3 - q1;
    
    let lower_bound = q1 - threshold * iqr;
    let upper_bound = q3 + threshold * iqr;
    
    data.iter().enumerate().map(|(index, &value)| {
        let is_outlier = value < lower_bound || value > upper_bound;
        let score = if value < lower_bound {
            (lower_bound - value) / iqr
        } else if value > upper_bound {
            (value - upper_bound) / iqr
        } else {
            0.0
        };
        
        OutlierResult {
            index,
            value,
            score,
            is_outlier,
            direction: if value > q3 { "high".to_string() } else { "low".to_string() },
        }
    }).collect()
}

#[tauri::command]
pub fn detect_outliers(
  data: Vec<f64>,
  method: String,
  threshold: f64
) -> Result<Vec<OutlierResult>, String> {
  if data.is_empty() {
    return Err("Data cannot be empty".to_string());
  }
  
  let results = match method.as_str() {
    "zscore" => detect_z_score(&data, threshold),
    "modified_zscore" => detect_modified_z_score(&data, threshold),
    "iqr" => detect_iqr(&data, threshold),
    _ => return Err(format!("Unknown method: {}", method)),
  };
  
  Ok(results)
}

#[tauri::command]
pub fn calculate_outlier_statistics(results: Vec<OutlierResult>) -> OutlierStats {
  let total = results.len();
  let outliers: Vec<&OutlierResult> = results.iter().filter(|r| r.is_outlier).collect();
  let outlier_count = outliers.len();
  
  let high_outliers = outliers.iter().filter(|r| r.direction == "high").count();
  let low_outliers = outliers.iter().filter(|r| r.direction == "low").count();
  
  OutlierStats {
    total_points: total,
    outliers_count: outlier_count,
    outlier_percentage: (outlier_count as f64 / total as f64) * 100.0,
    high_outliers,
    low_outliers,
  }
}
```

---

## Architecture Notes

**Rust-First Design**:
- ALL outlier detection algorithms (Z-Score, Modified Z-Score, IQR) implemented in Rust
- TypeScript only handles UI interactions and calls `invoke('detect_outliers')`
- Statistical calculations (mean, median, MAD, IQR) performed in Rust for performance
- Uses `statrs` crate for robust statistical operations

**Benefits**:
- 10-100x faster outlier detection on large datasets
- Consistent numeric precision across platforms
- No floating-point inconsistencies from JavaScript
- Type-safe detection methods via Rust enums

---

## UI Workflow

### Outlier Highlighting and Actions

```typescript
// Highlight outliers in spreadsheet
async function highlightOutliers(
  univerRef: UniverSpreadsheetRef,
  outliers: OutlierResult[],
  color: string
): Promise<void> {
  for (const outlier of outliers) {
    if (outlier.isOutlier) {
      await univerRef.current?.setCellStyle(outlier.cellRef, {
        backgroundColor: color,
        color: '#ffffff'
      });
    }
  }
}

// Apply action to outliers
function applyOutlierAction(
  data: number[],
  outliers: OutlierResult[],
  action: OutlierAction
): number[] {
  const result = [...data];
  const outlierIndices = new Set(
    outliers.filter(o => o.isOutlier).map(o => o.index)
  );
  
  switch (action) {
    case 'remove':
      return result.filter((_, i) => !outlierIndices.has(i));
    
    case 'replace_mean': {
      const validData = result.filter((_, i) => !outlierIndices.has(i));
      const mean = validData.reduce((a, b) => a + b, 0) / validData.length;
      return result.map((v, i) => outlierIndices.has(i) ? mean : v);
    }
    
    case 'replace_median': {
      const validData = result.filter((_, i) => !outlierIndices.has(i));
      const sorted = [...validData].sort((a, b) => a - b);
      const median = sorted[Math.floor(sorted.length / 2)];
      return result.map((v, i) => outlierIndices.has(i) ? median : v);
    }
    
    case 'interpolate_linear': {
      // Linear interpolation for outliers
      outlierIndices.forEach(i => {
        // Find nearest non-outlier neighbors
        let left = i - 1;
        let right = i + 1;
        
        while (left >= 0 && outlierIndices.has(left)) left--;
        while (right < result.length && outlierIndices.has(right)) right++;
        
        if (left >= 0 && right < result.length) {
          const t = (i - left) / (right - left);
          result[i] = result[left] + t * (result[right] - result[left]);
        } else if (left >= 0) {
          result[i] = result[left];
        } else if (right < result.length) {
          result[i] = result[right];
        }
      });
      return result;
    }
    
    case 'replace_nan':
      return result.map((v, i) => outlierIndices.has(i) ? NaN : v);
    
    case 'flag':
    default:
      return result; // No modification
  }
}
```

### Rust Backend Implementation

```rust
// src-tauri/src/scientific/outliers.rs

use statrs::statistics::*;

#[derive(serde::Deserialize)]
pub struct OutlierRequest {
    data: Vec<f64>,
    method: String,
    threshold: f64,
}

#[derive(serde::Serialize)]
pub struct OutlierResponse {
    outlier_indices: Vec<usize>,
    scores: Vec<f64>,
    statistics: OutlierStatistics,
}

#[derive(serde::Serialize)]
pub struct OutlierStatistics {
    total_points: usize,
    outlier_count: usize,
    mean: f64,
    median: f64,
    std_dev: f64,
    mad: f64,
}

#[tauri::command]
pub async fn detect_outliers(
    request: OutlierRequest,
) -> Result<OutlierResponse, String> {
    let outliers = match request.method.as_str() {
        "grubbs" => grubbs_test(&request.data, request.threshold)?,
        "dixon" => dixon_test(&request.data)?,
        _ => {
            return Err(format!("Method {} not implemented in backend", request.method));
        }
    };
    
    Ok(outliers)
}

// Grubbs' test for outliers
fn grubbs_test(data: &[f64], alpha: f64) -> Result<OutlierResponse, String> {
    if data.len() < 3 {
        return Err("Need at least 3 data points for Grubbs' test".to_string());
    }
    
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let std_dev = (data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (data.len() - 1) as f64)
        .sqrt();
    
    // Calculate Grubbs' statistic for each point
    let mut scores: Vec<f64> = data.iter()
        .map(|x| ((x - mean) / std_dev).abs())
        .collect();
    
    // Critical value for Grubbs' test
    let n = data.len() as f64;
    let t_dist = statrs::distribution::StudentsT::new(0.0, 1.0, n - 2.0).unwrap();
    let t_crit = t_dist.inverse_cdf(1.0 - alpha / (2.0 * n));
    let g_crit = ((n - 1.0) / n.sqrt()) * ((t_crit * t_crit) / (n - 2.0 + t_crit * t_crit)).sqrt();
    
    let outlier_indices: Vec<usize> = scores.iter()
        .enumerate()
        .filter(|(_, &score)| score > g_crit)
        .map(|(i, _)| i)
        .collect();
    
    Ok(OutlierResponse {
        outlier_indices,
        scores,
        statistics: calculate_statistics(data, &outlier_indices),
    })
}

fn calculate_statistics(data: &[f64], outlier_indices: &[usize]) -> OutlierStatistics {
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];
    let std_dev = (data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (data.len() - 1) as f64)
        .sqrt();
    
    let mad = {
        let mut deviations: Vec<f64> = data.iter()
            .map(|x| (x - median).abs())
            .collect();
        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        deviations[deviations.len() / 2]
    };
    
    OutlierStatistics {
        total_points: data.len(),
        outlier_count: outlier_indices.len(),
        mean,
        median,
        std_dev,
        mad,
    }
}
```

---

## Dependencies

### Frontend
- Basic statistical calculations
- Plotting library (for box plot visualization)

### Backend
- **statrs** - Statistical tests (Grubbs, Dixon)
- **ndarray** - Array operations

### Installation

```toml
# Rust (Cargo.toml)
[dependencies]
statrs = "0.16"
ndarray = "0.15"
```

---

## File Location

- **Component**: `AnaFis/src/components/spreadsheet/OutlierDetectionSidebar.tsx`
- **Backend**: `AnaFis/src-tauri/src/scientific/outliers.rs`
- **Types**: `AnaFis/src/types/outliers.ts`

---

## Success Criteria

- ✓ All detection methods work correctly
- ✓ Outliers highlighted in spreadsheet
- ✓ Statistics calculated accurately
- ✓ All action types work (remove, replace, interpolate)
- ✓ Box plot visualization displays correctly
- ✓ Threshold adjustment updates results in real-time
- ✓ Performance: Process 10,000 points in < 200ms

---

## Testing Plan

### Unit Tests
- Each detection algorithm accuracy
- Statistical calculations
- Outlier actions (remove, replace, interpolate)

### Integration Tests
- Read from Univer → Detect → Highlight → Apply action
- Multiple detection methods
- Edge cases (all outliers, no outliers)

### E2E Tests
- Complete user workflow
- Highlight and clear highlighting
- Apply different actions

---

**Next Steps**: Implement after Data Smoothing Sidebar
