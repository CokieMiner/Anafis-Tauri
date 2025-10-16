# Monte Carlo Uncertainty Function 🎲

**Status**: Planned
**Priority**: High
**Complexity**: Medium-High
**Dependencies**: Enhanced Formula Engine, Async Result Injection

---

## Purpose

Implement a spreadsheet function `MONTECARLO()` that performs Monte Carlo uncertainty propagation for formulas that don't support analytical derivatives. The function will normalize results to 68% or 95% confidence intervals and handle asynchronous computation through result injection.

---

## Function Signature

```
MONTECARLO(formula, input_ranges, uncertainty_ranges, sample_size, confidence_level)
```

### Parameters
- **formula**: String - Formula to evaluate (e.g., `"A1*B1 + SIN(C1)"`)
- **input_ranges**: Array - Cell ranges with input values
- **uncertainty_ranges**: Array - Cell ranges with input uncertainties
- **sample_size**: Number - Monte Carlo samples (default: 10000)
- **confidence_level**: Number - Confidence level (0.68 or 0.95, default: 0.95)

### Returns
- **Number**: Uncertainty value normalized to specified confidence interval

---

## Implementation Strategy

### Async Function with Result Injection

Since Univer functions must be synchronous, implement a two-phase approach:

1. **Function Call**: `MONTECARLO()` initiates async computation and returns placeholder
2. **Background Processing**: Sidebar runs Monte Carlo simulation
3. **Result Injection**: Completed result is written back to calling cell

---

## Usage Examples

### Basic Usage
```
= MONTECARLO("A1*B1 + SIN(C1)", {"A1:A10", "B1:B10", "C1:C10"}, {"D1:D10", "E1:E10", "F1:F10"})
```
Returns 95% confidence interval uncertainty using 10,000 samples.

### Custom Parameters
```
= MONTECARLO("A1*B1", {"A1:A10", "B1:B10"}, {"D1:D10", "E1:E10"}, 50000, 0.68)
```
Returns 68% confidence interval using 50,000 samples.

### Complex Formulas
```
= MONTECARLO("GAMMA(A1) * BETA(B1, C1) + ZETA(D1)", ranges..., uncertainties..., 25000, 0.95)
```
Works with functions that lack analytical derivatives.

---

## Implementation Architecture

### Backend (Rust/Tauri)

#### Monte Carlo Engine

```rust
pub struct MonteCarloFunction {
    pub formula: String,
    pub input_ranges: Vec<String>,
    pub uncertainty_ranges: Vec<String>,
    pub sample_size: usize,
    pub confidence_level: f64,
}

impl MonteCarloFunction {
    pub fn evaluate(&self, spreadsheet_data: &HashMap<String, f64>) -> Result<f64, String> {
        // Run Monte Carlo simulation
        // Return uncertainty normalized to confidence interval
    }
}
```

#### Tauri Commands

```rust
#[tauri::command]
pub async fn compute_monte_carlo_uncertainty(
    request: MonteCarloRequest
) -> Result<f64, String>;

#[tauri::command]
pub async fn start_monte_carlo_function(
    request: MonteCarloRequest,
    cell_ref: String
) -> Result<(), String>;
```

### Frontend (React/TypeScript)

#### Custom Formula Registration

Add to `customFormulas.ts`:

```typescript
// Register MONTECARLO function
formulaEngine.registerFunction(
    'MONTECARLO',
    (...args: FormulaParam[]) => {
        const formula = args[0] as string;
        const inputRanges = args[1] as string[];
        const uncertaintyRanges = args[2] as string[];
        const sampleSize = (args[3] as number) || 10000;
        const confidenceLevel = (args[4] as number) || 0.95;

        // Get current cell reference
        const cellRef = getCurrentCellReference();

        // Start async computation
        startMonteCarloComputation({
            formula,
            inputRanges,
            uncertaintyRanges,
            sampleSize,
            confidenceLevel,
            cellRef
        });

        // Return placeholder
        return "COMPUTING...";
    },
    'Monte Carlo uncertainty: MONTECARLO(formula, inputs, uncertainties, samples, confidence)'
);
```

#### Result Injection System

```typescript
const MonteCarloManager = {
    activeComputations: new Map(),

    startComputation(request: MonteCarloRequest) {
        const id = generateId();
        this.activeComputations.set(id, request);

        // Start async computation
        invoke('start_monte_carlo_function', { request, cellRef: request.cellRef });

        // Set up result listener
        this.listenForResult(id, request.cellRef);
    },

    listenForResult(computationId: string, cellRef: string) {
        // Listen for computation complete event
        // When received, update cell with result
        univerRef.current?.setCellValue(cellRef, result);
    }
};
```

---

## UI Integration

### Cell Feedback

When `MONTECARLO()` is called:
1. **"COMPUTING..."** - Initial placeholder
2. **"45% complete"** - Progress updates
3. **"2.34"** - Final uncertainty value

### Sidebar Monitoring

Uncertainty Sidebar shows active computations:

```
Active Monte Carlo Functions:
├── A1: MONTECARLO(...) - 67% complete
├── B5: MONTECARLO(...) - 23% complete
└── C10: MONTECARLO(...) - Computing...
```

---

## Technical Implementation

### Phase 1: Core Function (Week 1)
- [ ] Add MONTECARLO function to customFormulas.ts
- [ ] Basic Monte Carlo engine in Rust
- [ ] Synchronous evaluation for small samples
- [ ] Cell result injection mechanism

### Phase 2: Async Processing (Week 2)
- [ ] Async computation queue
- [ ] Progress tracking in cells
- [ ] Error handling and validation
- [ ] Result caching

### Phase 3: Advanced Features (Week 3)
- [ ] Confidence interval normalization
- [ ] Multiple distribution types
- [ ] Performance optimizations
- [ ] Comprehensive testing

---

## Performance Considerations

### Sample Size Limits
- **Synchronous**: Up to 10K samples (fast, blocking)
- **Async**: 10K-100K samples (background processing)
- **Hybrid**: Auto-select based on sample size

### Memory Management
- Stream processing for large datasets
- Result caching to avoid recomputation
- Cleanup of completed computations

---

## Testing Strategy

### Function Tests
- Basic MONTECARLO calls with simple formulas
- Complex formulas with custom functions
- Error conditions and invalid inputs
- Performance benchmarks

### Integration Tests
- Cell updates when computation completes
- Progress display accuracy
- Multiple simultaneous functions
- Memory usage and cleanup

---

## Success Criteria

- [ ] `MONTECARLO()` function works in spreadsheet cells
- [ ] Supports formulas with GAMMA, BETA, ZETA functions
- [ ] Returns properly normalized confidence intervals
- [ ] Handles 10K-50K samples without blocking UI
- [ ] Clear progress feedback in cells
- [ ] Automatic result injection when complete
- [ ] Robust error handling for invalid inputs</content>
<parameter name="filePath">/home/pedrom/Documentos/Anafis-Tauri/Plans/monte_carlo_uncertainty_function.md