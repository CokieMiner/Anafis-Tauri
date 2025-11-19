/// Result of fitting a statistical distribution to data
#[derive(Debug, Clone)]
pub struct DistributionFit {
    /// Name of the fitted distribution
    pub distribution_name: String,
    /// Fitted parameters as (name, value) pairs
    pub parameters: Vec<(String, f64)>,
    /// Log-likelihood of the fit
    pub log_likelihood: f64,
    /// Akaike Information Criterion
    pub aic: f64,
    /// Bayesian Information Criterion
    pub bic: f64,
    /// Goodness of fit statistic (Kolmogorov-Smirnov statistic)
    pub goodness_of_fit: f64,
}