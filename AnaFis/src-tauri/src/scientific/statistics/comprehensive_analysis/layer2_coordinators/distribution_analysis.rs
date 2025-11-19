use crate::scientific::statistics::types::NormalityTestResult;
use argmin::core::{CostFunction, Executor};
use argmin::solver::brent::BrentRoot;
use crate::scientific::statistics::types::DistributionFit;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::distribution::fitters::distribution_fitting_core::StatisticalDistributionEngine as CoreEngine;
use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::correlation::hypothesis_testing::CorrelationHypothesisTestingEngine;


#[derive(Debug, Clone)]
pub struct DistributionAnalysis {
    pub normality_tests: Vec<NormalityTestResult>,
    pub distribution_fits: Vec<DistributionFit>,
    pub best_fit_distribution: Option<DistributionFit>,
    pub transformation_suggestions: Vec<TransformationSuggestion>,
}

#[derive(Debug, Clone)]
pub struct TransformationSuggestion {
    pub transformation: String,
    pub improvement_score: f64,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct BoxCoxResult {
    pub transformed: Vec<f64>,
    pub lambda: f64,
    pub shift: f64,
}

/// Distribution Analysis Coordinator
/// Coordinates normality testing and distribution fitting
pub struct DistributionAnalysisCoordinator;

/// Cost function for Yeo-Johnson lambda optimization
struct YeoJohnsonCostFunction<'a> {
    data: &'a [f64],
}

impl<'a> CostFunction for YeoJohnsonCostFunction<'a> {
    type Param = f64;
    type Output = f64;

    fn cost(&self, lambda: &Self::Param) -> Result<Self::Output, argmin::core::Error> {
        let transformed = DistributionAnalysisCoordinator::apply_yeo_johnson(self.data, *lambda);
        match Self::normal_log_likelihood_static(&transformed) {
            Ok(ll) => Ok(-ll), // Minimize negative log-likelihood
            Err(_) => Ok(f64::INFINITY),
        }
    }
}

impl YeoJohnsonCostFunction<'_> {
    fn normal_log_likelihood_static(data: &[f64]) -> Result<f64, String> {
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (n - 1.0);
        
        if variance <= 0.0 {
            return Ok(f64::NEG_INFINITY);
        }
        
        let log_likelihood = -0.5 * n * (2.0 * std::f64::consts::PI * variance).ln() 
                           - data.iter().map(|x| (x - mean).powi(2) / (2.0 * variance)).sum::<f64>();
        
        Ok(log_likelihood)
    }
}

impl DistributionAnalysisCoordinator {
    pub fn analyze(data: &[f64]) -> Result<DistributionAnalysis, String> {
        if data.is_empty() {
            return Err("Cannot analyze distribution of empty dataset".to_string());
        }

        // Normality tests (only if sufficient data)
        let normality_tests = if data.len() >= 3 {
            CorrelationHypothesisTestingEngine::normality_tests(data)?
        } else {
            Vec::new() // Empty vector for small datasets
        };

        // Distribution fitting
        let distribution_fits = CoreEngine::fit_distributions(data)?;

        // Determine best fit
        let best_fit = distribution_fits.first().cloned();

        // Transformation suggestions
        let transformation_suggestions = Self::suggest_transformations(data, &normality_tests)?;

        Ok(DistributionAnalysis {
            normality_tests,
            distribution_fits,
            best_fit_distribution: best_fit,
            transformation_suggestions,
        })
    }

    /// Suggest data transformations to improve normality
    fn suggest_transformations(
        data: &[f64],
        normality_tests: &[NormalityTestResult],
    ) -> Result<Vec<TransformationSuggestion>, String> {
        let mut suggestions = Vec::new();

        // Skip transformation suggestions if data is too small for normality testing
        if data.len() < 3 {
            return Ok(suggestions);
        }

        // Get current normality (use Shapiro-Wilk if available, otherwise average)
        let current_normality = normality_tests.iter()
            .find(|test| test.test_name == "Shapiro-Wilk")
            .map(|test| test.p_value)
            .unwrap_or_else(|| {
                let valid_p_values: Vec<f64> = normality_tests.iter()
                    .map(|test| test.p_value)
                    .collect();
                if valid_p_values.is_empty() {
                    0.0
                } else {
                    valid_p_values.iter().sum::<f64>() / valid_p_values.len() as f64
                }
            });

        // Try log transformation
        if data.iter().all(|&x| x > 0.0) {
            let log_data: Vec<f64> = data.iter().map(|x| x.ln()).collect();
            let log_normality = CorrelationHypothesisTestingEngine::normality_tests(&log_data)?
                .first().map(|test| test.p_value).unwrap_or(0.0);

            if log_normality > current_normality {
                suggestions.push(TransformationSuggestion {
                    transformation: "log".to_string(),
                    improvement_score: log_normality - current_normality,
                    rationale: "Log transformation often helps with right-skewed data".to_string(),
                });
            }
        }

        // Try square root transformation
        if data.iter().all(|&x| x >= 0.0) {
            let sqrt_data: Vec<f64> = data.iter().map(|x| x.sqrt()).collect();
            let sqrt_normality = CorrelationHypothesisTestingEngine::normality_tests(&sqrt_data)?
                .first().map(|test| test.p_value).unwrap_or(0.0);

            if sqrt_normality > current_normality {
                suggestions.push(TransformationSuggestion {
                    transformation: "sqrt".to_string(),
                    improvement_score: sqrt_normality - current_normality,
                    rationale: "Square root transformation reduces right-skewness".to_string(),
                });
            }
        }

        // Try Yeo-Johnson transformation (can handle negative values)
        {
            if let Ok(yeojohnson_result) = Self::yeo_johnson_transform_optimized(data) {
                let yeojohnson_normality = CorrelationHypothesisTestingEngine::normality_tests(&yeojohnson_result.transformed)?
                    .first().map(|test| test.p_value).unwrap_or(0.0);

                if yeojohnson_normality > current_normality {
                    suggestions.push(TransformationSuggestion {
                        transformation: format!("yeojohnson(lambda={:.3})", yeojohnson_result.lambda),
                        improvement_score: yeojohnson_normality - current_normality,
                        rationale: format!("Yeo-Johnson transformation with λ={:.3} improves normality and handles negative values", yeojohnson_result.lambda),
                    });
                }
            }
        }

        // Sort by improvement score
        suggestions.sort_by(|a, b| b.improvement_score.total_cmp(&a.improvement_score));

        Ok(suggestions)
    }

    /// Optimized Yeo-Johnson transformation with parameter estimation
    fn yeo_johnson_transform_optimized(data: &[f64]) -> Result<BoxCoxResult, String> {
        if data.is_empty() {
            return Err("Cannot perform Yeo-Johnson transformation on empty data".to_string());
        }
        
        // Yeo-Johnson doesn't need shifting like Box-Cox
        let shift = 0.0;
        
        // Optimize lambda using maximum likelihood
        let optimal_lambda = Self::optimize_yeo_johnson_lambda(data)?;
        
        // Apply transformation
        let transformed = Self::apply_yeo_johnson(data, optimal_lambda);
        
        Ok(BoxCoxResult {
            transformed,
            lambda: optimal_lambda,
            shift,
        })
    }

    /// Optimize Yeo-Johnson lambda parameter using maximum likelihood
    fn optimize_yeo_johnson_lambda(data: &[f64]) -> Result<f64, String> {
        // Use Brent's method for optimization
        let cost = YeoJohnsonCostFunction { data };
        
        // Set up Brent optimizer with bounds -2 to 2 (typical range for Yeo-Johnson)
        let solver = BrentRoot::new(-2.0, 2.0, 1e-8);
        
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100))
            .run()
            .map_err(|e| format!("Yeo-Johnson optimization failed: {:?}", e))?;
        
        Ok(*res.state().best_param.as_ref().expect("best_param should be set after successful run"))
    }

    /// Apply Yeo-Johnson transformation with given lambda
    fn apply_yeo_johnson(data: &[f64], lambda: f64) -> Vec<f64> {
        data.iter().map(|&x| {
            if x >= 0.0 {
                // For positive values: ((x + 1)^λ - 1) / λ
                if lambda.abs() < 1e-10 {
                    (x + 1.0).ln()
                } else {
                    ((x + 1.0).powf(lambda) - 1.0) / lambda
                }
            } else {
                // For negative values: -((-x + 1)^(2-λ) - 1) / (2-λ)
                if (lambda - 2.0).abs() < 1e-10 {
                    (-x + 1.0).ln()
                } else {
                    -((-x + 1.0).powf(2.0 - lambda) - 1.0) / (2.0 - lambda)
                }
            }
        }).collect()
    }

    /// Inverse Yeo-Johnson transformation
    pub fn inverse_yeo_johnson(transformed: &[f64], lambda: f64) -> Vec<f64> {
        transformed.iter().map(|&y| {
            if lambda.abs() < 1e-10 {
                // Inverse of ln(x + 1): exp(y) - 1
                y.exp() - 1.0
            } else if y >= 0.0 {
                // Inverse of ((x + 1)^λ - 1) / λ: (λ*y + 1)^(1/λ) - 1
                (lambda * y + 1.0).powf(1.0 / lambda) - 1.0
            } else {
                // Inverse of -((-x + 1)^(2-λ) - 1) / (2-λ): -( (1 - (2-λ)*y)^(1/(2-λ)) - 1 )
                if (lambda - 2.0).abs() < 1e-10 {
                    // Special case: -(exp(y) - 1)
                    -(y.exp() - 1.0)
                } else {
                    -( (1.0 - (2.0 - lambda) * y).powf(1.0 / (2.0 - lambda)) - 1.0 )
                }
            }
        }).collect()
    }
}