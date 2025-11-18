//! Statistical distribution functions
//!
//! This module provides probability density functions (PDF),
//! cumulative distribution functions (CDF), and quantile functions
//! for various statistical distributions.

use statrs::distribution::{Continuous, ContinuousCDF, Normal, StudentsT, ChiSquared, FisherSnedecor};

/// Statistical distribution implementations
pub struct StatisticalDistributions;

impl StatisticalDistributions {
    /// Normal distribution PDF
    pub fn normal_pdf(x: f64, mean: f64, std: f64) -> f64 {
        let normal = Normal::new(mean, std).unwrap();
        normal.pdf(x)
    }

    /// Normal distribution CDF
    pub fn normal_cdf(x: f64, mean: f64, std: f64) -> f64 {
        let normal = Normal::new(mean, std).unwrap();
        normal.cdf(x)
    }

    /// Normal distribution quantile (inverse CDF)
    pub fn normal_quantile(p: f64, mean: f64, std: f64) -> f64 {
        let normal = Normal::new(mean, std).unwrap();
        normal.inverse_cdf(p)
    }

    /// Student's t-distribution PDF
    pub fn t_pdf(x: f64, df: f64) -> f64 {
        let t = StudentsT::new(0.0, 1.0, df).unwrap();
        t.pdf(x)
    }

    /// Student's t-distribution CDF
    pub fn t_cdf(x: f64, df: f64) -> f64 {
        let t = StudentsT::new(0.0, 1.0, df).unwrap();
        t.cdf(x)
    }

    /// Student's t-distribution quantile
    pub fn t_quantile(p: f64, df: f64) -> f64 {
        let t = StudentsT::new(0.0, 1.0, df).unwrap();
        t.inverse_cdf(p)
    }

    /// Chi-squared distribution PDF
    pub fn chi_squared_pdf(x: f64, df: f64) -> f64 {
        let chi2 = ChiSquared::new(df).unwrap();
        chi2.pdf(x)
    }

    /// Chi-squared distribution CDF
    pub fn chi_squared_cdf(x: f64, df: f64) -> f64 {
        let chi2 = ChiSquared::new(df).unwrap();
        chi2.cdf(x)
    }

    /// Chi-squared distribution quantile
    pub fn chi_squared_quantile(p: f64, df: f64) -> f64 {
        let chi2 = ChiSquared::new(df).unwrap();
        chi2.inverse_cdf(p)
    }

    /// F-distribution CDF
    pub fn f_cdf(x: f64, df1: f64, df2: f64) -> f64 {
        let f_dist = FisherSnedecor::new(df1, df2).unwrap();
        f_dist.cdf(x)
    }

    /// F-distribution quantile
    pub fn f_quantile(p: f64, df1: f64, df2: f64) -> f64 {
        let f_dist = FisherSnedecor::new(df1, df2).unwrap();
        f_dist.inverse_cdf(p)
    }
}