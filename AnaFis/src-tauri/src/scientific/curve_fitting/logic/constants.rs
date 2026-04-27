/// Minimum variance allowed for a data point to avoid division by zero and extreme conditioning.
pub const MIN_VARIANCE: f64 = 1e-16;
/// Threshold for considering a matrix singular during inversion.
pub const MATRIX_SINGULAR_EPS: f64 = 1e-14;
/// Tolerance for identical independent values to be considered correlated.
pub const CORRELATION_TOLERANCE: f64 = 1e-9;
/// Default maximum number of iterations for ODR convergence.
pub const DEFAULT_MAX_ITERATIONS: usize = 200;
/// Default convergence tolerance for ODR.
pub const DEFAULT_TOLERANCE: f64 = 1e-9;
/// Default initial damping factor (lambda) for Levenberg-Marquardt.
pub const DEFAULT_DAMPING: f64 = 1e-3;
/// Maximum allowed damping factor before assuming divergence.
pub const MAX_DAMPING: f64 = 1e15;
/// Minimum allowed damping factor.
pub const MIN_DAMPING: f64 = 1e-15;
/// Maximum number of compiled models to keep in the cache.
pub const MODEL_CACHE_MAX_ENTRIES: usize = 64;
/// Tolerance for eigenvalue checks to ensure Positive Semi-Definiteness.
pub const PSD_EIGEN_TOLERANCE: f64 = 1e-10;
/// Maximum iterations for per-point independent-variable correction.
pub const INNER_CORRECTION_MAX_ITERS: usize = 30;
/// Convergence tolerance for per-point correction step norm.
pub const INNER_CORRECTION_TOLERANCE: f64 = 1e-12;
/// Damping used in the per-point inner correction solve.
pub const INNER_CORRECTION_DAMPING: f64 = 1e-6;
/// Variance threshold for deciding whether a variable has real (user-provided)
/// uncertainty vs. clamped-to-minimum uncertainty. Set to 2 × `MIN_VARIANCE` to
/// absorb the floating-point round-trip error `sqrt(MIN_VARIANCE)² ≠ MIN_VARIANCE`
/// (about 1 ULP at 1e-16). Variables with covariance diagonal ≤ this threshold
/// are treated as having no measurable uncertainty and are excluded from latent
/// variable corrections.
pub const CORRECTION_VARIANCE_THRESHOLD: f64 = MIN_VARIANCE * 2.0;
