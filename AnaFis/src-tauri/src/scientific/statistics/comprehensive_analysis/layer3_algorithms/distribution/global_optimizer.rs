//! Global optimization strategies for distribution fitting
//!
//! This module implements advanced global optimization techniques to avoid local minima
//! in multimodal likelihood surfaces during distribution fitting.

use argmin::core::{CostFunction, Executor};
use argmin::solver::neldermead::NelderMead;
use rand::prelude::*;
use rand_pcg::Pcg64;
use rayon::prelude::*;
use std::sync::Arc;

/// Global optimization configuration
#[derive(Clone, Debug)]
pub struct GlobalOptimizationConfig {
    /// Number of random starts for multi-start optimization
    pub num_starts: usize,
    /// Maximum iterations per local optimization
    pub max_local_iters: u64,
    /// Tolerance for convergence
    pub tolerance: f64,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Enable basin-hopping
    pub basin_hopping: bool,
    /// Basin-hopping step size
    pub basin_hopping_step: f64,
    /// Maximum basin-hopping iterations
    pub max_basin_iters: usize,
    /// Enable adaptive precision
    pub adaptive_precision: bool,
}

impl Default for GlobalOptimizationConfig {
    fn default() -> Self {
        Self {
            num_starts: 10,
            max_local_iters: 100,
            tolerance: 1e-8,
            seed: None,
            basin_hopping: false,
            basin_hopping_step: 0.1,
            max_basin_iters: 50,
            adaptive_precision: true,
        }
    }
}

/// Optimization result with convergence diagnostics
#[derive(Clone, Debug)]
pub struct OptimizationResult {
    /// Best parameter vector found
    pub best_param: Vec<f64>,
    /// Best cost (negative log-likelihood)
    pub best_cost: f64,
    /// Number of function evaluations
    pub num_evaluations: usize,
    /// Number of local optimizations performed
    pub num_local_opts: usize,
    /// Convergence achieved
    pub converged: bool,
    /// All solutions found (for multi-modal analysis)
    pub all_solutions: Vec<(Vec<f64>, f64)>,
}

/// Global optimizer for distribution fitting
pub struct GlobalOptimizer<C> {
    cost_function: Arc<C>,
    config: GlobalOptimizationConfig,
    rng: Pcg64,
}

impl<C> GlobalOptimizer<C>
where
    C: CostFunction<Param = Vec<f64>, Output = f64> + Clone + Send + Sync + 'static,
{
    /// Create a new global optimizer
    pub fn new(cost_function: C, config: GlobalOptimizationConfig) -> Self {
        let seed = config.seed.unwrap_or(42);
        let rng = Pcg64::seed_from_u64(seed);

        Self {
            cost_function: Arc::new(cost_function),
            config,
            rng,
        }
    }

    /// Perform global optimization
    pub fn optimize(&mut self, initial_guess: &[f64], bounds: &[(f64, f64)]) -> Result<OptimizationResult, String> {
        if bounds.len() != initial_guess.len() {
            return Err("Bounds length must match parameter length".to_string());
        }

        let mut all_solutions = Vec::new();
        let mut best_param = initial_guess.to_vec();
        let mut best_cost = f64::INFINITY;
        let mut total_evaluations = 0;

        // Multi-start local optimization
        let solutions: Vec<_> = (0..self.config.num_starts)
            .into_par_iter()
            .map(|start_idx| {
                let mut local_rng = Pcg64::seed_from_u64(self.config.seed.unwrap_or(42) + start_idx as u64);

                // Generate random starting point within bounds
                let start_param: Vec<f64> = initial_guess.iter().enumerate()
                    .map(|(i, &val)| {
                        let (min, max) = bounds[i];
                        let range = max - min;
                        let random_offset = local_rng.random::<f64>() * range;
                        (val + random_offset - range / 2.0).clamp(min, max)
                    })
                    .collect();

                // Perform local optimization
                self.local_optimize(&start_param, bounds)
            })
            .collect();

        // Collect results
        for (param, cost, evals) in solutions.into_iter().flatten() {
            total_evaluations += evals;
            all_solutions.push((param.clone(), cost));

            if cost < best_cost {
                best_cost = cost;
                best_param = param;
            }
        }

        // Basin-hopping if enabled
        if self.config.basin_hopping && !best_param.is_empty() {
            let basin_result = self.basin_hopping(&best_param, bounds);
            if let Ok((param, cost, evals)) = basin_result {
                total_evaluations += evals;
                all_solutions.push((param.clone(), cost));

                if cost < best_cost {
                    best_cost = cost;
                    best_param = param;
                }
            }
        }

        // Sort solutions by cost
        all_solutions.sort_by(|a, b| match a.1.partial_cmp(&b.1) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        Ok(OptimizationResult {
            best_param,
            best_cost,
            num_evaluations: total_evaluations,
            num_local_opts: self.config.num_starts,
            converged: best_cost.is_finite() && best_cost < f64::INFINITY,
            all_solutions,
        })
    }

    /// Perform local optimization using Nelder-Mead (derivative-free)
    fn local_optimize(&self, start_param: &[f64], _bounds: &[(f64, f64)]) -> Result<(Vec<f64>, f64, usize), String> {
        // Create simplex around starting point
        let eps = 1e-6;
        let mut simplex = vec![start_param.to_vec()];
        for i in 0..start_param.len() {
            let mut point = start_param.to_vec();
            point[i] *= 1.0 + eps;
            simplex.push(point);
        }

        let solver = NelderMead::new(simplex);

        let executor = Executor::new(self.cost_function.as_ref().clone(), solver)
            .configure(|state| {
                state
                    .max_iters(self.config.max_local_iters)
                    .target_cost(self.config.tolerance)
            });

        match executor.run() {
            Ok(result) => {
                if let Some(best_param) = result.state.best_param {
                    let best_cost = result.state.best_cost;
                    let evaluations = 1; // Simplified - argmin doesn't expose this directly
                    Ok((best_param, best_cost, evaluations))
                } else {
                    Err("Nelder-Mead failed to find best parameters".to_string())
                }
            }
            Err(_) => Err("Nelder-Mead optimization failed".to_string()),
        }
    }

    /// Perform basin-hopping optimization
    fn basin_hopping(&mut self, start_param: &[f64], bounds: &[(f64, f64)]) -> Result<(Vec<f64>, f64, usize), String> {
        let mut current_param = start_param.to_vec();
        let mut total_evaluations = 0;

        // Get initial cost
        let mut current_cost = if let Ok(cost) = self.cost_function.cost(&current_param) {
            total_evaluations += 1;
            cost
        } else {
            return Err("Failed to evaluate initial cost".to_string());
        };

        for _ in 0..self.config.max_basin_iters {
            // Generate random displacement
            let mut new_param = current_param.clone();
            for (i, param) in new_param.iter_mut().enumerate() {
                let (min, max) = bounds[i];
                let displacement = self.rng.random_range(-self.config.basin_hopping_step..=self.config.basin_hopping_step);
                *param = (*param + displacement).clamp(min, max);
            }

            // Local optimization from displaced point
            if let Ok((local_param, local_cost, evals)) = self.local_optimize(&new_param, bounds) {
                total_evaluations += evals;

                // Accept if better
                if local_cost < current_cost {
                    current_param = local_param;
                    current_cost = local_cost;
                }
            }
        }

        Ok((current_param, current_cost, total_evaluations))
    }
}

/// Adaptive precision utilities
pub struct AdaptivePrecision;

impl AdaptivePrecision {
    /// Determine optimal precision based on problem characteristics
    pub fn determine_precision(data_size: usize, param_count: usize) -> f64 {
        // Higher precision for larger datasets and more parameters
        let base_precision = 1e-8;
        let size_factor = (data_size as f64).log10().max(1.0);
        let param_factor = (param_count as f64).sqrt();

        base_precision / (size_factor * param_factor)
    }

    /// Check if optimization has converged
    pub fn has_converged(cost_history: &[f64], tolerance: f64) -> bool {
        if cost_history.len() < 2 {
            return false;
        }

        let recent = &cost_history[cost_history.len().saturating_sub(10)..];
        if recent.len() < 2 {
            return false;
        }

        let min_cost = recent.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_cost = recent.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        (max_cost - min_cost).abs() < tolerance
    }
}

/// Convenience function for global optimization of distribution parameters
pub fn optimize_distribution_parameters<C>(
    cost_function: C,
    initial_guess: &[f64],
    bounds: &[(f64, f64)],
    config: Option<GlobalOptimizationConfig>,
) -> Result<OptimizationResult, String>
where
    C: CostFunction<Param = Vec<f64>, Output = f64> + Clone + Send + Sync + 'static,
{
    let config = config.unwrap_or_default();
    let mut optimizer = GlobalOptimizer::new(cost_function, config);
    optimizer.optimize(initial_guess, bounds)
}