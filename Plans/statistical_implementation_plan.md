# Statistical Features Implementation Plan

## Overview

This plan outlines the implementation of missing hypothesis testing and power analysis features in the AnaFis statistical analysis system, building on the existing 4-layer architecture (command → coordination → algorithms → primitives).

## Missing Features

1. **Hypothesis Testing Suite**:
   - t-tests: one-sample, two-sample (paired/unpaired), Welch's t-test
   - ANOVA: one-way, two-way, robust versions with post-hoc tests
   - Chi-square tests: goodness of fit, test of independence

2. **Power Analysis**:
   - Power calculations for t-tests, ANOVA, and chi-square tests
   - Sample size determination
   - Power curves over sample size ranges
   - Post-hoc power analysis

## Implementation Strategy

- **Backend-first approach**: Implement statistical algorithms in Rust first, then integrate with frontend
- **Layered architecture compliance**: Extend existing layers without breaking current structure
- **State-of-the-art methods**: Use Welch's t-test for unequal variances, robust ANOVA, FDR corrections
- **Performance optimization**: Leverage rayon for parallel computations where applicable

## File Structure and Changes

### types.rs Extensions
Add new result types:
- `TTestResult` for t-test outputs (t-statistic, p-value, confidence intervals, effect size)
- `AnovaResult` for ANOVA outputs (F-statistic, p-value, degrees of freedom, post-hoc results)
- `TwoWayAnovaResult` for two-way ANOVA
- `ChiSquareResult` for chi-square test outputs
- `PowerAnalysisResult` for power calculation outputs
- `PowerCurveResult` for power curve data points

### hypothesis_testing.rs Extensions
Add new test implementations in `layer3_algorithms/hypothesis_testing.rs`:
- `one_sample_t_test(data: &[f64], mu: f64) -> Result<TTestResult, Error>`
- `paired_t_test(data1: &[f64], data2: &[f64]) -> Result<TTestResult, Error>`
- `two_sample_t_test(data1: &[f64], data2: &[f64], equal_var: bool) -> Result<TTestResult, Error>`
- `welch_t_test(data1: &[f64], data2: &[f64]) -> Result<TTestResult, Error>`
- `one_way_anova(groups: &[&[f64]]) -> Result<AnovaResult, Error>`
- `two_way_anova(data: &[f64], factor1: &[usize], factor2: &[usize]) -> Result<TwoWayAnovaResult, Error>`
- `chi_square_goodness_of_fit(observed: &[f64], expected: &[f64]) -> Result<ChiSquareResult, Error>`
- `chi_square_independence(table: &[&[f64]]) -> Result<ChiSquareResult, Error>`

### statistical_power.rs Extensions
Extend `layer4_primitives/statistical_power.rs` with full power analysis:
- `power_t_test(delta: f64, sigma: f64, n: usize, alpha: f64, alternative: Alternative) -> f64`
- `sample_size_t_test(delta: f64, sigma: f64, power: f64, alpha: f64, alternative: Alternative) -> usize`
- `power_anova(k: usize, n: usize, effect_size: f64, alpha: f64) -> f64`
- `power_chi_square(w: f64, df: usize, n: usize, alpha: f64) -> f64`
- `post_hoc_power(test_stat: f64, df: usize, alpha: f64, alternative: Alternative) -> f64`
- `power_curve_t_test(delta: f64, sigma: f64, alpha: f64, alternative: Alternative, n_range: Range<usize>) -> Vec<(usize, f64)>`
- `power_curve_anova(k: usize, effect_size: f64, alpha: f64, n_range: Range<usize>) -> Vec<(usize, f64)>`

### Layer2 Coordination Extensions
Update coordinators in `layer2_coordination/` to handle new analysis types:
- Extend `HypothesisTestingCoordinator` to route to new test functions
- Add `PowerAnalysisCoordinator` for power calculations

### Testing
Add comprehensive tests following existing patterns:
- Unit tests for each new function
- Integration tests for coordinator layer
- Property-based tests for edge cases
- Performance benchmarks for large datasets

## Integration Points

- **Frontend Integration**: New result types will be serialized via existing serde setup
- **Chart Data Preparation**: Extend chart data functions to support hypothesis test and power analysis visualizations
- **Error Handling**: Use existing `thiserror` based error types
- **Parallel Processing**: Use rayon for computationally intensive operations (e.g., power curves, bootstrap post-hoc tests)

## Priority Order

1. Extend `types.rs` with new result structures
2. Implement hypothesis testing functions (t-tests first, then ANOVA, chi-square)
3. Implement power analysis functions
4. Update coordinators and command layer
5. Add comprehensive tests
6. Frontend UI integration (separate phase)

## Dependency Graph of Functions

The following dependency graph shows the hierarchical relationships between functions used to calculate statistical results. Functions are organized by layer, with arrows indicating dependencies (higher layers depend on lower layers).

### Layer 3 Algorithms (hypothesis_testing.rs)

```
one_sample_t_test
├── calculate_mean (layer4)
├── calculate_variance (layer4)
└── t_distribution.cdf (statrs)

paired_t_test
├── calculate_mean (layer4)
├── calculate_variance (layer4)
├── calculate_covariance (layer4)
└── t_distribution.cdf (statrs)

two_sample_t_test
├── calculate_mean (layer4)
├── calculate_variance (layer4)
├── pooled_variance (layer4)
└── t_distribution.cdf (statrs)

welch_t_test
├── calculate_mean (layer4)
├── calculate_variance (layer4)
├── calculate_std_error_unequal_var (layer4)
└── t_distribution.cdf (statrs)

one_way_anova
├── calculate_mean (layer4)
├── calculate_variance (layer4)
├── calculate_sum_of_squares (layer4)
└── f_distribution.cdf (statrs)

two_way_anova
├── calculate_mean (layer4)
├── calculate_variance (layer4)
├── calculate_sum_of_squares (layer4)
├── calculate_interaction_ss (layer4)
└── f_distribution.cdf (statrs)

chi_square_goodness_of_fit
├── calculate_chi_square_statistic (layer4)
└── chi_squared_distribution.cdf (statrs)

chi_square_independence
├── calculate_contingency_chi_square (layer4)
└── chi_squared_distribution.cdf (statrs)
```

### Layer 4 Primitives (statistical_power.rs)

```
power_t_test
├── normal_distribution.cdf (statrs)
└── t_distribution.quantile (statrs)

sample_size_t_test
├── normal_distribution.quantile (statrs)
├── t_distribution.quantile (statrs)
└── calculate_power_from_quantiles (internal)

power_anova
├── f_distribution.cdf (statrs)
└── non_central_f_distribution.cdf (statrs)

power_chi_square
├── chi_squared_distribution.cdf (statrs)
└── non_central_chi_squared_distribution.cdf (statrs)

post_hoc_power
├── t_distribution.cdf (statrs)
├── f_distribution.cdf (statrs)
└── chi_squared_distribution.cdf (statrs)

power_curve_t_test
├── power_t_test (recursive for each n)
└── rayon::par_iter for parallel computation

power_curve_anova
├── power_anova (recursive for each n)
└── rayon::par_iter for parallel computation
```

### Layer 4 Primitives (shared utilities)

```
calculate_mean
└── ndarray operations

calculate_variance
├── calculate_mean
└── ndarray operations

calculate_std_dev
├── calculate_variance
└── f64::sqrt

calculate_covariance
├── calculate_mean
└── ndarray operations

pooled_variance
├── calculate_variance
└── weighted average calculation

calculate_std_error_unequal_var
├── calculate_variance
└── sample size calculations

calculate_sum_of_squares
├── calculate_mean
└── ndarray operations

calculate_interaction_ss
├── calculate_sum_of_squares
└── factorial calculations

calculate_chi_square_statistic
└── ndarray operations

calculate_contingency_chi_square
├── calculate_expected_frequencies (internal)
└── ndarray operations

calculate_power_from_quantiles
├── normal_distribution operations
└── t_distribution operations
```

### External Dependencies

All statistical functions depend on the `statrs` crate for probability distributions:
- `statrs::distribution::Normal`
- `statrs::distribution::StudentsT`
- `statrs::distribution::FisherSnedecor`
- `statrs::distribution::ChiSquared`
- `statrs::distribution::NonCentralF`
- `statrs::distribution::NonCentralChiSquared`

Parallel computations use `rayon` for performance:
- `rayon::prelude::*` for parallel iterators in power curve calculations

Array operations use `ndarray`:
- `ndarray::Array1` for data vectors
- Basic arithmetic operations

Random number generation (for simulations if needed) uses `rand`:
- `rand::Rng` for Monte Carlo methods in complex power analyses