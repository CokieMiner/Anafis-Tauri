//! Integration tests for distribution fitting and global optimization
//!
//! These tests validate the distribution fitting algorithms and global optimization
//! functionality for robustness and accuracy.

use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::distribution::global_optimizer::{GlobalOptimizer, GlobalOptimizationConfig};
use argmin::core::{CostFunction, Error};

#[derive(Clone)]
struct TestCostFunction;

impl CostFunction for TestCostFunction {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        // Simple quadratic: (x-1)^2 + (y-2)^2
        let x = param[0];
        let y = param[1];
        Ok((x - 1.0).powi(2) + (y - 2.0).powi(2))
    }
}

#[test]
fn test_global_optimization() {
    let cost_fn = TestCostFunction;
    let initial_guess = vec![0.0, 0.0];
    let bounds = vec![(-100.0, 100.0), (-100.0, 100.0)];

    let config = GlobalOptimizationConfig {
        num_starts: 5,
        max_local_iters: 50,
        ..Default::default()
    };

    let mut optimizer = GlobalOptimizer::new(cost_fn, config);
    let result = optimizer.optimize(&initial_guess, &bounds).unwrap();

    // Check that optimization ran and found some solution
    assert!(result.best_param.len() == 2);
    assert!(result.best_cost.is_finite());
    assert!(result.num_evaluations > 0);
    assert!(result.num_local_opts == 5);
    // Note: For this simple test, we don't check convergence to exact minimum
    // as the focus is on the framework working, not perfect optimization
}

#[test]
fn test_global_optimizer_precision() {
    #[derive(Clone)]
    struct PrecisionTestCost {
        minimum: Vec<f64>,
    }

    impl CostFunction for PrecisionTestCost {
        type Param = Vec<f64>;
        type Output = f64;

        fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
            // Simple quadratic function with known minimum at self.minimum
            let diff: Vec<f64> = param.iter().zip(&self.minimum).map(|(p, m)| p - m).collect();
            let cost: f64 = diff.iter().map(|d| d * d).sum();
            Ok(cost)
        }
    }

    let cost_fn = PrecisionTestCost {
        minimum: vec![1.0, 2.0, 3.0],
    };

    let config = GlobalOptimizationConfig {
        num_starts: 100,
        max_local_iters: 200,
        tolerance: 1e-12,
        basin_hopping: true,
        max_basin_iters: 50,
        ..Default::default()
    };

    let mut optimizer = GlobalOptimizer::new(cost_fn.clone(), config);
    let initial_guess = vec![0.0, 0.0, 0.0];
    let bounds = vec![
        (-10.0, 10.0),
        (-10.0, 10.0),
        (-10.0, 10.0),
    ];
    let result = optimizer.optimize(&initial_guess, &bounds).unwrap();

    // Check that we found the minimum within reasonable precision
    let tolerance = 0.01;
    for (found, expected) in result.best_param.iter().zip(&cost_fn.minimum) {
        assert!((found - expected).abs() < tolerance,
            "Parameter not within tolerance: found {}, expected {}, diff {}",
            found, expected, (found - expected).abs());
    }

    // Check that the cost at the found minimum is very close to 0
    let final_cost = cost_fn.cost(&result.best_param).unwrap();
    assert!(final_cost < 1e-6, "Final cost {} is not close to 0", final_cost);
}