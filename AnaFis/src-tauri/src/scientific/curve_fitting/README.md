# AnaFis Scientific Curve-Fitting Engine

This directory contains the core numerical engine for high-fidelity curve fitting in AnaFis. It implements a **profiled Orthogonal Distance Regression (ODR)** solver designed for metrological applications, strictly following the **Guide to the Expression of Uncertainty in Measurement (GUM / JCGM 100:2008)** and its supplements.

---

## Table of Contents

1. [Core Methodology: Profiled ODR](#core-methodology-profiled-odr)
2. [Mathematical Formulation](#mathematical-formulation)
3. [Optimization Strategy](#optimization-strategy)
4. [Architectural Components](#architectural-components)
5. [GUM-Compliant Inference Pipeline](#gum-compliant-inference-pipeline)
6. [Pipeline Flow](#pipeline-flow)
7. [Module Guide](#module-guide)
8. [Numerical Stability & Safeguards](#numerical-stability--safeguards)
9. [Performance Optimizations](#performance-optimizations)
10. [Diagnostic Monitoring](#diagnostic-monitoring)
11. [Known Limitations & Design Trade-offs](#known-limitations--design-trade-offs)
12. [Computational Infrastructure](#computational-infrastructure)

---

## Core Methodology: Profiled ODR

AnaFis solves the fitting problem by minimizing the total weighted distance between the model and the data, accounting for uncertainties in both dependent (Y) and independent (X) variables. Unlike classical least-squares (which assumes error-free predictors), ODR produces unbiased parameter estimates when independent variables are also subject to measurement error — a requirement in most metrological and scientific contexts.

The engine uses a **profiled** (variable-projection) strategy: the latent per-point corrections δ are eliminated in an inner loop, and the outer optimizer only operates on the global parameters β. This reduces the outer problem dimension from `(P + N·C)` to just `P`, where `P` is the number of parameters, `N` the number of data points, and `C` the number of correctable independent variables. The profiled gradient and Hessian incorporate the implicit dependence of δ*(β) on β via the implicit function theorem.

---

## Mathematical Formulation

### The Objective Function

The engine minimizes the **profiled chi-squared objective**:

$$\chi^2(\beta, \delta) = \sum_{i=1}^N \left[ (y_i - f(x_i + \delta_i, \beta))^T W_{y,i} (y_i - f(x_i + \delta_i, \beta)) + \delta_i^T W_{x,i} \delta_i \right]$$

where:
- **β** are the global model parameters (shared across all data points and layers).
- **δ_i** are the latent corrections for the independent variables at point `i`. These represent the unknown "true" positions of the predictors on the model surface.
- **W** are the **inverse covariance matrices** (weight matrices). When correlations between independent and dependent variables exist at a point, a **joint covariance block** Σ_ij is constructed and inverted to form a unified weight matrix W_joint.

This formulation generalizes to **multiple dependent variables and layers** with full covariance matrices, accounting for correlations among all measured variables within each data point.

### Joint Residual Vector

For each data point `i` in layer `l`, the engine assembles a **joint residual vector**:

$$\tilde{r}_i^{(l)} = \begin{pmatrix} -\delta_i \\ y_i - f(x_i + \delta_i, \beta) \end{pmatrix}$$

The first block penalizes corrections away from the observed x-values, weighted by the independent-variable uncertainty. The second block measures the model misfit. When correlations between x and y exist, the joint weight matrix W_joint couples both blocks, correctly propagating cross-variable correlations into the fitting objective.

### Profiled Jacobian

After solving δ*(β) from the inner stationarity system, the **profiled Jacobian** with respect to β is:

$$\tilde{J}_\beta = J_\beta + J_\delta \cdot \frac{\partial c^*}{\partial \beta}$$

where `∂c*/∂β = -H_cc⁻¹ · H_cβ` is obtained from the inner stationarity Hessian blocks. This gives the correct gradient for the outer optimizer even though the corrections are not explicit optimization variables.

### Second-Order Curvature Correction

The outer Hessian is augmented beyond the Gauss-Newton approximation `JᵀWJ`:

$$H_{\text{total}} = J^T W J + \sum_i \sum_k (W \tilde{r})_k \cdot \frac{\partial^2 \tilde{r}_k}{\partial \beta^2}$$

The second-order term includes:
- **Direct parameter Hessian**: `∂²f/∂β²` weighted by the residual-scaled coefficient.
- **Mixed implicit corrections**: `∂²f/∂x∂β · dc*/dβ` terms from the implicit correction sensitivity.
- **Correction-by-correction terms**: `∂²f/∂x² · dc*/dβ_row · dc*/dβ_col` cross-terms.
- **Finite-difference correction curvature**: `∂f/∂x · d²c*/dβ²` from the numerical implicit-correction tensor.

The sign convention follows the standard ODRPACK/NL2SOL approximation: `outer_second_order_normal -= coeff * implicit_curvature`, which implements `+(W·r̃)_dep · ∂²r_dep/∂β²` because the model residual is `r_dep = y − f`, so `∂²r_dep/∂β² = −∂²f/∂β²`.

**Omitted terms**: The implementation omits the independent-residual-row curvature terms `(W·r̃)_indep · (−∂²c*/∂β²)`. This follows the standard ODRPACK/NL2SOL approximation; these terms are typically negligible when weighted independent residuals are near zero (i.e., corrections are small relative to uncertainties), which is the case for a good fit.

---

## Optimization Strategy

To handle large datasets efficiently, the engine uses a **profiled least-squares** approach, separating the optimization into two nested loops:

### Inner Loop — Batched Newton-Raphson (Latent Corrections)

For a fixed set of parameters β, the engine solves for the optimal δ_i that satisfy the **first-order stationarity condition** `∇_δ χ² = 0`.

The stationarity system for each point assembles:
- A **gradient** from `J_corrections^T · W_joint · r̃` (joint weighted residual projected onto correction space).
- A **Hessian** from `J_corrections^T · W_joint · J_corrections`, plus second-order corrections from the model Hessian `∂²f/∂x²` weighted by the dependent curvature coefficient `(W·r̃)_dep`.

The solver uses damped Newton-Raphson with:
- **Vectorized Evaluation**: Evaluates all active points together in a single vectorized pass, exploiting SIMD and optional parallelism within the evaluation engine.
- **Symmetry Enforcement**: Mixed partial derivatives `∂²f/∂x_i∂x_j` are symmetrized by averaging, ensuring a symmetric Newton system even with minor floating-point discrepancies.
- SVD-based linear system solve (fallback to diagonal regularization at `INNER_CORRECTION_DAMPING × max_diag` on singularity).
- Convergence tolerance `1e-12` on the stationarity gradient norm.
- Maximum 30 iterations per point.
- Point-level convergence tracking: converged points are deactivated; remaining points continue independently.

### Outer Loop — Levenberg-Marquardt (Global Parameters)

The engine optimizes β using a custom **Levenberg-Marquardt** algorithm with:

1. **Adaptive Parameter Scaling (Jacobi Preconditioning)**:
   - Diagonal scale factors `s_i = max(s_i, |H_ii|^0.5)` are accumulated across iterations (monotonically non-decreasing).
   - The normal matrix and gradient are scaled: `H_scaled = S⁻¹ H S⁻¹`, `g_scaled = S⁻¹ g`.
   - This transforms the damped system to `(H_scaled + λI) δ_scaled = −g_scaled`, preventing parameter scale mismatch from distorting the trust region.

2. **Cubic Damping Updates**:
   - After an accepted step (ρ > 0, actual reduction > 0): `λ ← λ · max(1/3, 1 − (2ρ − 1)³)`.
   - After a rejected step: `λ ← λ · ν`, where `ν` starts at 2 and doubles on each consecutive rejection (up to `1e12`).
   - This follows the Nielsen (1999) update strategy for fast convergence near the solution.

3. **Gain Ratio**:
   $$\rho = \frac{\chi^2_{\text{old}} - \chi^2_{\text{new}}}{-2 g^T \delta - \delta^T H \delta}$$
   where `g = Jᵀ W r̃` is the gradient of ½χ² (the code solves `(H + λI) δ = −g`), and `H = JᵀWJ + outer_second_order_normal` is the full augmented normal matrix. The predicted reduction `−2 gᵀ δ − δᵀ H δ` matches the code expression `-(2·g·δ + δ·H·δ)`. When this value is non-positive, the quadratic model is unreliable and ρ is set to −1 (step rejected).

4. **Robust Termination Criteria** (any one triggers):
   - **Scaled Gradient**: `‖g_scaled‖ ≤ ε` (default `1e-9`).
   - **Scaled Step**: `‖δ_scaled‖ ≤ ε · (‖β_scaled‖ + ε)`.
   - **Improvement**: `|Δχ²| ≤ ε` with positive gain ratio.
   - **Stagnation**: 25 consecutive rejected steps.
   - **Damping Saturation**: λ reaches `10^15`.
   - **Singular**: Effective rank of normal matrix drops to zero.
   - **Max Iterations**: Exhausted without convergence.

---

## Architectural Components

### 1. The Solver (`logic/engine/solver.rs`)
The global Levenberg-Marquardt loop. Key properties:
- Operates on the **profiled** chi-squared (corrections eliminated by inner solve).
- Uses SVD-based linear solves for the damped normal equations — never Cholesky — ensuring graceful handling of rank-deficient or ill-conditioned problems.
- Re-evaluates the full model (inner solve + batch evaluation + curvature) at each candidate parameter vector before accepting a step.

### 2. Batched Newton Inner Solve (`logic/engine/inner_solve.rs`)
Eliminates the latent δ corrections. Design:
- **Batched evaluation**: All data points are evaluated together via `evaluate_model_and_gradients_batch`, leveraging the `symb_anafis` SIMD/parallel evaluation engine.
- **Per-point independence**: The stationarity system for each point is solved independently (points are decoupled in the inner problem). Active points continue while converged points are frozen.
- **Second-order model Hessian**: The inner Hessian includes `∂²f/∂x²` terms weighted by `(W·r̃)_dep`, providing superior stability for models with high curvature.
- **Symmetry enforcement**: Mixed partial derivatives `∂²f/∂x_i∂x_j` and `∂²f/∂x_j∂x_i` are symmetrized by averaging, ensuring a symmetric Newton system even with minor floating-point discrepancies.

### 3. Curvature & Hessian Logic (`logic/engine/curvature.rs`)
Implements the second-order corrections to the Gauss-Newton approximation:
- **Implicit first-derivative tensor** `∂c*/∂β`: Computed analytically by solving `H_cc · ∂c*/∂β = −H_cβ` using the inner stationarity Hessian blocks.
- **Implicit second-derivative tensor** `∂²c*/∂β²`: Estimated via **central finite differences** with a 4-point stencil `(++, +−, −+, −−)`. Step sizes are `max(1e-5·|β|, 1e-5)·max(1, 1e-8)` per parameter. Unconverged stencils are zeroed (not included) to avoid noise injection. Complexity is `O(P²)` inner solves per data point, which is expensive for large P but provides accurate curvature for nonlinear models.

### 4. Model Evaluation (`logic/engine/evaluation.rs`)
The central orchestration module that ties everything together:
- Coordinates the inner correction solve for the full dataset.
- Assembles the profiled Jacobian (augmented by `J_δ · ∂c*/∂β`).
- Computes the second-order outer curvature correction per point/layer.
- Accumulates the Welch-Satterthwaite DOF contributions from sensitivity-weighted input variances.
- Separates chi-squared into **profiled** (full) and **observation-only** (dependent-variable residuals only) components.

### 5. Metrological Inference (`logic/engine/inference.rs` & `logic/dof_logic.rs`)
Converts the numerical optimum into GUM-compliant results. See [GUM-Compliant Inference Pipeline](#gum-compliant-inference-pipeline) below.

### 6. Data Preparation (`logic/engine/data_prep.rs`)
Transforms raw user input into the unified numerical space:
- Validates identifier syntax, value finiteness, and length consistency.
- Clamps near-zero uncertainties to `√MIN_VARIANCE = 1e-8`.
- Constructs per-point covariance matrices from uncertainties and correlations.
- Detects **homogeneous** uncertainty patterns (all points identical) and stores a single shared covariance matrix, reducing memory from `O(N·D²)` to `O(D²)`.
- Validates correlation matrices: symmetry, unit diagonal, finite values, range `[-1, 1]`, and positive semi-definiteness (using Sylvester criterion for dim ≤ 3, eigenvalue decomposition for dim > 3).

### 7. Diagnostics (`logic/engine/diagnostics.rs`)
SVD-based analysis of the normal matrix:
- **Effective rank**: Count of singular values exceeding `ε_sing · σ_max` where `ε_sing = 1e-14`.
- **Condition number**: `σ_max / σ_min_nonzero` (among singular values above the threshold).

### 8. Linear Algebra (`logic/engine/linear_algebra.rs`)
Core matrix operations:
- All linear system solves use **SVD** with `ε_sing = 1e-14` — never Cholesky or LU — for maximum numerical safety.
- **PSD matrix square root**: Via eigendecomposition with `λ⁻ = max(0, λ)`, followed by symmetry enforcement `√A = (√A + √Aᵀ)/2`.
- **Small PSD inversion**: SVD pseudo-inverse for joint covariance weight matrices.

---

## GUM-Compliant Inference Pipeline

The inference pipeline is designed to produce results traceable to JCGM 100:2008 (GUM) and JCGM 101:2008 (GUM Supplement 1).

### Step 1: Normal Matrix Construction

The final parameter covariance is derived from the **augmented normal matrix**:

$$H = J^T W J + H_{\text{second-order}}$$

where `H_second-order` includes all curvature corrections described above. The matrix is **symmetrized** after assembly: `H ← (H + Hᵀ)/2`.

### Step 2: Covariance Inversion

The information matrix `H` is inverted via **SVD pseudo-inverse** with threshold `1e-14`:

$$\Sigma_{\text{raw}} = H^{+}$$

This produces the **raw (unscaled) parameter covariance**. Standard uncertainties are `u_raw = √max(0, Σ_raw[i,i])`.

### Step 3: Observation-Only Chi-Squared Scaling

Following the **NIST/ODRPACK convention**, the raw covariance is scaled by the **observation-only reduced chi-squared**:

$$\Sigma_{\text{scaled}} = \frac{\chi^2_{\text{obs}}}{\nu_{\text{res}}} \cdot \Sigma_{\text{raw}}$$

where:
- `χ²_obs` uses **only dependent-variable residuals** (not the full profiled objective).
- `ν_res = N·L − P_eff`, where `P_eff` is the numerical rank of the normal matrix.

**Why observation-only?** The full profiled chi-squared includes the δ-penalty term `δᵀW_xδ`, which would double-count the independent-variable uncertainty contribution when input uncertainties are well-known. The observation-only scaling assumes prior uncertainties are reliable standard deviations and uses the goodness-of-fit only to assess excess scatter.

Both raw and scaled covariance/uncertainty sets are reported, allowing users to apply alternative scaling (e.g., profiled reduced chi-squared) if desired.

### Step 4: Welch-Satterthwaite Effective DOF

The coverage factor requires effective degrees of freedom. The engine computes two DOF sources:

**Input-side DOF** (per-point, sensitivity-weighted):

$$\nu_{\text{input}} = \frac{\left(\sum_i c_i^2 \sigma_i^2\right)^2}{\sum_i \frac{(c_i^2 \sigma_i^2)^2}{\nu_i}}$$

where the summation runs over all measured variables (both independent and dependent) across all data points and layers. `c_i` are the model sensitivities (`∂f/∂x` for independent variables, 1 for dependent variables), `σ_i²` are the input variances, and `ν_i` are per-variable DOF (user-specified or auto-inferred).

**Fit-side DOF**: `ν_fit = N·L − P_eff` (the residual degrees of freedom).

**Two-component combination**:

The scaled parameter variance is decomposed as:
- `u₁² = σ²_raw` (input-propagation component, DOF = `ν_input`)
- `u₂² = max(0, χ²_obs_red − 1) · σ²_raw` (excess residual scatter component, DOF = `ν_fit`)

The Welch-Satterthwaite formula then gives:

$$\nu_{\text{eff}} = \frac{(u_1^2 + u_2^2)^2}{\frac{u_1^4}{\nu_{\text{input}}} + \frac{u_2^4}{\nu_{\text{fit}}}}$$

- When `χ²_obs_red ≤ 1`: input uncertainties fully explain the scatter → `ν_eff = ν_input`.
- When `χ²_obs_red >> 1`: residual DOF dominates → `ν_eff → ν_fit`.

**Fallback**: If neither DOF source is available, the engine falls back to `k ≈ 1.96` (normal approximation), and a warning is emitted.

### Step 5: Student-t Coverage Factor

For the requested confidence level (default 95%):

$$k = t_{\nu_{\text{eff}}, (1+p)/2}$$

where `t` is the Student-t inverse CDF. The engine uses the `statrs` crate for this computation.

### Step 6: Expanded Uncertainty

$$U = k \cdot u_{\text{scaled}}$$

### Step 7: Correlation Matrix

Correlations are computed from the covariance: `ρ_ij = Σ_ij / (u_i · u_j)`. Any off-diagonal correlation exceeding `[-1, 1]` is **clamped** and a warning is emitted, indicating potential ill-conditioning.

### Rank-Deficiency Handling

When `P_eff < P` (rank-deficient normal matrix):
- All parameter uncertainties are reported as **∞**.
- Diagonal covariance entries are set to `∞`; off-diagonal entries are set to `NaN`.
- This avoids reporting false precision for non-identifiable parameters.

---

## Pipeline Flow

The engine processes requests through a deterministic scientific pipeline:

1. **Validation & Normalization** (`sanitization.rs`, `orchestrator.rs`):
   - Identifiers are trimmed, lowercased, and validated for syntax (letters/digits/underscores, must start with a letter).
   - Duplicate names (case-insensitive) are rejected.
   - Independent variable / parameter name collisions are rejected.
   - All initial guess values must be finite.
   - Max iterations clamped to `[5, 5000]`; confidence level clamped to `[0.5, 0.999999]`.

2. **Model JIT Compilation** (`cache.rs`):
   - Formulas are parsed into symbolic expression trees using `symb_anafis`.
   - All required symbolic derivatives are computed: `∂f/∂x`, `∂f/∂β`, `∂²f/∂x²`, `∂²f/∂x∂β`, `∂²f/∂β²`.
   - Independent-variable Hessians are also compiled into `CompiledEvaluator` objects for point-wise inner-solve evaluation.
   - Compiled models are stored in a **global LRU cache** (max 64 entries) keyed by formula + variable names + parameter names, avoiding redundant recompilation.

3. **Data Preparation** (`data_prep.rs`):
   - Covariance matrices are assembled from uncertainties and optional correlation matrices.
   - Near-zero uncertainties are clamped to `σ_min = 1e-8`.
   - Poisson weighting (`σ = √max(y, MIN_VARIANCE)`) is applied for dependent variables without explicit uncertainties when `use_poisson_weighting = true`.
   - Per-point correlation matrices are validated for PSD property.
   - Type A DOF auto-inference: variables with `uncertainty_type = TypeA` but no explicit DOF receive `ν = n − 1`.

4. **Nested Optimization** (`solver.rs` + `inner_solve.rs`):
   - The Levenberg-Marquardt loop (outer) and Batched Newton loop (inner) iteratively minimize the profiled chi-squared.
   - Each outer iteration performs: inner correction solve → batch model evaluation → Jacobian assembly → curvature correction → normal equation construction → LM step → gain ratio evaluation.

5. **Curvature Correction** (`curvature.rs` + `evaluation.rs`):
   - Second-order implicit tensors `∂c*/∂β` and `∂²c*/∂β²` are computed to augment the final parameter covariance.
   - The outer normal matrix is symmetrized after assembly.

6. **Inference & Assembly** (`inference.rs` + `dof_logic.rs` + `response_builder.rs` + `fit_notes.rs`):
   - Results are scaled, Student-t coverage factors computed, and scientific notes/warnings generated for the user.
   - R² is computed as a descriptive statistic (unweighted, using observed dependent values).
   - RMSE and residual standard error are reported separately (RMSE divides by total residuals; RSE divides by residual DOF).

---

## Module Guide

| Module | Responsibility |
| :--- | :--- |
| `logic/orchestrator.rs` | Entry point for fitting requests; manages validation, compilation, and solver invocation. |
| `logic/response_builder.rs` | Final assembly of numerical results into a structured GUM response; computes DOF, coverage factors, and R². |
| `logic/engine/mod.rs` | Re-exports for all engine sub-modules and high-level solver traits. |
| `logic/engine/solver.rs` | Global Levenberg-Marquardt optimization loop with cubic damping and Jacobi preconditioning. |
| `logic/engine/inner_solve.rs` | Batched Newton solve for per-point independent-variable corrections; includes second-order Hessian terms. |
| `logic/engine/evaluation.rs` | Multi-layer model evaluation orchestrator; assembles profiled Jacobian, curvature corrections, and W-S DOF. |
| `logic/engine/curvature.rs` | Finite-difference implicit correction tensor `∂²c*/∂β²`; joint covariance block extraction with PSD regularization; dependent curvature coefficient computation. |
| `logic/engine/inference.rs` | SVD-based covariance inversion, chi-squared scaling, Student-t coverage factor computation, and correlation clamping. |
| `logic/engine/batch_eval.rs` | High-performance batch evaluation of symbolic expressions using `symb_anafis::eval_f64`; supports model values, gradients, and Hessians. |
| `logic/engine/diagnostics.rs` | SVD-based effective rank and condition number estimation for the normal matrix. |
| `logic/engine/linear_algebra.rs` | Core matrix operations: SVD solve, PSD matrix square root, small-matrix pseudo-inverse. |
| `logic/engine/data_prep.rs` | Input validation, uncertainty clamping, covariance matrix construction, PSD checking (with fast paths for dim ≤ 3). |
| `logic/engine/state.rs` | Core data structures: `EvaluationState`, `PreparedData`, `PointCovariances`, `MatrixDiagnostics`, `OdrTerminationReason`. |
| `logic/dof_logic.rs` | Two-component Welch-Satterthwaite DOF combination for GUM coverage factor selection. |
| `logic/fit_notes.rs` | Generation of scientific diagnostics, assumption disclosures, and quality-of-fit warnings. |
| `logic/fit_metrics.rs` | Calculation of R² (global and per-layer), RMSE, and residual standard error. |
| `logic/sanitization.rs` | Identifier validation and normalization; symbol-set disjointness checks. |
| `logic/cache.rs` | Global LRU model cache with double-checked locking for thread-safe compilation deduplication. |
| `logic/constants.rs` | All numerical constants used by the engine (tolerances, limits, thresholds). |
| `commands.rs` | Tauri command handlers for `fit_custom_odr`, `evaluate_model_curve`, and `evaluate_model_grid`. |
| `types.rs` | Request/response types, `UncertaintyType` enum, and `OdrError` error taxonomy. |
| `tests.rs` | Integration tests covering linear, nonlinear, multilayer, correlated, Poisson-weighted, rank-deficient, and edge-case scenarios. |

---

## Numerical Stability & Safeguards

The engine incorporates multiple safeguards to ensure reliability across diverse scientific datasets:

### Numerical Constants and Limits

| Constant | Value | Purpose |
| :--- | :--- | :--- |
| `MIN_VARIANCE` | `1e-16` | Minimum variance enforced to prevent division-by-zero in weight matrices. |
| `MATRIX_SINGULAR_EPS` | `1e-14` | SVD pseudo-inverse threshold relative to `σ_max`. |
| `CORRELATION_TOLERANCE` | `1e-9` | Symmetry and unit-diagonal tolerance for correlation matrix validation. |
| `DEFAULT_TOLERANCE` | `1e-9` | Outer LM convergence tolerance. |
| `DEFAULT_DAMPING` | `1e-3` | Initial Levenberg-Marquardt damping λ. |
| `MAX_DAMPING` | `1e15` | Upper bound on λ; reaching this triggers `DampingSaturated`. |
| `MIN_DAMPING` | `1e-15` | Lower bound on λ. |
| `INNER_CORRECTION_TOLERANCE` | `1e-12` | Convergence tolerance for per-point stationarity gradient norm. |
| `INNER_CORRECTION_MAX_ITERS` | `30` | Maximum Newton iterations per inner correction solve. |
| `INNER_CORRECTION_DAMPING` | `1e-6` | Fallback diagonal regularization factor (× max diagonal) when inner Hessian is singular. |
| `CORRECTION_VARIANCE_THRESHOLD` | `2e-16` | Variables with diagonal variance ≤ this are treated as fixed (no latent correction attempted). Set to `2 × MIN_VARIANCE` to absorb `√(1e-16)² ≠ 1e-16` round-trip error. |
| `PSD_EIGEN_TOLERANCE` | `1e-10` | Eigenvalue tolerance for positive semi-definiteness checks. |
| `MODEL_CACHE_MAX_ENTRIES` | `64` | Maximum compiled models in the LRU cache. |

### Safeguard Mechanisms

- **SVD Everywhere**: All linear system solves and matrix inversions use SVD-based methods — never Cholesky or LU decomposition. This gracefully handles rank-deficient and ill-conditioned problems without catastrophic failure.
- **Variance Clamping**: All covariance diagonal entries are clamped to `max(v, MIN_VARIANCE)` before use in weight computation.
- **PSD Regularization**: If a joint covariance block (independent + dependent variables) is not PSD, diagonal jitter is added starting at `MIN_VARIANCE` and increasing by 10× per iteration (up to 8 attempts). If still not PSD, a `Numerical` error is raised.
- **Inner Solve Fallback**: When the Newton Hessian is singular, diagonal damping `INNER_CORRECTION_DAMPING × max_diag` is applied, ensuring the correction step always makes progress.
- **Correlation Clamping**: Off-diagonal correlations exceeding `[-1, 1]` are clamped with a warning, preventing impossible correlation values from propagating.
- **Symmetry Enforcement**: The outer normal matrix is symmetrized after curvature assembly: `H ← (H + Hᵀ)/2`. Independent-variable Hessians are symmetrized by averaging mixed partials.
- **Non-Finite Parameter Rejection**: Trial parameter vectors containing any non-finite value are immediately rejected (step rejected, damping increased).
- **Hessian Regularization in Sensitivity Solve**: When computing `∂c*/∂β` via `H_cc⁻¹ · H_cβ`, singularity triggers diagonal regularization with `INNER_CORRECTION_DAMPING × max_diag`.

---

## Performance Optimizations

### Memory Efficiency
- **Shared Covariance Storage**: When all data points have identical uncertainties and no per-point correlations are provided, a single covariance matrix is stored (`PointCovariances::Shared`), reducing memory from `O(N·D²)` to `O(D²)`.
- **Zero-Copy Data Prep**: `PreparedData` maintains cache-friendly columnar layouts of variable layers, with direct slicing into batch evaluation.
- **Pre-Allocation**: All major vectors and matrices are pre-allocated with `Vec::with_capacity` based on known dimensions.

### Computation Efficiency
- **Batched Symbolic Evaluation**: All model values, gradients, and Hessians are evaluated via `symb_anafis::eval_f64` in a single batched call per layer, enabling SIMD vectorization and internal parallelism within the evaluation engine.
- **Lock-Step Inner Solve**: Data points are processed in blocks during the inner Newton loop; model gradients for the entire dataset are evaluated in a single vectorized pass per inner iteration.
- **Active-Point Tracking**: Converged points are deactivated in the inner loop, skipping unnecessary evaluation for already-converged corrections.
- **Expression JIT Compilation**: Symbolic models are compiled into high-performance evaluation trees using `symb_anafis` before the first iteration. The compiled `Expr` objects support fast repeated evaluation without re-parsing.
- **Model Cache**: A global LRU cache of 64 compiled models avoids redundant symbolic parsing and derivative computation. Double-checked locking prevents duplicate compilation under concurrent access.

### Algorithmic Efficiency
- **Profiled Strategy**: The outer optimizer operates on `P` parameters instead of `P + N·C`, where `N·C` (total latent corrections) can be thousands. This dramatically reduces the outer problem dimension.
- **Homogeneous Uncertainty Fast Path**: Detection of constant-per-variable uncertainties (`all_homogeneous`) avoids constructing `N` identical covariance matrices.
- **Fast PSD Check for Small Matrices**: Custom Sylvester-criterion implementations for dim ≤ 3 (using principal minor checks) avoid the cost of eigenvalue decomposition. Full eigenvalue decomposition is used only for dim > 3.

---

## Diagnostic Monitoring

The engine performs continuous scientific auditing of the optimization process, reporting warnings via the `message` field:

### Quality-of-Fit Assessments
| Condition | Warning |
| :--- | :--- |
| `χ²_obs_red > 5.0` | **Poor fit** — model does not explain data within given uncertainties. Check for model mismatch or underestimated variances. |
| `χ²_obs_red < 0.1` and `ν > 5` | **Suspiciously low** — uncertainties may be overestimated or model is over-saturated. |
| `P_eff < P` | **Rank-deficient** — effective rank vs. parameter count; parameter uncertainties reported as infinite. |
| `cond(H) > 1e12` | **Ill-conditioned** — parameter uncertainties may be numerically unstable. |

### Convergence Diagnostics
| Condition | Warning |
| :--- | :--- |
| Inner stationarity max norm `> 1e-3` | **Weak** latent correction convergence; profiled linearization may be inaccurate. |
| Inner stationarity max norm `> 1e-6` | **Moderate** convergence; verify on strongly nonlinear datasets. |
| Non-converged inner correction points `> 0` | Some per-point inner corrections did not converge. |
| FD tensor unconverged perturbations `> 0` | Some `∂²c*/∂β²` finite-difference stencils failed; those entries were zeroed. |
| Max iterations reached | Best available estimate reported. |
| Damping saturated | Solution may be weakly constrained. |

### Data Quality Warnings
| Condition | Warning |
| :--- | :--- |
| Uncertainty clamping occurred | Zero/near-zero uncertainties were clamped to minimum. |
| Poisson low counts (`< 20`) | Plug-in `σ = √n` may underestimate uncertainty in low-count regime. |
| Poisson zero counts | Variance clamped to `MIN_VARIANCE`; plug-in sigma unreliable. |
| Type A DOF auto-inferred | DOF was set to `n − 1` for variables without explicit DOF. |
| PSD regularization applied | Joint covariance blocks required diagonal jitter; weighting was stabilized. |
| Corrections suppressed | Variables with variance below correction threshold are treated as fixed. |
| Correlation clamped | At least one correlation exceeded `[-1, 1]` before clamping. |
| Normal coverage fallback | `k ≈ 1.96` used because effective DOF were unavailable. |
| Shared measured variable dependencies | A dependent variable is reused as independent across layers; coupling may affect Gauss-Newton approximation validity. |

### Assumptions Disclosed
All results are accompanied by an **assumptions** list that discloses:
1. ODR accounts for uncertainties in both independent and dependent variables.
2. Two uncertainty sets are reported: Raw (unscaled inverse matrix) and Scaled (× observation-only reduced chi-squared). The scaling follows ODRPACK convention.
3. Profiled DOF uses `N × L − P_eff` for both profiled and observation-based DOF.
4. The outer curvature model augments Gauss-Newton with second-order terms including implicit-correction coupling.
5. The outer Hessian omits independent-correction-row curvature terms (ODRPACK/NL2SOL approximation).
6. Numerical stability safeguards: minimum variance clamping, PSD regularization, bounded correlation.
7. Covariance and confidence intervals assume approximate linearity near the optimum.
8. The effective DOF uses two-component Welch-Satterthwaite decomposition (GUM F.1.1.3 principle).
9. R² is a descriptive statistic only — not a rigorous goodness-of-fit measure when predictors have uncertainty.

For **multi-layer** models, two additional assumptions are disclosed:
10. Per-layer R² should be preferred over global R² (different units/scales across layers).
11. Shared measured variables are coupled through inner corrections; outer curvature ignores coupling-map implicit curvature.

---

## Known Limitations & Design Trade-offs

### Omitted Hessian Terms
The outer Hessian includes only the **dependent-residual curvature** contribution `(W·r̃)_dep · ∂²f/∂β²`. The **independent-correction-row curvature** terms `(W·r̃)_indep · (−∂²c*/∂β²)` are omitted, following the ODRPACK/NL2SOL standard approximation. These terms are typically negligible when:
- The fit is good (weighted independent residuals are near zero).
- Independent-variable uncertainties are moderate relative to the model curvature.
- The model is not severely nonlinear.

If independent uncertainties dominate and the model is severely nonlinear, the omitted terms could modestly affect parameter covariance. The reported **inner stationarity norms** help assess this risk: large stationarity norms indicate the corrections have not fully converged, suggesting the linearization may be inaccurate.

### Finite-Difference Correction Tensor Cost
The `∂²c*/∂β²` tensor requires `O(P²)` inner correction solves per data point (each involving 4 perturbed parameter evaluations). For large P, this is the dominant cost. Future work could explore automatic differentiation through the inner solve or adjoint-based approaches.

### Welch-Satterthwaite Assumptions
The W-S DOF combination assumes independent variance components. Correlations among input quantities are accounted for in the weighting matrix but **not** in the DOF combination formula. This is a standard GUM limitation acknowledged in JCGM 100:2008 G.4.1.

### Poisson Weighting Limitations
For low counts (< 20), the plug-in estimator `σ = √n` underestimates uncertainty because it ignores the variance of the variance estimate. For zero counts, variance is clamped to `MIN_VARIANCE`, which is a conservative lower bound but not a statistically motivated one. Users working with count data in the low-count regime should provide explicit uncertainties (e.g., from a likelihood-based treatment) rather than relying on Poisson weighting.

### Linearity Assumption for Confidence Intervals
The GUM framework assumes the model is approximately linear near the optimum. For strongly nonlinear models, the reported coverage intervals may not achieve the nominal coverage probability. In such cases, Monte Carlo methods (GUM Supplement 1 / JCGM 101:2008) or Markov-chain approaches should be used for uncertainty evaluation.

### Multi-Layer Shared Variables
When a dependent variable from one layer serves as an independent variable in another, the latent corrections are jointly coupled across layers in the inner solve, but the outer optimizer uses a Gauss-Newton approximation that ignores the implicit curvature of this coupling map. This is generally acceptable when shared-variable corrections are small.

---

## Computational Infrastructure

The engine is built on top of high-performance Rust libraries:

- **nalgebra**: All matrix operations use `DMatrix` and `DVector` types. For maximum numerical safety, the engine defaults to **Singular Value Decomposition (SVD)** for solving linear systems and computing pseudo-inverses. This allows the solver to handle rank-deficient and ill-conditioned problems gracefully without catastrophic failure. Eigendecomposition is used for PSD matrix square roots and PSD checking.
- **statrs**: Used for Student-t distribution inverse CDF calculations during confidence interval estimation. The fallback coverage factor is `k = 1.959963984540054` (the normal 97.5% quantile).
- **symb_anafis**: Provides the JIT expression evaluation for model functions and all their symbolic derivatives (first and second order). The `eval_f64` batch evaluation function supports SIMD vectorization and internal parallelism.
