# AnaFis Master Roadmap

## 1. Project Overview & Architecture
ANAFIS is a detachable-notebook desktop application for scientific data analysis, featuring a Spreadsheet, Curve-Fitting, and Equation Solver. It relies on a "Dumb Backend / Smart Engine" architecture where Univer.js (React) handles UI and Tauri/Rust (`symb_anafis`, `nalgebra`) handles all heavy computation.

### Code Rules & Design
- **Frontend**: React/TS, Biome (lint/format), Plotly. State via Zustand.
- **Backend**: Rust, `wgpu`, `rayon`.
- **Differentiation**: `symb_anafis` CAS is the primary differentiation engine. Finite differences are the current fallback where CAS doesn't apply. Hyperdual numbers are earmarked for specific cases genuinely requiring 2nd-order derivatives (e.g. Hessian-based uncertainty propagation, curvature in nonlinear optimization).
- **Principles**: Heavy use of existing Rust crates, pure functional frontend processing, data flows via Tauri IPC.

### Current Status (Completed Infrastructure)
* **Core Systems**: Tab management, SQLite Data Library (FTS5), 10-format Import/Export system, file associations (`.anafispread`).
* **Sidebars**: Uncertainty Propagation, Unit Conversion, Quick Plot (Plotly).
* **Math Backends**: ODR engine with Levenberg-Marquardt, `symb_anafis` CAS.
* **Code Health**: 100% strict TS, 0 Biome issues, Clippy compliant.

---

## 2. Mathematical Curve & Distribution Fitting
The core backend supports multi-layer ODR, Poisson weighting, and point correlations. The UI must now be cleanly extended.

### Pending UI Enhancements
- **Histogram Mode (Distribution Fitting)**: Add a toggle to automatically enable Poisson weighting (1/√N), ask for 1 data column, and generate bins for fitting user-defined PDFs to counts via `fit_custom_odr`.
- **Multi-layer & Variable Definitions UI**: Allow fitting a system of equations where intermediate variables are defined by their own formulas (e.g., `y = a * x1 + c`, `x1 = b * x2`).
- **Point Correlations**: Allow users to supply a covariance matrix/correlation slider for datasets where points are not independent.
- **Advanced Global Optimization**: Introduce simulated annealing loops for complex energy landscapes where standard Levenberg-Marquardt gets stuck. Allow bounding parameter constraints (e.g., $p > 0$) using penalty functions.

---

## 3. Automatic Uncertainty Propagation Plugin
Cell modifications in Univer are unfeasible due to constraints. The solution is an **Uncertainty Univer Plugin** to hook natively into the system without breaking functionality.

### Core Design
- **Custom Renderers**: Introduce a cell border/overlay (e.g., blue for uncertain, green for correlated) and draw compact notation (e.g., `5.0 ± 0.1` or `5.0(1)`).
- **Custom Data Types**: Define `UncertaintyValue` and `CorrelatedUncertainty` safely within the extension registry.
- **Formula Interception**: The plugin catches operations like `=A1 + B1`, spots uncertainty types, delegates calculation to the Rust Tauri Backend, and saves the new uncertainty type directly to the result cell.
- **Correlations**: Keep a global Map of covariance matrices. Variables derived from the same source properly cancel/compound error during backend recalculations.

### Integration Plan
1. Initialize `UncertaintyPlugin` with `UncertaintyValue` and `UncertaintyCellRenderer`.
2. Tie the `FormulaParserService` to intercept uncertainty cells. 
3. Pass raw formulas to `propagate_uncertainty(formula, inputs)` via Tauri IPC.
4. Create a global matrix editor sidebar for cross-cell $\rho$ values.
5. Permit the spreadsheet to seamlessly use both this continuous plugin and the manual "target cell" sidebar logic simultaneously.

---

## 4. Analytical Tools & Sidebars

Every sidebar strictly follows the established pattern: **Univer.js** is the single source of truth. Typescript is UI (zero math), Rust computes the Math via Tauri IPC, and **Plotly** provides the visualizations.

### Statistical Analysis Sidebar
Provide comprehensive statistical capabilities using rigorous Bayesian Estimation instead of legacy P-values and point estimates.
1. **Descriptive Statistics**: Bayesian Bootstrap (resampling weights via Dirichlet distributions for smoother posteriors) replacing standard bootstrap.
2. **Correlation**: Distance Correlation (for non-linear structures), Mutual Information (KNN), and Errors-In-Variables regression to avoid Dilution Bias.
3. **Hypothesis Testing**:
   - Fit full posterior distributions to groups, returning $P(A > B)$ rather than binary null-rejections.
   - Exact Permutation Tests for absolutely non-parametric physical shuffling when priors are uninformative.
4. **Power Analysis**: Calculate sample sizes vs effect sizes constraint matching.
- **Architecture**: Rust computations via MCMC/Variational Inference handling the posterior sampling natively. Derivatives via `symb_anafis` CAS; finite differences as fallback. Must run `validate_assumptions(data)` before evaluating tests to correct users away from mathematically invalid correlations (e.g. Pearson on curves).

### Data Smoothing/Filtering Sidebar
Apply smoothing and filtering algorithms to noisy experimental data to improve signal quality.
1. **Smoothing Filters**: Simple/Exponential/Weighted Moving Averages, Savitzky-Golay (preserves peaks), LOWESS, Gaussian.
2. **Frequency Filters**: Low-pass, High-pass, Band-pass, Notch Filters (utilizing `rustfft`).
3. **Live Previews**: Side-by-side Plotly preview overlaying original vs smoothed curves along with RMS noise reduction readouts.
- **Architecture**: TS dispatches configs. Rust leverages `ndarray`, `nalgebra`, and `rustfft`.

### Outlier Detection Sidebar
Identify and handle anomalous data points systematically using weighted and deep learning approaches.
1. **Detection Methods**: Weighted Isolation Forest (utilizes sample weights derived from inverse uncertainty so noisy distributions aren't unfairly penalized), along with rigorous Conformal Outlier bounding.
2. **Remedial Actions**: Flag (paint background red), Remove (drop rows), Interpolate (Linear/Spline), or Replace (Median/NaN).
- **Architecture**: Rust backend computes isolation trees and probability bounds; frontend directly applies styling via Univer formatting APIs.
