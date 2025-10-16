# Monte Carlo Uncertainty Propagation Implementation Plan 🎲

**Status**: Planned
**Priority**: High
**Complexity**: High
**Dependencies**: rand (Rust), ndarray (Rust), Web Workers (Frontend)

---

## Purpose

Implement Monte Carlo uncertainty propagation as an alternative to analytical uncertainty propagation. Monte Carlo methods provide more accurate results for complex functions with non-linear dependencies and can handle arbitrary probability distributions.

---

## Overview

Monte Carlo uncertainty propagation works by:
1. Sampling random values from input uncertainty distributions
2. Evaluating the function for each sample
3. Analyzing the resulting output distribution

This approach is more computationally intensive but provides better accuracy for complex scenarios.

---

## Features

### Core Functionality
- **Sampling Methods**: Support for normal, uniform, triangular, and custom distributions
- **Sample Size Control**: Configurable number of Monte Carlo iterations (1K to 1M)
- **Convergence Monitoring**: Real-time convergence statistics and confidence intervals
- **Output Statistics**: Mean, standard deviation, confidence intervals, percentiles

### Distribution Types
- **Normal Distribution**: μ ± σ
- **Uniform Distribution**: [min, max]
- **Triangular Distribution**: min, mode, max
- **Custom Distribution**: From data arrays or empirical distributions

### Advanced Features
- **Correlation Support**: Handle correlated input variables
- **Multivariate Outputs**: Propagate uncertainty through multi-output functions
- **Sensitivity Analysis**: Identify which inputs contribute most to output uncertainty
- **Convergence Diagnostics**: Statistical tests for Monte Carlo convergence

### Performance Optimizations
- **Web Workers**: Run Monte Carlo simulations in background threads
- **Streaming Results**: Show partial results as simulation progresses
- **Adaptive Sampling**: Automatically adjust sample size based on convergence
- **GPU Acceleration**: Use WebGL for large-scale simulations (future)

---

## Implementation Architecture

### Backend (Rust/Tauri)

#### New Modules
```
src-tauri/src/scientific/
├── monte_carlo.rs              # Core Monte Carlo engine
├── distributions.rs            # Probability distribution implementations
└── sampling.rs                 # Random sampling algorithms
```

#### Key Components

**MonteCarloEngine**
```rust
pub struct MonteCarloEngine {
    rng: ThreadRng,
    distributions: HashMap<String, Box<dyn Distribution<f64>>>,
}

impl MonteCarloEngine {
    pub fn run_simulation(&self, config: MonteCarloConfig) -> MonteCarloResult;
}
```

**Distribution Types**
```rust
pub enum DistributionType {
    Normal { mean: f64, std_dev: f64 },
    Uniform { min: f64, max: f64 },
    Triangular { min: f64, mode: f64, max: f64 },
    Custom { samples: Vec<f64> },
}
```

**Configuration**
```rust
pub struct MonteCarloConfig {
    pub formula: String,
    pub variables: Vec<Variable>,
    pub sample_size: usize,
    pub seed: Option<u64>,
    pub convergence_threshold: f64,
}
```

#### Tauri Commands
```rust
#[tauri::command]
pub async fn run_monte_carlo_simulation(
    config: MonteCarloConfig
) -> Result<MonteCarloResult, String>;

#[tauri::command]
pub async fn get_distribution_info(
    dist_type: DistributionType
) -> Result<DistributionInfo, String>;
```

### Frontend (React/TypeScript)

#### Enhanced UncertaintySidebar

Add Monte Carlo tab to existing uncertainty sidebar:

```
Uncertainty Propagation
├── Analytical Method (current)
│   ├── Variables setup
│   ├── Formula input
│   └── Results display
└── Monte Carlo Method (new)
    ├── Distribution setup
    ├── Simulation parameters
    ├── Progress monitoring
    └── Statistical results
```

#### New Components

**DistributionSelector**
```tsx
interface DistributionSelectorProps {
  variable: Variable;
  onDistributionChange: (distribution: DistributionConfig) => void;
}

const DistributionSelector: React.FC<DistributionSelectorProps> = ({
  variable,
  onDistributionChange
}) => {
  // UI for selecting and configuring probability distributions
};
```

**MonteCarloProgress**
```tsx
interface MonteCarloProgressProps {
  progress: number;
  currentSample: number;
  totalSamples: number;
  convergence: ConvergenceStats;
}

const MonteCarloProgress: React.FC<MonteCarloProgressProps> = ({
  progress,
  currentSample,
  totalSamples,
  convergence
}) => {
  // Real-time progress display with convergence monitoring
};
```

**StatisticalResults**
```tsx
interface StatisticalResultsProps {
  results: MonteCarloResult;
  confidenceLevel: number;
}

const StatisticalResults: React.FC<StatisticalResultsProps> = ({
  results,
  confidenceLevel
}) => {
  // Display statistical analysis of Monte Carlo results
};
```

#### State Management

Add Monte Carlo state to uncertainty propagation:

```tsx
interface MonteCarloState {
  method: 'analytical' | 'monte_carlo';
  sampleSize: number;
  distributions: Record<string, DistributionConfig>;
  simulationRunning: boolean;
  progress: number;
  results: MonteCarloResult | null;
  convergence: ConvergenceStats;
}
```

---

## UI Layout

### Distribution Setup Tab

```
┌─────────────────────────────────────┐
│ Monte Carlo Uncertainty         [X] │
├─────────────────────────────────────┤
│ Method: (•) Analytical  ( ) Monte Carlo │
│                                     │
│ Monte Carlo Settings:              │
│ Sample Size: [10000] [1K] [10K] [100K] │
│ Random Seed: [auto] [12345]       │
│                                     │
│ Variables:                         │
│ ┌─────────────────────────────────┐ │
│ │ x: Normal(10, 0.5) [Edit]      │ │
│ │ y: Uniform(5, 15)  [Edit]      │ │
│ │ z: Triangular(0, 5, 10) [Edit] │ │
│ └─────────────────────────────────┘ │
│ [Add Variable]                     │
│                                     │
│ [Run Simulation] [Stop]            │
│                                     │
│ Progress: ████████░░░░ 75%         │
│ Samples: 7,500 / 10,000            │
│ Convergence: ✓ (p=0.95)            │
│                                     │
│ Results:                            │
│ ┌─────────────────────────────────┐ │
│ │ Output Statistics               │ │
│ │ ─────────────────────────────── │ │
│ │ Mean:        42.35 ± 2.15       │ │
│ │ Std Dev:     8.42               │ │
│ │ 95% CI:      [38.12, 46.58]     │ │
│ │ Min/Max:     25.3 / 58.9        │ │
│ │                                 │ │
│ │ Distribution Fit                │ │
│ │ ─────────────────────────────── │ │
│ │ Skewness:    -0.12              │ │
│ │ Kurtosis:     0.08              │ │
│ │ Normality:    ✓ (p=0.87)        │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

### Distribution Editor Dialog

```
┌─────────────────────────────────────┐
│ Edit Distribution: x             [X] │
├─────────────────────────────────────┤
│ Distribution Type:                 │
│ (•) Normal     ( ) Uniform         │
│ ( ) Triangular ( ) Custom          │
│                                     │
│ Parameters:                        │
│ Mean: [10.0]                       │
│ Std Dev: [0.5]                     │
│                                     │
│ Preview:                           │
│ ┌─────────────────────────────┐   │
│ │          ▄▄▄                │   │
│ │        ▄█████▄              │   │
│ │      ▄█████████▄            │   │
│ │    ▄█████████████▄          │   │
│ │  ▄█████████████████▄        │   │
│ │ ▄████████████████████▄      │   │
│ └─────────────────────────────┘   │
│                                     │
│ [OK] [Cancel]                      │
└─────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Implement basic Monte Carlo engine in Rust
- [ ] Add support for normal and uniform distributions
- [ ] Create basic Tauri commands for simulation
- [ ] Add Monte Carlo tab to UncertaintySidebar UI

### Phase 2: Distribution Support (Week 3)
- [ ] Implement triangular and custom distributions
- [ ] Add distribution parameter validation
- [ ] Create distribution preview/visualization
- [ ] Add convergence monitoring

### Phase 3: Advanced Features (Week 4)
- [ ] Implement correlation support
- [ ] Add sensitivity analysis
- [ ] Create statistical result analysis
- [ ] Add export capabilities

### Phase 4: Performance & Polish (Week 5)
- [ ] Implement Web Workers for background processing
- [ ] Add adaptive sampling
- [ ] Performance optimizations
- [ ] Comprehensive testing and documentation

---

## Technical Considerations

### Performance Requirements
- **Sample Sizes**: Support 10K to 1M samples
- **Real-time Updates**: Progress updates every 100-1000 samples
- **Memory Usage**: Efficient storage of large sample arrays
- **Threading**: Background execution without blocking UI

### Accuracy Requirements
- **Convergence**: Statistical tests for Monte Carlo convergence
- **Precision**: High-precision floating point calculations
- **Randomness**: High-quality random number generation

### Integration Points
- **Spreadsheet**: Read input ranges and write results
- **Formula Engine**: Evaluate formulas with sampled values
- **Data Library**: Save/load Monte Carlo configurations

---

## Testing Strategy

### Unit Tests
- Distribution sampling accuracy
- Formula evaluation correctness
- Statistical calculation validation

### Integration Tests
- End-to-end Monte Carlo simulation
- Spreadsheet integration
- Large dataset performance

### Validation Tests
- Compare with analytical results for simple cases
- Verify convergence behavior
- Test edge cases and error conditions

---

## Dependencies

### Rust Crates
```toml
[dependencies]
rand = "0.8"
rand_distr = "0.4"
statrs = "0.16"
ndarray = "0.15"
```

### Frontend Libraries
```json
{
  "d3": "^7.8.0",
  "react-plotly.js": "^2.6.0"
}
```

---

## Risk Assessment

### High Risk
- **Performance**: Monte Carlo simulations can be computationally intensive
- **Accuracy**: Ensuring statistical validity of results
- **Memory Usage**: Large sample arrays may cause memory issues

### Mitigation Strategies
- **Web Workers**: Run simulations in background threads
- **Streaming**: Process results in batches
- **Validation**: Comprehensive statistical testing
- **Optimization**: Efficient algorithms and data structures

---

## Success Criteria

- [ ] Monte Carlo propagation produces results within 1% of analytical methods for linear functions
- [ ] Support for 10K-100K samples with real-time progress updates
- [ ] All major probability distributions supported
- [ ] Convergence monitoring and automatic stopping
- [ ] Integration with existing uncertainty propagation workflow
- [ ] Performance suitable for interactive use (< 30 seconds for 10K samples)

---

## Future Enhancements

- **GPU Acceleration**: WebGL-based Monte Carlo for massive simulations
- **Distributed Computing**: Multi-machine Monte Carlo clusters
- **Advanced Distributions**: Beta, gamma, log-normal, etc.
- **Time-dependent Uncertainty**: Monte Carlo for dynamic systems
- **Bayesian Integration**: Combine with Bayesian uncertainty quantification</content>
<parameter name="filePath">/home/pedrom/Documentos/Anafis-Tauri/Plans/monte_carlo_uncertainty_propagation.md