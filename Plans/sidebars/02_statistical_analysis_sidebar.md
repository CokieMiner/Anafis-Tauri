# Statistical Analysis Sidebar 📈

**Status**: Planned  
**Priority**: High  
**Complexity**: Medium  
**Dependencies**: statrs (Rust), simple-statistics (optional frontend)

---

## Purpose

Calculate and display statistical measures for selected data ranges.

---

## Features

### Basic Statistics
- Mean
- Median
- Mode
- Standard Deviation
- Variance

### Range Statistics
- Minimum
- Maximum
- Range
- Q1 (First Quartile)
- Q3 (Third Quartile)
- IQR (Interquartile Range)

### Distribution Analysis
- Skewness
- Kurtosis
- Normality tests (Shapiro-Wilk, Kolmogorov-Smirnov)
- Histogram data

### Confidence Intervals
- 90% CI
- 95% CI
- 99% CI

### Correlation Analysis (Two Ranges)
- Pearson correlation coefficient
- Spearman rank correlation
- Covariance

### Calculation Modes
- Sample statistics (n-1 degrees of freedom)
- Population statistics (n degrees of freedom)

### Export Options
- Write statistics table to spreadsheet
- Copy to clipboard
- Export as formatted report

---

## UI Layout

```
┌─────────────────────────────────────┐
│ Statistical Analysis            [X] │
├─────────────────────────────────────┤
│ Data Selection:                     │
│  Range 1: [A1:A100] [Select]       │
│  Range 2: [B1:B100] [Select]       │
│           (optional for correlation)│
│                                     │
│ Mode: (•) Sample  ( ) Population   │
│                                     │
│ [Calculate] [Clear]                │
│                                     │
│ Results:                            │
│ ┌─────────────────────────────┐   │
│ │ Descriptive Statistics      │   │
│ │ ─────────────────────────── │   │
│ │ Count (n):   100            │   │
│ │ Mean:        52.34 ± 1.23   │   │
│ │ Median:      51.20          │   │
│ │ Mode:        50.00          │   │
│ │ Std Dev:     8.45           │   │
│ │ Variance:    71.40          │   │
│ │                             │   │
│ │ Range Statistics            │   │
│ │ ─────────────────────────── │   │
│ │ Min:         35.10          │   │
│ │ Max:         72.80          │   │
│ │ Range:       37.70          │   │
│ │ Q1:          46.25          │   │
│ │ Q3:          58.40          │   │
│ │ IQR:         12.15          │   │
│ │                             │   │
│ │ Distribution                │   │
│ │ ─────────────────────────── │   │
│ │ Skewness:    0.15           │   │
│ │ Kurtosis:    -0.42          │   │
│ │ Normality:   ✓ (p=0.23)     │   │
│ │   (Shapiro-Wilk test)       │   │
│ │                             │   │
│ │ Confidence Intervals        │   │
│ │ ─────────────────────────── │   │
│ │ 90% CI:  [51.2, 53.5]       │   │
│ │ 95% CI:  [50.3, 54.4]       │   │
│ │ 99% CI:  [49.1, 55.6]       │   │
│ │                             │   │
│ │ Correlation (Range 1 vs 2)  │   │
│ │ ─────────────────────────── │   │
│ │ Pearson r:   0.87 (p<0.001) │   │
│ │ Spearman ρ:  0.84 (p<0.001) │   │
│ │ Covariance:  45.23          │   │
│ └─────────────────────────────┘   │
│                                     │
│ [📋 Copy] [Write to Sheet]         │
│ Output Location: [E1] [Select]     │
│                                     │
│ [📊 Show Histogram]                │
└─────────────────────────────────────┘
```

---

## Data Flow Pattern

**Type**: Read → Analyze → Display (Pattern B)

1. User selects one or two data ranges
2. User chooses calculation mode (sample/population)
3. Sidebar reads values from Univer
4. **Send data to Rust backend for all calculations**
5. **Rust calculates all statistics using `statrs` library**
6. Display results in sidebar (TypeScript UI only)
7. Optional: User clicks "Write to Sheet" to export summary (calls Rust command)
8. Optional: Show histogram visualization (data from Rust)

---

## Technical Implementation

### TypeScript Interfaces

```typescript
interface StatisticalAnalysisSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef: UniverSpreadsheetRef;
  onSelectionChange: (cellRef: string) => void;
}

interface StatisticsInput {
  range1: string;
  range2?: string;
  mode: 'sample' | 'population';
}

interface StatisticsResult {
  // Descriptive
  count: number;
  mean: number;
  meanStdError: number; // Standard error of mean
  median: number;
  mode: number[];
  stdDev: number;
  variance: number;
  
  // Range
  min: number;
  max: number;
  range: number;
  q1: number;
  q2: number; // Same as median
  q3: number;
  iqr: number;
  
  // Distribution
  skewness: number;
  kurtosis: number;
  normalityTests: {
    shapiroWilk: {
      statistic: number;
      pValue: number;
      isNormal: boolean;
    };
    kolmogorovSmirnov?: {
      statistic: number;
      pValue: number;
      isNormal: boolean;
    };
  };
  
  // Confidence intervals for mean
  confidenceIntervals: {
    ci90: { lower: number; upper: number };
    ci95: { lower: number; upper: number };
    ci99: { lower: number; upper: number };
  };
  
  // Correlation (if two ranges provided)
  correlation?: {
    pearson: {
      coefficient: number;
      pValue: number;
      significant: boolean;
    };
    spearman: {
      coefficient: number;
      pValue: number;
      significant: boolean;
    };
    covariance: number;
  };
  
  // Histogram data
  histogram?: {
    bins: number[];
    counts: number[];
    binWidth: number;
  };
}

interface ExportOptions {
  includeHeaders: boolean;
  format: 'table' | 'list';
  significantDigits: number;
}
```

### Helper Functions (TypeScript - UI Only)

```typescript
// All calculations moved to Rust backend!
// TypeScript only handles UI and data passing

// Format statistics for display
function formatStatistic(value: number, decimals: number = 2): string {
  return value.toFixed(decimals);
}

// Calculate statistics by calling Rust backend
async function calculateStatistics(
  data1: number[],
  data2: number[] | undefined,
  mode: 'sample' | 'population'
): Promise<StatisticsResult> {
  try {
    // Call Rust backend - ALL calculations happen in Rust
    const result = await invoke<StatisticsResult>('calculate_statistics', {
      data1,
      data2,
      mode,
    });
    return result;
  } catch (error) {
    console.error('Statistics calculation failed:', error);
    throw error;
  }
}

// Write statistics to spreadsheet (call Rust to format)
async function writeStatisticsToSheet(
  univerRef: UniverSpreadsheetRef,
  stats: StatisticsResult,
  outputRange: string,
  options: ExportOptions
): Promise<void> {
  try {
    // Call Rust to format the statistics table
    const formattedTable = await invoke<string[][]>('format_statistics_table', {
      stats,
      options,
    });
    
    // Write to Univer
    await univerRef.current?.setRange(outputRange, formattedTable);
  } catch (error) {
    console.error('Failed to write statistics:', error);
    throw error;
  }
}
```

### Rust Backend Commands

```rust
// src-tauri/src/scientific/statistics.rs

use statrs::statistics::*;
use statrs::distribution::{ContinuousCDF, StudentsT};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StatisticsRequest {
    data: Vec<f64>,
    mode: String, // "sample" or "population"
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NormalityTest {
    statistic: f64,
    p_value: f64,
    is_normal: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StatisticsResponse {
    skewness: f64,
    kurtosis: f64,
    shapiro_wilk: NormalityTest,
}

#[tauri::command]
pub async fn calculate_advanced_statistics(
    request: StatisticsRequest,
) -> Result<StatisticsResponse, String> {
    let data = &request.data;
    
    // Calculate skewness
    let skewness = calculate_skewness(data);
    
    // Calculate kurtosis
    let kurtosis = calculate_kurtosis(data);
    
    // Shapiro-Wilk test
    let shapiro_wilk = shapiro_wilk_test(data)?;
    
    Ok(StatisticsResponse {
        skewness,
        kurtosis,
        shapiro_wilk,
    })
}

fn calculate_skewness(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let std_dev = (data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (n - 1.0))
        .sqrt();
    
    let skew = data.iter()
        .map(|x| ((x - mean) / std_dev).powi(3))
        .sum::<f64>() / n;
    
    skew * (n * (n - 1.0)).sqrt() / (n - 2.0)
}

fn calculate_kurtosis(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let std_dev = (data.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (n - 1.0))
        .sqrt();
    
    let kurt = data.iter()
        .map(|x| ((x - mean) / std_dev).powi(4))
        .sum::<f64>() / n;
    
    // Excess kurtosis (subtract 3 for normal distribution baseline)
    kurt - 3.0
}

fn shapiro_wilk_test(data: &[f64]) -> Result<NormalityTest, String> {
    // Implement Shapiro-Wilk test
    // This is a placeholder - use a proper statistical library
    
    if data.len() < 3 || data.len() > 5000 {
        return Err("Sample size must be between 3 and 5000".to_string());
    }
    
    // TODO: Implement actual Shapiro-Wilk algorithm
    // For now, return placeholder values
    
    Ok(NormalityTest {
        statistic: 0.98,
        p_value: 0.23,
        is_normal: true,
    })
}

#[tauri::command]
pub async fn calculate_correlation(
    data1: Vec<f64>,
    data2: Vec<f64>,
) -> Result<CorrelationResponse, String> {
    if data1.len() != data2.len() {
        return Err("Data ranges must have equal length".to_string());
    }
    
    let pearson = calculate_pearson(&data1, &data2);
    let spearman = calculate_spearman(&data1, &data2);
    
    Ok(CorrelationResponse {
        pearson,
        spearman,
    })
}

#[tauri::command]
pub async fn format_statistics_table(
    stats: StatisticsResult,
    options: ExportOptions,
) -> Result<Vec<Vec<String>>, String> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let decimals = options.significant_digits;
    
    if options.format == "table" {
        // Header
        if options.include_headers {
            rows.push(vec!["Statistic".to_string(), "Value".to_string()]);
        }
        
        // Descriptive statistics
        rows.push(vec!["Count".to_string(), stats.count.to_string()]);
        rows.push(vec!["Mean".to_string(), format!("{:.prec$}", stats.mean, prec = decimals)]);
        rows.push(vec!["Median".to_string(), format!("{:.prec$}", stats.median, prec = decimals)]);
        rows.push(vec!["Std Dev".to_string(), format!("{:.prec$}", stats.std_dev, prec = decimals)]);
        rows.push(vec!["Variance".to_string(), format!("{:.prec$}", stats.variance, prec = decimals)]);
        rows.push(vec!["Min".to_string(), format!("{:.prec$}", stats.min, prec = decimals)]);
        rows.push(vec!["Max".to_string(), format!("{:.prec$}", stats.max, prec = decimals)]);
        rows.push(vec!["Range".to_string(), format!("{:.prec$}", stats.range, prec = decimals)]);
        
        // Spacing
        rows.push(vec!["".to_string(), "".to_string()]);
        
        // Quartiles
        rows.push(vec!["Q1".to_string(), format!("{:.prec$}", stats.q1, prec = decimals)]);
        rows.push(vec!["Q3".to_string(), format!("{:.prec$}", stats.q3, prec = decimals)]);
        rows.push(vec!["IQR".to_string(), format!("{:.prec$}", stats.iqr, prec = decimals)]);
        
        // Spacing
        rows.push(vec!["".to_string(), "".to_string()]);
        
        // Confidence intervals
        rows.push(vec![
            "95% CI Lower".to_string(),
            format!("{:.prec$}", stats.confidence_intervals.ci95.lower, prec = decimals)
        ]);
        rows.push(vec![
            "95% CI Upper".to_string(),
            format!("{:.prec$}", stats.confidence_intervals.ci95.upper, prec = decimals)
        ]);
    }
    
    Ok(rows)
}
```

---

## Architecture Notes

**Rust-First Design**:
- ✅ **All calculations in Rust** using `statrs` library
- ✅ **All statistical algorithms in Rust** (mean, median, std dev, quartiles, etc.)
- ✅ **Normality tests in Rust** (Shapiro-Wilk, Kolmogorov-Smirnov)
- ✅ **Correlation calculations in Rust** (Pearson, Spearman)
- ✅ **Table formatting in Rust** (precise control over significant digits)
- ✅ **TypeScript only for UI** (display results, handle user input)

**Benefits**:
- More accurate calculations (proper statistical libraries)
- Better performance (Rust is faster than JavaScript)
- Type-safe statistical operations
- Consistent precision handling
- Easier to test and validate

---

## Dependencies

### Frontend (UI Only)
- No statistics libraries needed!
- Only UI libraries: `@mui/material`, `react`

### Backend (All Logic)
- **statrs** (Rust) - Statistical functions (mean, median, std dev, distributions)
- **nalgebra** (Rust) - Linear algebra for correlations

### Installation

```bash
# Frontend - No statistics libraries!
# Just install via npm install (already in package.json)
```

```toml
# Rust (Cargo.toml)
[dependencies]
statrs = "0.16"
nalgebra = "0.32"  # For correlation calculations
serde = { version = "1.0", features = ["derive"] }
```

---

## File Location

- **Component**: `AnaFis/src/components/spreadsheet/StatisticalAnalysisSidebar.tsx`
- **Backend**: `AnaFis/src-tauri/src/scientific/statistics.rs`
- **Types**: `AnaFis/src/types/statistics.ts`

---

## Success Criteria

- ✓ All calculations performed in Rust (no JavaScript math)
- ✓ Calculate all basic statistics correctly using `statrs`
- ✓ Support both sample and population modes
- ✓ Normality tests return accurate p-values
- ✓ Confidence intervals calculated with proper t-distribution
- ✓ Correlation for two ranges works (Pearson & Spearman)
- ✓ Export to spreadsheet formats correctly (Rust formats the table)
- ✓ Handle edge cases (empty data, single value, etc.)
- ✓ Performance: Handle up to 100,000 data points efficiently

---

## Testing Plan

### Unit Tests
- Statistical calculation accuracy
- Edge cases (empty, single value, identical values)
- Confidence interval calculations

### Integration Tests
- Read from Univer → Calculate → Display
- Write results back to spreadsheet
- Two-range correlation analysis

### E2E Tests
- Complete user workflow
- Export functionality
- Error handling

---

**Next Steps**: Implement after Quick Plot Sidebar is complete
