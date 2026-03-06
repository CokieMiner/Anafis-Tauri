# AnaFis Statistical Module Enhancement Tickets

**Date**: November 20, 2025  
**Status**: Active Development  
**Priority**: High  
**Context**: AnaFis has achieved mathematical rigor and performance superiority over Python/R, but lacks algorithmic breadth for experimental data analysis workflows.

---

## 🎯 Executive Summary

AnaFis excels in **uncertainty quantification**, **performance**, and **spreadsheet integration** but needs **6 critical feature gaps** to achieve functional parity with Python/R for experimental workflows. These enhancements will make AnaFis functionally equivalent for 99% of experimental use cases while maintaining its unique advantages.

**Strategic Impact**: Transform AnaFis from "statistically rigorous" to "feature-complete for experimental science" - the "Excel for Scientists" that combines spreadsheet familiarity with professional statistical capabilities.

---

## 🏗️ Architecture & Code Quality Assessment

**Status**: ✅ **EXCELLENT - Production Ready** - Domain-driven design successfully implemented with verified dependencies and SOTA algorithms.

### ✅ Strengths
- **Domain-Driven Design**: `distributions`, `time_series`, `correlation` modules provide excellent organization
- **Trait-Based Architecture**: `StatisticalMoments` trait makes operations feel native to `Vec<f64>`
- **Pipeline Orchestration**: `pipeline.rs` automates metadata extraction, imputation, and test selection - UX advantage over Excel/Python
- **State-of-the-Art Algorithms**: FFT with rustfft, hybrid KD-Tree imputation, Pure Rust Prophet with MCMC, professional time series via oxidiviner
- **Dependencies Verified**: All crates are production-ready (oxidiviner, statrs, ndarray, rand)

### ⚠️ Critical Issues Identified

#### 1. **UnifiedStats Redundancy** 
**Issue**: `StatisticalMoments` trait AND `UnifiedStats` struct create duplication.
**Recommendation**: Deprecate `UnifiedStats`. Move logic to `HypothesisTest` trait or standalone functions in relevant modules.

#### 2. **Numerical Stability - CRITICAL BLOCKER**
**Issue**: Finite differences in `distributions/fitting.rs` and `uncertainty/propagation.rs` will fail catastrophically on physics functions with singularities.
**Solution**: Tickets 021-022 (Dual Numbers) are **mandatory prerequisites** - not optional enhancements.

#### 3. **Dependency Verification - RESOLVED ✅**
**Status**: All dependencies confirmed as production-ready public crates.
- `oxidiviner`: High-quality time series library with ARIMA, GARCH, ETS models
- Usage in `time_series/arima.rs` and `forecasting.rs` correctly delegates to specialized crate
- **Cargo.toml**: Add `oxidiviner = { version = "0.1", features = ["ndarray_support"] }`

### 🧪 Testing Strategy Requirements
**Gold Standard Testing**: Create CSV test suites from R/Python with known inputs/outputs (including edge cases like ties, collinearity, NaNs). Ensure AnaFis matches to 4 decimal places.

---

### Priority Levels
- 🔴 **CRITICAL**: Blocks core experimental workflows
- 🟠 **HIGH**: Essential for physics/experimental use cases
- 🟡 **MEDIUM**: Important for robustness/completeness
- 🟢 **LOW**: Nice-to-have for advanced users

### Implementation Phases
1. **Phase 0**: Baseline Testing (CRITICAL prerequisite - 2-3 weeks)
2. **Phase 1**: Non-parametric tests (immediate user pain point)
3. **Phase 2A**: Dual Numbers Infrastructure (MANDATORY prerequisite - numerical stability)
4. **Phase 2B**: General curve fitting (physics enablement - blocked by Phase 2A)
5. **Phase 3**: Categorical data support (UX critical)
6. **Phase 4**: Advanced post-hoc tests (robustness)
7. **Phase 5**: Multivariate analysis (exploratory)
8. **Phase 6**: GLM framework (advanced modeling)

---

## 🔴 CRITICAL PREREQUISITE: Baseline Testing

### TICKET-000: Comprehensive Baseline Test Suite
**Priority**: 🔴 CRITICAL  
**Phase**: 0  
**Effort**: High (2-3 weeks)  
**Impact**: Critical  

**Problem**:  
The existing statistics module lacks comprehensive tests. Before implementing enhancements, we must establish a solid baseline of working, tested functionality to prevent regressions and ensure precision.

**Requirements**:
- **Complete Coverage**: Test all existing statistical functions with gold standard comparisons
- **Precision Validation**: Ensure results match R/Python implementations to 4+ decimal places
- **Performance Benchmarks**: Establish baseline performance metrics
- **Edge Case Testing**: Handle NaN, Inf, empty datasets, single points, etc.
- **Integration Testing**: Test pipeline orchestration and data flow
- **Property-Based Testing**: Use statistical properties to validate implementations

**Test Categories**:

**1. Core Statistics (`primitives`, `descriptive`)**
- ✅ Statistical moments (mean, variance, skewness, kurtosis) vs R/Python
- ✅ Quantiles (Hyndman-Fan method) vs R `quantile()`
- ✅ Covariance/correlation matrices vs numpy/scipy
- ✅ Basic hypothesis tests (t-test, F-test) vs scipy.stats

**2. Distributions (`distributions`)**
- ✅ PDF/CDF/quantile functions vs scipy.stats
- ✅ MLE fitting accuracy vs scipy.stats
- ✅ Random sampling quality vs numpy
- ✅ Edge cases (boundary values, extreme parameters)

**3. Time Series (`time_series`)**
- ✅ FFT accuracy vs numpy.fft
- ✅ Spectral analysis vs scipy.signal
- ✅ Forecasting with oxidiviner vs statsmodels
- ✅ Changepoint detection vs ruptures library

**4. Regression (`regression`)**
- ✅ Robust regression vs statsmodels
- ✅ Uncertainty estimation vs scipy.optimize.curve_fit
- ✅ Residual analysis and diagnostics

**5. Uncertainty (`uncertainty`)**
- ✅ Propagation accuracy vs analytical derivatives
- ✅ Monte Carlo vs analytical methods
- ✅ Error propagation through complex expressions

**6. Preprocessing (`preprocessing`)**
- ✅ Imputation accuracy vs sklearn.impute
- ✅ Missing data handling edge cases
- ✅ Pipeline integration testing

**7. Pipeline (`pipeline`)**
- ✅ End-to-end workflow testing
- ✅ Metadata extraction accuracy
- ✅ Test selection logic validation

**Technical Implementation**:
- **Gold Standard Datasets**: CSV files with known R/Python outputs
- **Precision Requirements**: Match reference implementations to 1e-10 relative error
- **Performance Baselines**: Establish timing benchmarks for optimization
- **CI Integration**: Automated testing on every commit
- **Documentation**: Test results as living documentation of precision

**Acceptance Criteria**:
- ✅ All existing functions have comprehensive tests
- ✅ Results match R/Python references within numerical precision
- ✅ Performance benchmarks established and documented
- ✅ Edge cases handled gracefully
- ✅ CI pipeline prevents regressions
- ✅ Test coverage >90% for statistics module

**Files to Create**:
- `src/scientific/statistics/tests/` (comprehensive test suite)
- `tests/reference/` (reference datasets and expected outputs)
- `tests/benchmarks/` (performance baseline measurements)
- `tests/integration/` (end-to-end pipeline tests)

**BLOCKING**: All enhancement tickets are blocked until this baseline is established.

---

## 🔴 CRITICAL GAP: Non-Parametric Hypothesis Testing

### TICKET-001: Mann-Whitney U Test Implementation
**Priority**: 🔴 CRITICAL  
**Phase**: 1  
**Effort**: Low (2-3 days)  
**Impact**: High  

**Problem**:  
When Shapiro-Wilk normality test fails (common with experimental data), users have no alternative to independent t-tests. R/Python users immediately switch to Mann-Whitney U, but AnaFis offers no non-parametric options.

**Requirements**:
- Implement Mann-Whitney U statistic calculation
- Handle ties properly (average ranks) - **CRITICAL**: Ensure tie correction matches R's `wilcox.test` implementation
- Provide exact p-values for small samples, normal approximation for large samples
- Include effect size (r = |U - μ_U| / σ_U)
- Support one-tailed and two-tailed tests
- Confidence intervals for the effect size

**Acceptance Criteria**:
- ✅ Produces same results as `scipy.stats.mannwhitneyu()`
- ✅ Handles tied ranks correctly
- ✅ Provides accurate p-values
- ✅ Includes effect size and confidence intervals
- ✅ Integrated into hypothesis testing pipeline
- ✅ UI option appears when normality test fails

**Files to Modify**:
- `src/scientific/statistics/hypothesis_testing/mod.rs`
- `src/scientific/statistics/hypothesis_testing/types.rs`
- UI components for test selection

---

### TICKET-002: Wilcoxon Signed-Rank Test Implementation
**Priority**: 🔴 CRITICAL  
**Phase**: 1  
**Effort**: Low (2-3 days)  
**Impact**: High  

**Problem**:  
Paired t-test alternative missing. When paired data violates normality, users need Wilcoxon signed-rank test but AnaFis doesn't provide it.

**Requirements**:
- Handle paired differences (x_i - y_i)
- Rank absolute differences, ignore zero differences
- Calculate W statistic (sum of ranks for positive differences)
- Exact p-values for small samples, normal approximation for large
- Effect size (r = W / sqrt(N(N+1)/2))
- Handle ties and zero differences properly - **CRITICAL**: Ensure tie correction matches R's implementation

**Acceptance Criteria**:
- ✅ Matches `scipy.stats.wilcoxon()` results
- ✅ Properly handles zero differences and ties
- ✅ Accurate p-values and effect sizes
- ✅ Integrated into paired test selection UI

**Files to Modify**:
- `src/scientific/statistics/hypothesis_testing/mod.rs`
- `src/scientific/statistics/hypothesis_testing/types.rs`

---

### TICKET-003: Kruskal-Wallis H Test Implementation
**Priority**: 🔴 CRITICAL  
**Phase**: 1  
**Effort**: Low (3-4 days)  
**Impact**: High  

**Problem**:  
One-way ANOVA alternative missing. When k-group data violates normality assumptions, users need Kruskal-Wallis test for non-parametric k-group comparison.

**Requirements**:
- Rank all observations across groups
- Calculate H statistic with tie correction - **CRITICAL**: Ensure tie correction matches scipy.stats.kruskal implementation
- Chi-square approximation for p-values
- Eta-squared effect size
- Post-hoc pairwise comparisons (Dunn's test)
- Handle ties properly

**Acceptance Criteria**:
- ✅ Matches `scipy.stats.kruskal()` results
- ✅ Proper tie correction
- ✅ Includes post-hoc test results
- ✅ Effect size calculation
- ✅ UI integration with ANOVA selection

**Files to Modify**:
- `src/scientific/statistics/hypothesis_testing/mod.rs`
- `src/scientific/statistics/hypothesis_testing/types.rs`

---

### TICKET-004: Friedman Test Implementation
**Priority**: 🔴 CRITICAL  
**Phase**: 1  
**Effort**: Medium (4-5 days)  
**Impact**: Medium  

**Problem**:  
Repeated measures ANOVA alternative missing. For non-parametric repeated measures designs, Friedman test is essential but unavailable.

**Requirements**:
- Rank within subjects/blocks
- Calculate chi-square statistic
- Handle ties properly - **CRITICAL**: Ensure tie correction matches scipy.stats.friedmanchisquare
- Post-hoc pairwise comparisons (Nemenyi test for multiple groups)
- Kendall's W effect size
- Integration with repeated measures workflow

**Acceptance Criteria**:
- ✅ Matches `scipy.stats.friedmanchisquare()` results
- ✅ Proper tie handling
- ✅ Post-hoc capabilities
- ✅ Effect size calculation

**Files to Modify**:
- `src/scientific/statistics/hypothesis_testing/mod.rs`
- `src/scientific/statistics/hypothesis_testing/types.rs`

---

## 🟠 HIGH PRIORITY: Generalized Linear Models & Curve Fitting

### TICKET-005: General Regression Engine (Multi-dimensional, Multiple Methods)
**Priority**: 🟠 HIGH  
**Phase**: 2  
**Effort**: High (3-4 weeks)  
**Impact**: Critical  

**Problem**:  
AnaFis lacks comprehensive regression capabilities. Current robust_regression only does linear/polynomial fitting with limited methods. Experimental scientists need to fit complex physical models with multiple variables and parameters using various regression approaches. **Physics is distinct from general data science because errors in independent variables (x) are just as common as errors in dependent variables (y)**. 

*(Note: The base ODR engine with Levenberg-Marquardt has now been implemented in `src/scientific/curve_fitting`, fulfilling a large part of this requirement. The remaining work is extending this to other methods and integrating it fully with the UI.)*

**CRITICAL DEPENDENCY**: Must be implemented AFTER Tickets 021-022 (Dual Numbers). Building ODR on finite differences will fail catastrophically on experimental data with singularities or flat regions.

**Requirements**:
- **Multi-dimensional regression**: Fit equations f(x₁, x₂, ..., xₙ; θ₁, θ₂, ..., θₘ) where x are independent variables and θ are parameters
- **All methods support arbitrary functions**: Every regression method (ODR, WLS, robust, non-linear) must work with any user-defined function via string parsing or closures
- **Mixed uncertainty handling**: Support scenarios where some variables have uncertainties and others don't:
  - Variables with known σ: Use full error propagation
  - Variables without uncertainties: Assume zero error or use default weighting
  - Graceful degradation: Methods automatically adjust when uncertainty information is partial
- **Flexible regression methods with automatic fallbacks**:
  - **Orthogonal Distance Regression (ODR)**: Handles errors in both x and y; falls back to WLS when x-errors are zero; supports arbitrary functions; uses SVD for linear models, iterative solver for non-linear
  - **Weighted Least Squares (WLS)**: For heteroscedastic errors; falls back to OLS when weights are uniform; supports non-linear functions
  - **Robust regression methods**: Huber, Tukey biweight, etc.; fall back to standard methods when robustness not needed; handle arbitrary functions
  - **Quantile Regression (LAD)**: Least Absolute Deviations for median fitting; robust alternative to mean-based methods; supports arbitrary quantiles (τ = 0.5 for median)
  - **Regularized Regression**: Ridge, Lasso, ElasticNet for multicollinear data; adds penalty terms to prevent overfitting
  - **Non-linear least squares**: Levenberg-Marquardt for arbitrary functions; falls back to linear methods for linear functions
- **Transparent auto-selection with user override**: 
  - Automatic method selection based on data characteristics
  - UI shows selected method and reasoning: *"Detected errors in X and Y: Defaulting to ODR. (Click to change)"*
  - Never switches methods silently
- **User-defined function interface**: Support for custom equations via:
  - String expressions: `"a*sin(b*x) + c"`
  - Built-in models with hard-coded Jacobians for performance
- **Performance optimizations**:
  - **Forward-mode automatic differentiation** using dual numbers for exact Jacobians (machine precision, no numerical errors)
  - Hard-coded analytical Jacobians for built-in models (fastest)
  - Dual number evaluation for user-defined functions (exact derivatives, as fast as function evaluation)
  - AST-based expression evaluator supporting generic types for dual number computation
- **Parameter constraints and bounds**
- **Uncertainty estimation**: Bootstrap confidence intervals for parameters
- **Goodness-of-fit diagnostics**: R², RMSE, AIC, BIC, reduced chi-square, residual analysis
- **Built-in common models** (with analytical Jacobians):
  - Linear: `y = a*x + b`
  - Polynomial: `y = a0 + a1*x + a2*x² + ...`
  - Exponential: `y = A*exp(-k*x) + C`
  - Gaussian: `y = A*exp(-(x-μ)²/(2σ²)) + C`
  - Lorentzian: `y = A/(1 + ((x-μ)/γ)²) + C`
  - Hill equation: `y = V_max * x^n / (K^n + x^n)`
  - Michaelis-Menten: `v = V_max * [S] / (K_m + [S])`
  - Multi-variable: `z = f(x, y; a, b, c)`

**TECHNICAL CHALLENGES**:
- **ODR Complexity**: Minimizing errors in both X and Y creates much larger optimization problems (parameters + N corrections for X)
- **Performance**: For user-defined functions in optimization loops, consider JIT compilation (cranelift) or fast bytecode interpreter instead of AST traversal per optimizer iteration
- **Numerical Stability**: Exact derivatives via dual numbers are mandatory - finite differences will fail on physics functions with singularities

**Acceptance Criteria**:
- ✅ Handles multi-dimensional inputs (n independent variables)
- ✅ Supports multiple regression methods (WLS, ODR, robust, quantile, regularized)
- ✅ User-defined function interface for arbitrary equations (string + closure support)
- ✅ Transparent method selection with user override
- ✅ Parameter uncertainties via bootstrap
- ✅ Comprehensive goodness-of-fit diagnostics
- ✅ Matches results from `scipy.odr`, `scipy.optimize.curve_fit`, `statsmodels`
- ✅ UI for equation specification and method selection with reasoning display
- ✅ Performance acceptable for complex models (1000+ data points)
- ✅ Exact derivatives via dual numbers (machine precision, no numerical errors)
- ✅ Hard-coded Jacobians for built-in models provide optimal performance

**Technical Implementation**:
- **Unified Cost Function Architecture**: Single `GenericRegressionCost` struct passed to argmin optimizer
- **Residual Modes**: `Vertical` (standard) and `Orthogonal` (for ODR)
- **Loss Functions**: 
  - `Squared`: Standard Least Squares
  - `Absolute`: Quantile Regression (LAD) - Robust to Y outliers
  - `Huber(f64)`: Robust regression
  - `Tukey(f64)`: Robust regression
- **Regularization Penalties**:
  - `None`: Standard regression
  - `Ridge(f64)`: L2 penalty λ×∑θ²
  - `Lasso(f64)`: L1 penalty λ×∑|θ|
  - `ElasticNet(f64, f64)`: Mixed L1/L2 penalties
- **Mathematical Interface**: All regression problems minimize ∑Loss(Residualᵢ) + Penalty(θ)
- **3-Stage Implementation**:
  - **Stage 1**: Non-linear least squares foundation (`scipy.optimize.curve_fit` equivalent)
  - **Stage 2**: ODR upgrade + dual number autodiff (`scipy.odr` equivalent)  
  - **Stage 3**: Robustness & advanced diagnostics
- Use `argmin` crate for optimization framework
- Implement ODR using orthogonal distance formulation
- Bootstrap for uncertainty quantification
- Parallel computation for bootstrap samples

---

### TICKET-006: Logistic Regression Implementation
**Priority**: 🟠 HIGH  
**Phase**: 6  
**Effort**: Medium (1-2 weeks)  
**Impact**: High  

**Problem**:  
Binary outcome modeling missing. For pass/fail, survival/death, or other binary experimental outcomes, logistic regression is essential.

**Requirements**:
- Maximum likelihood estimation
- Logit link function
- Odds ratios and confidence intervals
- Model diagnostics (Hosmer-Lemeshow test)
- ROC curves and AUC
- Classification metrics (precision, recall, F1)
- Support for categorical predictors

**Acceptance Criteria**:
- ✅ Matches `statsmodels.api.Logit` results
- ✅ Proper convergence and diagnostics
- ✅ Odds ratio interpretation
- ✅ ROC curve generation

**Files to Modify**:
- `src/scientific/statistics/regression/mod.rs` (extend)
- `src/scientific/statistics/regression/types.rs`

---

### TICKET-007: Poisson Regression Implementation
**Priority**: 🟠 HIGH  
**Phase**: 6  
**Effort**: Medium (1-2 weeks)  
**Impact**: Medium  

**Problem**:  
Count data modeling missing. For event counts, particle counts, or other Poisson-distributed experimental data.

**Requirements**:
- Log link function
- Incident rate ratios
- Overdispersion detection
- Zero-inflation handling (optional)
- Goodness-of-fit tests

**Acceptance Criteria**:
- ✅ Matches `statsmodels.api.Poisson` results
- ✅ Proper handling of count data
- ✅ Rate ratio interpretation

**Files to Modify**:
- `src/scientific/statistics/regression/mod.rs` (extend)

---

## 🟡 MEDIUM PRIORITY: Categorical Data Processing

### TICKET-008: Categorical Encoding Engine
**Priority**: 🟡 MEDIUM  
**Phase**: 3  
**Effort**: Medium (1-2 weeks)  
**Impact**: Critical  

**Problem**:  
AnaFis regression/ANOVA expects numeric indices, not categorical strings. Users must manually convert "Treatment A/B/C" to "0/1/2", breaking spreadsheet-native workflow.

**Requirements**:
- **String-to-Number Conversion**: Accepts `Vec<String>` and converts to `Vec<f64>` (One-Hot Encoding/Dummy Variables)
- **Dummy Variable Trap Prevention**: For k categories, generate k-1 columns to avoid multicollinearity (X^T X singular matrix)
- Automatic categorical column detection from user-selected data ranges
- Reference level selection for dummy encoding
- Handle missing categories gracefully
- Memory-efficient for large datasets (100k+ rows)
- **Dumb Backend Pattern**: Receives structured data from UI (no formula parsing)
- Integration with ANOVA (group labels) and regression preprocessing

**Acceptance Criteria**:
- ✅ Converts categorical strings to numeric matrices efficiently
- ✅ Handles missing data appropriately
- ✅ Memory-efficient for large datasets
- ✅ Integrates with regression and ANOVA workflows
- ✅ No manual preprocessing required for selected ranges

**Technical Implementation**:
- Receives `RegressionRequest` struct with `x_categorical: Vec<Vec<String>>`
- Returns design matrix with one-hot encoded columns
- Memory-efficient processing for large datasets

**Files to Create**:
- `src/scientific/statistics/preprocessing/categorical.rs`
- `src/scientific/statistics/preprocessing/design_matrix.rs`

---

### TICKET-009: Feature Engineering Engine
**Priority**: 🟡 MEDIUM  
**Phase**: 3  
**Effort**: Low (3-5 days)  
**Impact**: High  

**Problem**:  
Cannot automatically create interaction terms like "Treatment × Age" for regression models from structured UI instructions.

**Requirements**:
- **Instruction-Driven Interactions**: Receives `interactions: Vec<(usize, usize)>` instructions (e.g., "Multiply X_col[0] with X_col[1]")
- Generate interaction columns ($X_1 \times X_2$) from numeric data
- Polynomial interactions (x₁ × x₂)
- Higher-order interactions (x₁ × x₂ × x₃)
- Memory-efficient computation for large datasets
- **Dumb Backend Pattern**: No formula parsing, just executes UI-generated instructions
- Integration with categorical encoding engine

**Acceptance Criteria**:
- ✅ Generates correct interaction columns from instruction tuples
- ✅ Handles memory constraints for large datasets
- ✅ Integrates with regression preprocessing pipeline
- ✅ No formula parsing required

**Technical Implementation**:
- Receives `RegressionRequest` with `interactions: Vec<(usize, usize)>`
- Generates multiplied columns before fitting
- Memory-efficient processing

**Files to Modify**:
- `src/scientific/statistics/preprocessing/design_matrix.rs`

---

## 🟡 MEDIUM PRIORITY: Multivariate Exploratory Analysis

### TICKET-011: K-Means Clustering Implementation
**Priority**: 🟢 LOW  
**Phase**: 5  
**Effort**: Medium (1-2 weeks)  
**Impact**: Medium  

**Problem**:  
No clustering for exploratory data analysis. Users cannot group similar experimental observations.

**Requirements**:
- K-means algorithm with k-means++ initialization
- Elbow method for optimal k selection
- Silhouette analysis for cluster validation
- Parallel implementation for performance
- Visualization support (cluster plots)

**Acceptance Criteria**:
- ✅ Matches `sklearn.cluster.KMeans` results
- ✅ Efficient for large datasets
- ✅ Cluster validation metrics
- ✅ Visualization integration

**Files to Create**:
- `src/scientific/statistics/clustering/mod.rs`
- `src/scientific/statistics/clustering/kmeans.rs`

---

### TICKET-012: Hierarchical Clustering with Dendrograms
**Priority**: 🟢 LOW  
**Phase**: 5  
**Effort**: Medium (1-2 weeks)  
**Impact**: Medium  

**Problem**:  
No hierarchical clustering for understanding data structure relationships.

**Requirements**:
- Single/complete/average linkage methods
- Distance metrics (Euclidean, Manhattan, etc.)
- Dendrogram generation
- Cluster cutting at specified heights
- Memory-efficient for large datasets

**Acceptance Criteria**:
- ✅ Matches `scipy.cluster.hierarchy` results
- ✅ Dendrogram visualization
- ✅ Multiple linkage methods
- ✅ Cluster extraction

**Files to Modify**:
- `src/scientific/statistics/clustering/mod.rs`

---

### TICKET-013: t-SNE/UMAP Dimensionality Reduction
**Priority**: 🟢 LOW  
**Phase**: 5  
**Effort**: High (2-3 weeks)  
**Impact**: Low  

**Problem**:  
PCA covers most physics use cases, but t-SNE/UMAP becoming standard for complex datasets.

**Requirements**:
- t-SNE implementation (Barnes-Hut approximation)
- UMAP implementation
- Perplexity optimization for t-SNE
- Visualization integration
- Performance optimization

**Acceptance Criteria**:
- ✅ Reasonable results on test datasets
- ✅ Performance acceptable for medium datasets
- ✅ Visualization integration

**Files to Create**:
- `src/scientific/statistics/dimensionality_reduction/mod.rs`

---

## 🟢 LOW PRIORITY: Survival Analysis Extensions

### TICKET-014: Kaplan-Meier Estimator
**Priority**: 🟢 LOW  
**Effort**: Medium (1-2 weeks)  
**Impact**: Low  

**Problem**:  
Reliability module focuses on psychometrics, not engineering survival analysis.

**Requirements**:
- Handle censored data
- Survival curve estimation
- Confidence intervals
- Median survival times
- Log-rank test integration

**Acceptance Criteria**:
- ✅ Matches `lifelines.KaplanMeierFitter` results
- ✅ Proper censoring handling
- ✅ Confidence intervals

**Files to Create**:
- `src/scientific/statistics/survival/mod.rs`

---

### TICKET-015: Log-Rank Test for Survival Curves
**Priority**: 🟢 LOW  
**Effort**: Low (3-5 days)  
**Impact**: Low  

**Problem**:  
Cannot compare survival curves between groups.

**Requirements**:
- Mantel-Haenszel chi-square test
- Handle stratified data
- P-value calculation
- Effect size measures

**Acceptance Criteria**:
- ✅ Matches `lifelines.logrank_test` results
- ✅ Multiple groups support

**Files to Modify**:
- `src/scientific/statistics/survival/mod.rs`

---

## 🟡 MEDIUM PRIORITY: Advanced Post-Hoc Tests

### TICKET-016: Games-Howell Post-Hoc Test
**Priority**: 🟡 MEDIUM  
**Phase**: 4  
**Effort**: Low (3-5 days)  
**Impact**: Medium  

**Problem**:  
Tukey HSD assumes equal variances, but AnaFis detects unequal variances via Levene/Bartlett tests without providing solutions.

**Requirements**:
- Games-Howell procedure for unequal variances
- Individual confidence intervals
- P-value adjustments
- Integration with ANOVA results

**Acceptance Criteria**:
- ✅ Matches `scipy.stats.posthoc_gameshowell` results
- ✅ Proper unequal variance handling
- ✅ Integrated with ANOVA workflow

**Files to Modify**:
- `src/scientific/statistics/hypothesis_testing/mod.rs`

---

### TICKET-017: Dunnett's Test Implementation
**Priority**: 🟡 MEDIUM  
**Phase**: 4  
**Effort**: Low (3-5 days)  
**Impact**: Medium  

**Problem**:  
Common experimental design (multiple treatments vs control) lacks dedicated test.

**Requirements**:
- One-sided t-tests with control group
- Step-down procedure
- Family-wise error control
- Integration with ANOVA framework

**Acceptance Criteria**:
- ✅ Matches `scipy.stats.dunnett` results
- ✅ Proper multiple comparison control
- ✅ Control group specification

**Files to Modify**:
- `src/scientific/statistics/hypothesis_testing/mod.rs`

---

## 🔧 Infrastructure & Testing Tickets

### TICKET-021: Upgrade Uncertainty Propagation to Dual Numbers
**Priority**: 🔴 CRITICAL  
**Phase**: 2  
**Effort**: Medium (2-3 weeks)  
**Impact**: Critical  

**Problem**:  
Uncertainty propagation currently uses finite differences (h = 1e-8) for numerical differentiation, which introduces approximation errors and stability issues. For complex functions, this can lead to inaccurate uncertainty estimates. **CRITICAL RISK**: In physics applications, functions often have singularities or flat regions where finite differences suffer from catastrophic cancellation, making the entire uncertainty quantification unreliable.

**Requirements**:
- **MANDATORY**: Replace finite difference derivatives with forward-mode automatic differentiation using dual numbers
- Support arbitrary user-defined functions for uncertainty propagation
- Maintain backward compatibility with existing analytical derivative functions
- Provide exact derivatives (machine precision) instead of numerical approximations
- Handle complex expressions like `f(x,y) = sin(x)*exp(y) + x²/y`
- **Special Functions Support**: Ensure AST parser supports physics special functions (Bessel, Gamma, Error functions) with dual number rules
- **BLOCKING DEPENDENCY**: Must be completed before implementing General Regression (Ticket 005) - building ODR on finite differences will fail catastrophically on experimental data

**Acceptance Criteria**:
- ✅ Uncertainty propagation matches analytical results for test functions
- ✅ No numerical differentiation errors or step-size selection issues
- ✅ Performance comparable to current implementation
- ✅ Supports arbitrary user-defined functions via string expressions
- ✅ Maintains existing API for functions with known analytical derivatives
- ✅ **GOLD STANDARD**: Results match symbolic differentiation for physics functions with singularities

**Technical Implementation**:
- Use `num-dual` crate for forward-mode autodiff
- Create AST-based expression evaluator supporting generic types
- Dual numbers: (value, derivative) pairs with automatic differentiation rules
- Integration with existing covariance matrix computations
- **Performance Consideration**: For user-defined functions in optimization loops, consider JIT compilation (cranelift) or fast bytecode interpreter instead of AST traversal per iteration

**Dependencies**:
- Add `num-dual = "0.7"` for dual number computations
- Add `symb_anafis` (CompiledEvaluator) for expression evaluation

**Files to Modify**:
- `src/scientific/statistics/uncertainty/propagation.rs` (replace finite differences with dual numbers)
- `src/scientific/statistics/uncertainty/ast/` (new module for expression parsing)

---

### TICKET-022: Upgrade Distribution Fitting to Dual Numbers
**Priority**: 🔴 CRITICAL  
**Phase**: 2  
**Effort**: Medium (2-3 weeks)  
**Impact**: Critical  

**Problem**:  
Distribution fitting uses argmin optimization with numerical gradients, leading to slower convergence and potential stability issues for complex distributions like Johnson SU or Burr Type XII. **CRITICAL RISK**: Finite differences fail catastrophically on distributions with singularities or flat regions common in physics applications.

**Requirements**:
- **MANDATORY**: Integrate dual number autodiff for exact parameter derivatives in MLE optimization
- Support for custom distribution functions with autodiff
- Improved fitting accuracy and convergence for heavy-tailed distributions
- Better handling of complex distribution shapes with exact derivatives
- **Special Functions**: Support physics distributions (Bessel, Gamma, Error functions) with dual number derivatives
- **BLOCKING DEPENDENCY**: Must be completed alongside Ticket 021 - distribution fitting stability is critical for experimental workflows

**Acceptance Criteria**:
- ✅ Distribution fitting results identical to current implementation for simple cases
- ✅ Improved convergence speed and stability for complex distributions
- ✅ No numerical gradient issues or step-size problems
- ✅ All existing distributions still supported
- ✅ **GOLD STANDARD**: Matches symbolic MLE for physics distributions with exact derivatives

**Technical Implementation**:
- Modify CostFunction implementations to use dual numbers instead of finite differences
- AST-based PDF evaluation for user-defined distributions
- Integration with existing GlobalOptimizer
- **Performance**: Dual numbers provide exact gradients without numerical instability

**Dependencies**:
- Add `num-dual = "0.7"` for gradient computation
- Use `symb_anafis` for expression parsing

**Files to Modify**:
- `src/scientific/statistics/distributions/fitting.rs` (upgrade CostFunction implementations)
- `src/scientific/statistics/distributions/mod.rs` (add AST support)

---

### TICKET-023: Upgrade ARIMA Optimization to Dual Numbers
**Priority**: 🟢 LOW  
**Phase**: 5  
**Effort**: Low (1-2 weeks)  
**Impact**: Low  

**Problem**:  
ARIMA fitting uses simplified gradient descent with manual derivative approximations, leading to suboptimal parameter estimates and slower convergence.

**Requirements**:
- Replace manual gradient computations with dual number autodiff
- Exact derivatives for AR and MA coefficient estimation
- Improved convergence for ARIMA model fitting
- Better parameter stability and accuracy
- **Robust Optimization**: Keep hybrid approach (gradient-based + derivative-free methods) due to complex ARIMA likelihood surfaces

**Acceptance Criteria**:
- ✅ ARIMA fitting results comparable or better than current implementation
- ✅ Improved convergence speed and stability
- ✅ No manual derivative coding required
- ✅ Maintains existing ARIMA API

**Technical Implementation**:
- Use dual numbers for ARMA coefficient updates
- Automatic differentiation for log-likelihood gradients
- Integration with existing Yule-Walker initialization

**Dependencies**:
- Add `num-dual = "0.7"` for gradient computation

**Files to Modify**:
- `src/scientific/statistics/time_series/arima.rs` (replace manual gradients with dual numbers)

---

### TICKET-018: Statistical Test Suite Expansion
**Priority**: 🟡 MEDIUM  
**Effort**: Medium (1 week)  
**Impact**: High  

**Problem**:  
Need comprehensive test suite comparing against scipy.stats and statsmodels.

**Requirements**:
- Unit tests for all new statistical functions
- Comparison tests against Python/R reference implementations
- Edge case testing (small samples, ties, etc.)
- Performance benchmarks
- Accuracy validation
- **Gold Standard**: CSV test suites from R/Python with known inputs/outputs matching to 4 decimal places

**Acceptance Criteria**:
- ✅ All new functions have comprehensive tests
- ✅ Results match reference implementations within tolerance
- ✅ Performance benchmarks established

**Files to Create/Modify**:
- `src/scientific/statistics/tests/` (expand)

---

### TICKET-019: UI Integration for New Tests
**Priority**: 🟡 MEDIUM  
**Effort**: Medium (1-2 weeks)  
**Impact**: High  

**Problem**:  
New statistical methods need UI integration.

**Requirements**:
- Test selection UI updates
- Parameter input forms
- Result display components
- Visualization integration
- Help/documentation

**Acceptance Criteria**:
- ✅ All new tests accessible via UI
- ✅ Proper parameter validation
- ✅ Results clearly displayed
- ✅ Help text provided

**Files to Modify**:
- UI components for statistical analysis

---

### TICKET-020: Documentation Updates
**Priority**: 🟡 MEDIUM  
**Effort**: Low (3-5 days)  
**Impact**: Medium  

**Problem**:  
Documentation needs updates for new features.

**Requirements**:
- Update API documentation
- Add examples for new methods
- Update comparison charts
- User guide updates

**Acceptance Criteria**:
- ✅ Documentation current and accurate
- ✅ Examples provided for all new features

**Files to Modify**:
- `docs/`
- `README.md`
- Inline documentation

---

## 📊 Implementation Metrics & Success Criteria

### Phase Completion Targets
- **Phase 0** (Baseline Testing): 2-3 weeks, **CRITICAL** - establishes tested foundation
- **Phase 1** (Non-parametric): 2-3 weeks, enables robust experimental workflows
- **Phase 2A** (Dual Numbers): 2-3 weeks, **CRITICAL** - enables numerical stability for physics
- **Phase 2B** (Curve fitting): 3-4 weeks, enables physics model fitting (blocked by 2A)
- **Phase 3** (Categorical): 2-3 weeks, enables spreadsheet-native UX
- **Phase 4** (Post-hoc): 1 week, completes ANOVA robustness
- **Phase 5** (Multivariate): 2-3 weeks, enables exploratory analysis
- **Phase 6** (GLM): 4-6 weeks, advanced modeling capabilities

### Feature Parity Metrics
- **80% Python/R feature parity** for experimental workflows
- **100% coverage** of common statistical tests used in physics/engineering
- **Superior uncertainty quantification** compared to Python/R
- **Better performance** than Python/R for numerical computations
- **Seamless spreadsheet integration** unmatched by Python/R

### Quality Assurance
- All functions match reference implementations (scipy.stats, statsmodels, R)
- Comprehensive test coverage (>90%)
- Performance benchmarks established
- Memory efficient for large datasets (10,000+ points)
- Static analysis tools integrated (Clippy, Miri, Rudra)

---

## 🎯 Strategic Impact Assessment

**Before Implementation**: AnaFis is "mathematically superior but functionally limited" - great for uncertainty but missing basic experimental workflow support.

**After Implementation**: AnaFis becomes "feature-complete for experimental science" - the "Excel for Scientists" that combines:
- ✅ Spreadsheet familiarity (unique advantage)
- ✅ Professional statistical rigor (matches Python/R)
- ✅ Superior uncertainty quantification (beats Python/R)
- ✅ **Exact mathematical derivatives** via dual numbers (beats Python/R numerical methods)
- ✅ Better performance (beats Python/R)
- ✅ Modern UX (beats Python/R complexity)

**Market Positioning**: Fill the $X billion gap between expensive/complex commercial software (Igor Pro, Origin) and inadequate free tools (Excel, basic Python scripts), targeting experimental scientists who need reliable results without becoming programmers.

**Cargo.toml Note**: Add `oxidiviner = { version = "0.1", features = ["ndarray_support"] }` for time series functionality.

---

*This ticket system provides a comprehensive roadmap to transform AnaFis from a promising statistical engine into the definitive tool for experimental data analysis.*