//! Root finding algorithms
//!
//! This module provides numerical methods for finding roots
//! of nonlinear equations, including Brent's method and bisection.

use argmin::core::{CostFunction, Executor};
use argmin::solver::brent::BrentRoot;

/// Root finding algorithms
pub struct RootFinding;

impl RootFinding {
    /// Brent's method for root finding
    pub fn brent_root<F>(f: F, a: f64, b: f64, tol: f64) -> Result<f64, String>
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        if f(a) * f(b) >= 0.0 {
            return Err("Function must have opposite signs at endpoints".to_string());
        }

        let solver = BrentRoot::new(a, b, tol);
        let cost = RootFindingCost { f };

        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100))
            .run()
            .map_err(|e| format!("Root finding failed: {:?}", e))?;

        Ok(*res.state.best_param.as_ref().expect("best_param should be set after successful run"))
    }

    /// Bisection method (more robust but slower)
    pub fn bisection_root<F>(f: F, mut a: f64, mut b: f64, tol: f64, max_iter: usize) -> Result<f64, String>
    where
        F: Fn(f64) -> f64,
    {
        let mut fa = f(a);
        let fb = f(b);

        if fa * fb >= 0.0 {
            return Err("Function must have opposite signs at endpoints".to_string());
        }

        for _ in 0..max_iter {
            let c = (a + b) / 2.0;
            let fc = f(c);

            if fc.abs() < tol {
                return Ok(c);
            }

            if fa * fc < 0.0 {
                b = c;
            } else {
                a = c;
                fa = fc;
            }

            if (b - a).abs() < tol {
                return Ok((a + b) / 2.0);
            }
        }

        Err("Maximum iterations reached".to_string())
    }
}

/// Cost function wrapper for argmin optimization
struct RootFindingCost<F> {
    f: F,
}

impl<F> CostFunction for RootFindingCost<F>
where
    F: Fn(f64) -> f64 + Send + Sync + 'static,
{
    type Param = f64;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, argmin::core::Error> {
        Ok((self.f)(*param))
    }
}