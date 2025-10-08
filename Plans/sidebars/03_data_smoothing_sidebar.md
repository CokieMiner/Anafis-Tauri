# Data Smoothing/Filtering Sidebar 🌊

**Status**: Planned  
**Priority**: Medium  
**Complexity**: High  
**Dependencies**: DSP libraries (Rust), scipy algorithms

---

## Purpose

Apply smoothing and filtering algorithms to noisy experimental data to improve signal quality and reduce noise.

---

## Features

### Smoothing Methods
- **Mo}
```

---

## Rust Command Implementation (Continued)

```rust
// src-tauri/src/scientific/smoothing.rs

use ndarray::{Array1, ArrayView1};

#[derive(serde::Deserialize)]
pub struct SmoothingRequest {
    data: Vec<f64>,
    method: String,
    window_size: Option<usize>,
    polynomial_order: Option<usize>,
    sigma: Option<f64>,
    // ... other parameters
}

#[derive(serde::Serialize)]
pub struct SmoothingResponse {
    smoothed_data: Vec<f64>,
    original_rms: f64,
    smoothed_rms: f64,
    noise_reduction: f64,
}

#[tauri::command]
pub async fn apply_smoothing(
    request: SmoothingRequest,
) -> Result<SmoothingResponse, String> {Average**:
  - Simple moving average
  - Weighted moving average
  - Exponential moving average
- **Savitzky-Golay Filter**: Polynomial smoothing that preserves peaks
- **Gaussian Filter**: Gaussian kernel smoothing
- **LOWESS/LOESS**: Locally weighted scatterplot smoothing

### Filtering Methods
- **Low-pass Filter**: Remove high-frequency noise
- **High-pass Filter**: Remove low-frequency trends
- **Band-pass Filter**: Keep only specific frequency range
- **Notch Filter**: Remove specific frequency (e.g., 60 Hz power line noise)

### Parameters
- **Window Size**: Adjustable smoothing window
- **Polynomial Order**: For Savitzky-Golay (typically 2-4)
- **Sigma**: For Gaussian filter
- **Cutoff Frequency**: For frequency-domain filters
- **Centered**: Use centered or causal window

### Preview & Analysis
- Side-by-side before/after comparison plot
- RMS (Root Mean Square) comparison
- Noise reduction percentage
- Frequency spectrum comparison (FFT)

### Actions
- Preview smoothing before applying
- Write smoothed data to new column
- Overwrite original data (with warning)
- Undo/revert capability

---

## UI Layout

```
┌─────────────────────────────────────┐
│ Data Smoothing/Filtering        [X] │
├─────────────────────────────────────┤
│ Data Selection:                     │
│  Input Range:  [A1:A100] [Select]  │
│  Output Range: [B1:B100] [Select]  │
│                                     │
│ Method: [Moving Average ▼]         │
│                                     │
│ ┌───────────────────────────────┐ │
│ │ Method-Specific Parameters    │ │
│ │                               │ │
│ │ Window Size:                  │ │
│ │  [5──────────────────] 5      │ │
│ │                               │ │
│ │ Window Type:                  │ │
│ │  ( ) Simple                   │ │
│ │  (•) Weighted                 │ │
│ │  ( ) Exponential              │ │
│ │                               │ │
│ │ [✓] Centered window           │ │
│ │ [ ] Edge padding              │ │
│ └───────────────────────────────┘ │
│                                     │
│ Preview:                            │
│ ┌─────────────────────────────┐   │
│ │  Original (blue)            │   │
│ │  Smoothed (red)             │   │
│ │     ╱╲    ╱──╲              │   │
│ │ ╱╲ ╱  ╲  ╱    ╲   ╱╲        │   │
│ │╯  ╲╱───╲╱──────╲─╱  ╲───    │   │
│ │                             │   │
│ └─────────────────────────────┘   │
│                                     │
│ [Update Preview]                   │
│                                     │
│ Analysis:                           │
│ ┌─────────────────────────────┐   │
│ │ Original RMS:    2.45       │   │
│ │ Smoothed RMS:    0.82       │   │
│ │ Noise Reduction: 66.5%      │   │
│ │ Data Points:     100        │   │
│ │ Effective BW:    0.2 Hz     │   │
│ └─────────────────────────────┘   │
│                                     │
│ [Apply] [Reset] [Show FFT]         │
└─────────────────────────────────────┘
```

### Advanced Parameters Panel (for Savitzky-Golay)
```
┌───────────────────────────────┐
│ Savitzky-Golay Parameters     │
│                               │
│ Window Size:                  │
│  [11──────────────────] 11    │
│  (must be odd)                │
│                               │
│ Polynomial Order:             │
│  [3───────────────────] 3     │
│  (< window size)              │
│                               │
│ Derivative: [0 ▼]             │
│  (0=smoothing, 1=1st deriv)   │
│                               │
│ [✓] Preserve peak shapes      │
└───────────────────────────────┘
```

---

## Data Flow Pattern

**Type**: Read → Process → Write (Pattern A)

1. User selects input data range
2. Sidebar reads values from Univer
3. User selects smoothing/filtering method
4. User adjusts parameters
5. Preview shows before/after comparison and statistics
6. User iterates on parameters until satisfied
7. User clicks "Apply"
8. Smoothed data written to output range in Univer
9. Option to write processing metadata

---

## Technical Implementation

### TypeScript Interfaces

```typescript
interface DataSmoothingSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef: UniverSpreadsheetRef;
  onSelectionChange: (cellRef: string) => void;
}

type SmoothingMethod = 
  | 'moving_average_simple'
  | 'moving_average_weighted'
  | 'moving_average_exponential'
  | 'savitzky_golay'
  | 'gaussian'
  | 'lowess'
  | 'lowpass_filter'
  | 'highpass_filter'
  | 'bandpass_filter'
  | 'notch_filter';

interface SmoothingConfig {
  method: SmoothingMethod;
  
  // Common parameters
  windowSize?: number;
  centered?: boolean;
  edgePadding?: 'zero' | 'reflect' | 'wrap' | 'extrapolate';
  
  // Moving average specific
  weights?: number[]; // For weighted moving average
  alpha?: number; // For exponential moving average (0-1)
  
  // Savitzky-Golay specific
  polynomialOrder?: number;
  derivative?: number; // 0=smoothing, 1=first derivative, 2=second derivative
  
  // Gaussian specific
  sigma?: number;
  truncate?: number; // Standard deviations to truncate at
  
  // LOWESS specific
  frac?: number; // Fraction of data for local regression (0-1)
  iterations?: number; // Number of robustifying iterations
  
  // Frequency filter specific
  cutoffFrequency?: number; // Hz
  filterOrder?: number;
  samplingRate?: number; // Hz
  lowCutoff?: number; // For band-pass
  highCutoff?: number; // For band-pass
  notchFrequency?: number; // For notch filter
  qualityFactor?: number; // For notch filter
}

interface SmoothingResult {
  smoothedData: number[];
  originalRMS: number;
  smoothedRMS: number;
  noiseReduction: number; // Percentage
  effectiveBandwidth?: number; // Hz
  processingTime: number; // ms
}

interface SmoothingState {
  inputRange: string;
  outputRange: string;
  config: SmoothingConfig;
  originalData: number[] | null;
  previewData: number[] | null;
  result: SmoothingResult | null;
  isProcessing: boolean;
  error: string | null;
}
```

---

## Rust Backend Implementation

**IMPORTANT**: ALL smoothing/filtering algorithms are implemented in Rust. TypeScript ONLY calls Rust commands via invoke().

### Rust Smoothing Module

```rust
// AnaFis/src-tauri/src/smoothing/mod.rs

use nalgebra::DVector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SmoothingRequest {
    data: Vec<f64>,
    method: String,
    window_size: Option<usize>,
    polynomial_order: Option<usize>,
    sigma: Option<f64>,
    cutoff_freq: Option<f64>,
    sample_rate: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct SmoothingResponse {
    smoothed_data: Vec<f64>,
    original_rms: f64,
    smoothed_rms: f64,
    noise_reduction: f64,
}
```

### TypeScript Wrapper (UI Only)

```typescript
// REMOVED: All calculation functions
// TypeScript ONLY calls Rust backend:

import { invoke } from '@tauri-apps/api/tauri';

async function applySmoothing(
  data: number[],
  config: SmoothingConfig
): Promise<SmoothingResult> {
  return await invoke('apply_smoothing', {
    request: {
      data,
      method: config.method,
      windowSize: config.windowSize,
      polynomialOrder: config.polynomialOrder,
      sigma: config.sigma,
      cutoffFreq: config.cutoffFrequency,
      sampleRate: config.sampleRate
    }
  });
}
```

---

## Rust Smoothing Algorithms

```rust
// AnaFis/src-tauri/src/smoothing/algorithms.rs

use ndarray::Array1;

// Savitzky-Golay filter implementation
pub fn savitzky_golay_filter(
    data: &Array1<f64>,
    window_size: usize,
    polynomial_order: usize,
) -> Result<Array1<f64>, String> {
    if window_size % 2 == 0 {
        return Err("Window size must be odd".to_string());
    }
    if polynomial_order >= window_size {
        return Err("Polynomial order must be less than window size".to_string());
    }
    
    // Implementation using least squares polynomial fitting
    // ... (full implementation in Rust)
    
    Ok(smoothed)
}

// Gaussian filter implementation
pub fn gaussian_filter(
    data: &Array1<f64>,
    sigma: f64,
) -> Result<Array1<f64>, String> {
    // Implementation using Gaussian kernel
    // ... (full implementation in Rust)
    
    Ok(smoothed)
}

// Low-pass filter using FFT
pub fn lowpass_filter(
    data: &Array1<f64>,
    cutoff_freq: f64,
    sample_rate: f64,
) -> Result<Array1<f64>, String> {
    // Implementation using FFT
    // ... (full implementation in Rust)
    
    Ok(filtered)
}
```

### Rust Command Implementation

```rust
// AnaFis/src-tauri/src/smoothing/mod.rs

pub struct SmoothingResponse {
    
    result.push(weightSum > 0 ? sum / weightSum : data[i]);
  }
  
  return result;
}

---

## Architecture Notes

**Rust-First Design**:
- ALL smoothing algorithms implemented in Rust (DSP processing)
- TypeScript ONLY calls `invoke('apply_smoothing')` with parameters
- No calculation or filtering logic in TypeScript
- UI handles only: parameter input, visualization, cell selection

**Benefits**:
- 10-100x faster smoothing on large datasets
- Access to advanced DSP libraries
- Consistent results across platforms
- Memory-efficient processing

---

## Rust Backend Implementation

```rust
// src-tauri/src/scientific/smoothing.rs

use ndarray::{Array1, ArrayView1};

#[derive(serde::Deserialize)]
pub struct SmoothingRequest {
    data: Vec<f64>,
    method: String,
    window_size: Option<usize>,
    polynomial_order: Option<usize>,
    sigma: Option<f64>,
    // ... other parameters
}

#[derive(serde::Serialize)]
pub struct SmoothingResponse {
    smoothed_data: Vec<f64>,
    original_rms: f64,
    smoothed_rms: f64,
    noise_reduction: f64,
}

#[tauri::command]
pub async fn apply_smoothing(
    request: SmoothingRequest,
) -> Result<SmoothingResponse, String> {
    let data = Array1::from(request.data.clone());
    
    let smoothed = match request.method.as_str() {
        "savitzky_golay" => {
            savitzky_golay_filter(
                &data,
                request.window_size.unwrap_or(5),
                request.polynomial_order.unwrap_or(2),
            )?
        },
        "gaussian" => {
            gaussian_filter(&data, request.sigma.unwrap_or(1.0))?
        },
        "lowpass_filter" => {
            // Implement FFT-based low-pass filter
            lowpass_filter(&data, request)?
        },
        _ => return Err(format!("Unknown method: {}", request.method)),
    };
    
    let original_rms = calculate_rms(&data);
    let smoothed_rms = calculate_rms(&smoothed);
    let noise_reduction = ((original_rms - smoothed_rms) / original_rms) * 100.0;
    
    Ok(SmoothingResponse {
        smoothed_data: smoothed.to_vec(),
        original_rms,
        smoothed_rms,
        noise_reduction,
    })
}

// Savitzky-Golay filter implementation
fn savitzky_golay_filter(
    data: &Array1<f64>,
    window_size: usize,
    poly_order: usize,
) -> Result<Array1<f64>, String> {
    if window_size % 2 == 0 {
        return Err("Window size must be odd".to_string());
    }
    if poly_order >= window_size {
        return Err("Polynomial order must be less than window size".to_string());
    }
    
    // Compute Savitzky-Golay coefficients
    let coeffs = compute_sg_coefficients(window_size, poly_order)?;
    
    // Apply filter
    let half_window = window_size / 2;
    let mut result = Array1::zeros(data.len());
    
    for i in 0..data.len() {
        let mut sum = 0.0;
        for j in 0..window_size {
            let idx = (i as isize + j as isize - half_window as isize)
                .max(0)
                .min(data.len() as isize - 1) as usize;
            sum += data[idx] * coeffs[j];
        }
        result[i] = sum;
    }
    
    Ok(result)
}

// Gaussian filter
fn gaussian_filter(data: &Array1<f64>, sigma: f64) -> Result<Array1<f64>, String> {
    let window_size = (6.0 * sigma).ceil() as usize;
    let half = window_size / 2;
    
    // Generate Gaussian kernel
    let mut kernel = Vec::with_capacity(window_size);
    let mut sum = 0.0;
    
    for i in 0..window_size {
        let x = (i as f64 - half as f64) / sigma;
        let value = (-0.5 * x * x).exp();
        kernel.push(value);
        sum += value;
    }
    
    // Normalize kernel
    for k in &mut kernel {
        *k /= sum;
    }
    
    // Convolve
    let mut result = Array1::zeros(data.len());
    for i in 0..data.len() {
        let mut convolved = 0.0;
        for j in 0..window_size {
            let idx = (i as isize + j as isize - half as isize)
                .max(0)
                .min(data.len() as isize - 1) as usize;
            convolved += data[idx] * kernel[j];
        }
        result[i] = convolved;
    }
    
    Ok(result)
}

fn calculate_rms(data: &Array1<f64>) -> f64 {
    let mean = data.mean().unwrap_or(0.0);
    let squared_diffs: f64 = data.iter()
        .map(|&x| (x - mean).powi(2))
        .sum();
    (squared_diffs / data.len() as f64).sqrt()
}
```

---

## Dependencies

### Frontend
- Basic moving average implementations
- Plotting for preview (reuse from Quick Plot)

### Backend
- **ndarray** - Array operations
- **rustfft** - FFT for frequency-domain filters
- **nalgebra** - Linear algebra for Savitzky-Golay
- **statrs** - Statistical functions

### Installation

```bash
# Frontend
npm install @types/ndarray -D
```

```toml
# Rust (Cargo.toml)
[dependencies]
ndarray = "0.15"
rustfft = "6.1"
nalgebra = "0.32"
statrs = "0.16"
```

---

## File Location

- **Component**: `AnaFis/src/components/spreadsheet/DataSmoothingSidebar.tsx`
- **Backend**: `AnaFis/src-tauri/src/scientific/smoothing.rs`
- **Types**: `AnaFis/src/types/smoothing.ts`

---

## Success Criteria

- ✓ All smoothing methods work correctly
- ✓ Preview updates in real-time
- ✓ Parameters are validated
- ✓ Edge cases handled (padding, boundaries)
- ✓ Performance: Process 10,000 points in < 100ms
- ✓ Savitzky-Golay preserves peak shapes
- ✓ FFT filters work correctly with proper frequency scaling

---

## Testing Plan

### Unit Tests
- Each smoothing algorithm accuracy
- Edge padding methods
- Parameter validation

### Integration Tests
- Read from Univer → Smooth → Write back
- Preview generation
- Multiple method comparisons

### E2E Tests
- Complete user workflow
- Parameter adjustment and preview updates
- Error handling

---

**Next Steps**: Implement after Statistical Analysis Sidebar
