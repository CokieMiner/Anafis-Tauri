//! Statistical distribution analysis module

pub mod types;
pub mod moments;
pub mod fitting;
pub mod goodness_of_fit;

pub use types::{DistributionFit, WeibullCost};
pub use moments::{moments, variance, rank_transformation};
pub use fitting::StatisticalDistributionEngine;
pub use goodness_of_fit::{goodness_of_fit, gamma, beta, digamma, trigamma, gamma_cdf, beta_cdf, student_t_cdf, cauchy_cdf, pareto_cdf, gumbel_cdf};