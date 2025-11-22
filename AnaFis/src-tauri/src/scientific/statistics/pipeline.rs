//! Automated statistical analysis pipeline
//!
//! This module provides high-level orchestration for comprehensive statistical analysis,
//! automatically applying appropriate tests and methods based on data characteristics.

use serde::{Deserialize, Serialize};
use crate::scientific::statistics::{
    StatisticalMoments, outliers::OutlierDetectionEngine,
    correlation::CorrelationMethods,
    StationarityEngine, TimeSeriesDecompositionEngine,
    distributions::StatisticalDistributionEngine, preprocessing::DataImputationEngine,
    distributions::DataTransformationEngine, MatrixOpsEngine, OutlierDetectionConfig,
};
use crate::scientific::statistics::time_series::{SpectralEngine};
use crate::scientific::statistics::descriptive::{Quantiles, QuantileMethod};
use crate::scientific::statistics::descriptive::dispersion;
use crate::scientific::statistics::stationarity::types::AdfRegressionType;
use crate::scientific::statistics::correlation::CorrelationHypothesisTestingEngine;
use crate::scientific::statistics::correlation::types::NormalityTestResult as CorrelationNormalityTestResult;
use rand_pcg::Pcg64;

/// Comprehensive analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveAnalysis {
    /// Dataset metadata
    pub metadata: DatasetMetadata,
    /// Descriptive statistics
    pub descriptive_stats: DescriptiveStatistics,
    /// Data quality assessment
    pub data_quality: DataQualityAssessment,
    /// Distribution analysis
    pub distribution_analysis: DistributionAnalysis,
    /// Correlation analysis
    pub correlation_analysis: CorrelationAnalysis,
    /// Outlier analysis
    pub outlier_analysis: OutlierAnalysis,
    /// Time series analysis (if applicable)
    pub time_series_analysis: Option<TimeSeriesAnalysis>,
    /// Recommendations for further analysis
    pub recommendations: Vec<String>,
}

/// Dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub n_observations: usize,
    pub n_variables: usize,
    pub data_types: Vec<String>,
    pub missing_value_rate: f64,
    pub analysis_timestamp: String,
}

/// Descriptive statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptiveStatistics {
    pub univariate_stats: Vec<UnivariateStats>,
    pub multivariate_stats: MultivariateStats,
}

/// Univariate statistics for each variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnivariateStats {
    pub variable_name: String,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub quantiles: Vec<f64>,
}

/// Multivariate statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultivariateStats {
    pub correlation_matrix: Vec<Vec<f64>>,
    pub covariance_matrix: Vec<Vec<f64>>,
}

/// Data quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityAssessment {
    pub missing_values: MissingValueAnalysis,
    pub outliers: OutlierSummary,
    pub normality_tests: Vec<NormalityTestResult>,
    pub transformations_applied: Vec<String>,
}

/// Missing value analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingValueAnalysis {
    pub total_missing: usize,
    pub missing_by_variable: Vec<(String, usize)>,
    pub imputation_method: Option<String>,
}

/// Outlier summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierSummary {
    pub total_outliers: usize,
    pub outliers_by_variable: Vec<(String, usize)>,
    pub outlier_methods_used: Vec<String>,
}

/// Normality test result (wrapper around correlation module's result with variable name)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalityTestResult {
    pub variable: String,
    pub test_result: CorrelationNormalityTestResult,
}

/// Distribution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionAnalysis {
    pub fitted_distributions: Vec<DistributionFitSummary>,
    pub best_fits: Vec<String>,
}

/// Distribution fit summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionFitSummary {
    pub variable: String,
    pub distribution: String,
    pub aic: f64,
    pub bic: f64,
    pub parameters: Vec<(String, f64)>,
}

/// Correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    pub significant_correlations: Vec<CorrelationResult>,
    pub correlation_tests: Vec<CorrelationTest>,
}

/// Correlation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    pub var1: String,
    pub var2: String,
    pub correlation: f64,
    pub p_value: f64,
    pub method: String,
}

/// Correlation test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationTest {
    pub variables: (String, String),
    pub test_type: String,
    pub statistic: f64,
    pub p_value: f64,
    pub significant: bool,
}

/// Outlier analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierAnalysis {
    pub detected_outliers: Vec<OutlierDetection>,
    pub robust_statistics: Vec<RobustStats>,
}

/// Outlier detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierDetection {
    pub variable: String,
    pub method: String,
    pub outlier_indices: Vec<usize>,
    pub outlier_values: Vec<f64>,
}

/// Robust statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustStats {
    pub variable: String,
    pub median: f64,
    pub mad: f64, // Median absolute deviation
    pub iqr: f64, // Interquartile range
}

/// Time series analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesAnalysis {
    pub stationarity_tests: Vec<StationarityTest>,
    pub decomposition: Option<TimeSeriesDecomposition>,
    pub spectral_analysis: Option<SpectralAnalysis>,
    pub forecasts: Option<ForecastResults>,
}

/// Stationarity test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationarityTest {
    pub variable: String,
    pub test_name: String,
    pub statistic: f64,
    pub p_value: f64,
    pub is_stationary: bool,
}

/// Time series decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesDecomposition {
    pub trend: Vec<f64>,
    pub seasonal: Vec<f64>,
    pub residual: Vec<f64>,
    pub seasonality_period: Option<usize>,
}

/// Spectral analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralAnalysis {
    pub frequencies: Vec<f64>,
    pub power_spectrum: Vec<f64>,
    pub dominant_frequency: f64,
    pub spectral_entropy: f64,
}

/// Forecast results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResults {
    pub method: String,
    pub forecasts: Vec<f64>,
    pub confidence_intervals: Vec<(f64, f64)>,
    pub accuracy_metrics: ForecastAccuracy,
}

/// Forecast accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastAccuracy {
    pub mae: f64,
    pub rmse: f64,
    pub mape: f64,
}

/// Automated statistical analysis pipeline
pub struct StatisticalAnalysisPipeline;

impl StatisticalAnalysisPipeline {
    /// Perform comprehensive automated statistical analysis
    pub fn comprehensive_analysis(
        data: &[Vec<f64>],
        variable_names: Option<Vec<String>>,
        time_series: bool,
    ) -> Result<ComprehensiveAnalysis, String> {
        if data.is_empty() || data[0].is_empty() {
            return Err("Cannot analyze empty dataset".to_string());
        }

        let n_vars = data[0].len();

        // Generate default variable names if not provided
        let var_names = variable_names.unwrap_or_else(|| {
            (0..n_vars).map(|i| format!("Var{}", i + 1)).collect()
        });

        // 1. Dataset metadata
        let metadata = Self::analyze_metadata(data, &var_names);

        // 2. Data preprocessing (imputation and transformation)
        let (processed_data, preprocessing_info) = Self::preprocess_data(data)?;

        // 3. Descriptive statistics
        let descriptive_stats = Self::compute_descriptive_stats(&processed_data, &var_names)?;

        // 4. Data quality assessment
        let data_quality = Self::assess_data_quality(&processed_data, &var_names, &preprocessing_info)?;

        // 5. Distribution analysis
        let distribution_analysis = Self::analyze_distributions(&processed_data, &var_names)?;

        // 6. Correlation analysis
        let correlation_analysis = Self::analyze_correlations(&processed_data, &var_names)?;

        // 7. Outlier analysis
        let outlier_analysis = Self::analyze_outliers(&processed_data, &var_names)?;

        // 8. Time series analysis (if requested and data looks time-series like)
        let time_series_analysis = if time_series && Self::is_time_series_data(&processed_data) {
            Some(Self::analyze_time_series(&processed_data, &var_names)?)
        } else {
            None
        };

        // 9. Generate recommendations
        let recommendations = Self::generate_recommendations(
            &data_quality,
            &distribution_analysis,
            &correlation_analysis,
            &time_series_analysis,
        );

        Ok(ComprehensiveAnalysis {
            metadata,
            descriptive_stats,
            data_quality,
            distribution_analysis,
            correlation_analysis,
            outlier_analysis,
            time_series_analysis,
            recommendations,
        })
    }

    /// Analyze dataset metadata
    fn analyze_metadata(data: &[Vec<f64>], var_names: &[String]) -> DatasetMetadata {
        let n_obs = data.len();
        let n_vars = data[0].len();

        // Estimate data types (simplified)
        let data_types = var_names.iter().map(|_| "numeric".to_string()).collect();

        // Calculate missing value rate
        let total_cells = n_obs * n_vars;
        let missing_cells = data.iter()
            .flat_map(|row| row.iter())
            .filter(|&&x| x.is_nan())
            .count();
        let missing_value_rate = missing_cells as f64 / total_cells as f64;

        DatasetMetadata {
            n_observations: n_obs,
            n_variables: n_vars,
            data_types,
            missing_value_rate,
            analysis_timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Preprocess data (imputation and transformation)
    fn preprocess_data(data: &[Vec<f64>]) -> Result<(Vec<Vec<f64>>, Vec<String>), String> {
        let mut preprocessing_info = Vec::new();

        // Check for missing values
        let has_missing = data.iter()
            .any(|row| row.iter().any(|&x| x.is_nan()));

        let mut processed_data = if has_missing {
            // Apply imputation
            let impute_result = DataImputationEngine::auto_impute(data, f64::NAN)?;
            preprocessing_info.push(format!("Applied {} imputation", impute_result.method));
            impute_result.imputed_data
        } else {
            data.to_vec()
        };

        // Check if transformations are needed (based on normality)
        let mut needs_transformation = false;
        for col in 0..processed_data[0].len() {
            let column_data: Vec<f64> = processed_data.iter().map(|row| row[col]).collect();
            let skewness = column_data.as_slice().skewness();
            let kurtosis = column_data.as_slice().kurtosis();
            // Check for significant non-normality
            if skewness.abs() > 1.0 || !(2.5..=3.5).contains(&kurtosis) {
                needs_transformation = true;
                break;
            }
        }

        if needs_transformation {
            // Try to find best transformation
            for col in 0..processed_data[0].len() {
                let column_data: Vec<f64> = processed_data.iter().map(|row| row[col]).collect();
                if let Ok(transform_result) = DataTransformationEngine::find_best_transformation(&column_data) {
                    // Apply transformation if it improves normality
                    if transform_result.normality_test_statistic > 0.5 { // Arbitrary threshold
                        for (i, row) in processed_data.iter_mut().enumerate() {
                            row[col] = transform_result.transformed_data[i];
                        }
                        preprocessing_info.push(format!("Applied {} transformation to variable {}",
                                                      transform_result.transformation_name, col + 1));
                    }
                }
            }
        }

        Ok((processed_data, preprocessing_info))
    }

    /// Compute descriptive statistics
    fn compute_descriptive_stats(data: &[Vec<f64>], var_names: &[String]) -> Result<DescriptiveStatistics, String> {
        let mut univariate_stats = Vec::with_capacity(var_names.len());
        let n_rows = data.len();
        let _n_cols = data[0].len();

        // Pre-allocate column buffer to reuse memory
        let mut column_data = Vec::with_capacity(n_rows);

        for (i, var_name) in var_names.iter().enumerate() {
            // Extract column data efficiently
            column_data.clear();
            for row in data {
                column_data.push(row[i]);
            }

            // 1. Calculate moments (no sorting needed)
            let mean = column_data.mean();
            let std_dev = column_data.std_dev();
            let skewness = column_data.skewness();
            let kurtosis = column_data.kurtosis();
            
            // Min/Max (single pass)
            let (min, max) = column_data.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &x| {
                (min.min(x), max.max(x))
            });

            // 2. Sort for quantiles (sorting once)
            // We use a separate buffer for sorted data if we want to keep column_data for other things,
            // but here we don't need column_data anymore in this iteration.
            // However, we need to handle NaNs for sorting.
            column_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            // 3. Calculate quantiles and median from sorted data
            let median = Quantiles::median(&column_data); // O(1) on sorted data
            
            let quantiles = vec![
                Quantiles::quantile(&column_data, 0.25, QuantileMethod::Type8).unwrap_or(f64::NAN),
                Quantiles::quantile(&column_data, 0.5, QuantileMethod::Type8).unwrap_or(f64::NAN),
                Quantiles::quantile(&column_data, 0.75, QuantileMethod::Type8).unwrap_or(f64::NAN),
            ];

            univariate_stats.push(UnivariateStats {
                variable_name: var_name.clone(),
                mean,
                median,
                std_dev,
                min,
                max,
                skewness,
                kurtosis,
                quantiles,
            });
        }

        // Compute correlation and covariance matrices
        let cov_matrix = MatrixOpsEngine::covariance_matrix(data, 1)?;
        let correlation_matrix = MatrixOpsEngine::correlation_matrix_from_covariance(&cov_matrix)?;

        // Convert to Vec<Vec<f64>> for the result
        let correlation_matrix_vec: Vec<Vec<f64>> = correlation_matrix
            .axis_iter(ndarray::Axis(0))
            .map(|row| row.to_vec())
            .collect();
        let covariance_matrix_vec: Vec<Vec<f64>> = cov_matrix
            .axis_iter(ndarray::Axis(0))
            .map(|row| row.to_vec())
            .collect();

        let multivariate_stats = MultivariateStats {
            correlation_matrix: correlation_matrix_vec,
            covariance_matrix: covariance_matrix_vec,
        };

        Ok(DescriptiveStatistics {
            univariate_stats,
            multivariate_stats,
        })
    }

    /// Assess data quality
    fn assess_data_quality(
        data: &[Vec<f64>],
        var_names: &[String],
        preprocessing_info: &[String],
    ) -> Result<DataQualityAssessment, String> {
        // Missing value analysis
        let mut missing_by_variable = Vec::new();
        let mut total_missing = 0;

        for (i, var_name) in var_names.iter().enumerate() {
            let missing_count = data.iter()
                .filter(|row| row[i].is_nan())
                .count();
            missing_by_variable.push((var_name.clone(), missing_count));
            total_missing += missing_count;
        }

        let imputation_method = if preprocessing_info.iter().any(|s| s.contains("imputation")) {
            Some("auto".to_string())
        } else {
            None
        };

        let missing_values = MissingValueAnalysis {
            total_missing,
            missing_by_variable,
            imputation_method,
        };

        // Outlier analysis summary
        let outlier_analysis = Self::analyze_outliers(data, var_names)?;
        let outliers = OutlierSummary {
            total_outliers: outlier_analysis.detected_outliers.iter()
                .map(|od| od.outlier_indices.len())
                .sum(),
            outliers_by_variable: outlier_analysis.detected_outliers.iter()
                .map(|od| (od.variable.clone(), od.outlier_indices.len()))
                .collect(),
            outlier_methods_used: outlier_analysis.detected_outliers.iter()
                .map(|od| od.method.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
        };

        // Normality tests
        let mut normality_tests = Vec::new();
        for (i, var_name) in var_names.iter().enumerate() {
            let column_data: Vec<f64> = data.iter().map(|row| row[i]).collect();

            // Use hypothesis testing engine for normality tests
            let engine = CorrelationHypothesisTestingEngine::new();
            let normality_results = engine.normality_tests(&column_data)?;
            if let Some(sw_test) = normality_results.iter().find(|test| test.test_name == "Shapiro-Wilk") {
                normality_tests.push(NormalityTestResult {
                    variable: var_name.clone(),
                    test_result: sw_test.clone(),
                });
            } else {
                // Fallback if Shapiro-Wilk not available
                normality_tests.push(NormalityTestResult {
                    variable: var_name.clone(),
                    test_result: CorrelationNormalityTestResult {
                        test_name: "Approximate".to_string(),
                        statistic: column_data.skewness().abs() + column_data.kurtosis().abs(),
                        p_value: 0.5, // Neutral
                        is_normal: true,
                        method: "Approximate normality check".to_string(),
                    },
                });
            }
        }

        Ok(DataQualityAssessment {
            missing_values,
            outliers,
            normality_tests,
            transformations_applied: preprocessing_info.to_vec(),
        })
    }

    /// Analyze distributions
    fn analyze_distributions(data: &[Vec<f64>], var_names: &[String]) -> Result<DistributionAnalysis, String> {
        let mut fitted_distributions = Vec::new();
        let mut best_fits = Vec::new();

        for (i, var_name) in var_names.iter().enumerate() {
            let column_data: Vec<f64> = data.iter().map(|row| row[i]).collect();
            let fits = StatisticalDistributionEngine::fit_distributions(&column_data, None)?;

            // Take top 3 fits
            for fit in fits.into_iter().take(3) {
                fitted_distributions.push(DistributionFitSummary {
                    variable: var_name.clone(),
                    distribution: fit.distribution_name,
                    aic: fit.aic,
                    bic: fit.bic,
                    parameters: fit.parameters,
                });
            }

            // Record best fit
            if let Some(best) = fitted_distributions.last() {
                best_fits.push(format!("{}: {}", var_name, best.distribution));
            }
        }

        Ok(DistributionAnalysis {
            fitted_distributions,
            best_fits,
        })
    }

    /// Analyze correlations
    fn analyze_correlations(data: &[Vec<f64>], var_names: &[String]) -> Result<CorrelationAnalysis, String> {
        let mut significant_correlations = Vec::new();
        let mut correlation_tests = Vec::new();

        for i in 0..var_names.len() {
            for j in (i + 1)..var_names.len() {
                let var1_data: Vec<f64> = data.iter().map(|row| row[i]).collect();
                let var2_data: Vec<f64> = data.iter().map(|row| row[j]).collect();

                // Pearson correlation
                let pearson_corr = CorrelationMethods::pearson_correlation(&var1_data, &var2_data, None, None).map(|(r, _)| r)?;

                // Correlation test using hypothesis testing engine
                let mut rng = Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7);
                let engine = CorrelationHypothesisTestingEngine::new();
                let test_results = engine.correlation_tests(
                    &var1_data, &var2_data, i, j, None, Some(1000), &mut rng
                )?;
                let test_result = test_results.into_iter().find(|r| r.method == "Pearson")
                    .ok_or("Pearson correlation test not found")?;

                correlation_tests.push(CorrelationTest {
                    variables: (var_names[i].clone(), var_names[j].clone()),
                    test_type: "Pearson".to_string(),
                    statistic: test_result.statistic,
                    p_value: test_result.p_value,
                    significant: test_result.significant,
                });

                if test_result.significant {
                    significant_correlations.push(CorrelationResult {
                        var1: var_names[i].clone(),
                        var2: var_names[j].clone(),
                        correlation: pearson_corr,
                        p_value: test_result.p_value,
                        method: "Pearson".to_string(),
                    });
                }
            }
        }

        Ok(CorrelationAnalysis {
            significant_correlations,
            correlation_tests,
        })
    }

    /// Analyze outliers
    fn analyze_outliers(data: &[Vec<f64>], var_names: &[String]) -> Result<OutlierAnalysis, String> {
        let mut detected_outliers = Vec::new();
        let mut robust_statistics = Vec::new();

        for (i, var_name) in var_names.iter().enumerate() {
            let column_data: Vec<f64> = data.iter().map(|row| row[i]).collect();

            // Use the outlier detection engine
            let config = OutlierDetectionConfig::default();
            let outlier_result = OutlierDetectionEngine::detect_outliers(&column_data, &config)?;

            detected_outliers.push(OutlierDetection {
                variable: var_name.clone(),
                method: "Multiple methods".to_string(),
                outlier_indices: outlier_result.combined_outliers.clone(),
                outlier_values: outlier_result.combined_outliers.iter()
                    .map(|&idx| column_data[idx])
                    .collect(),
            });

            // Robust statistics
            let mut sorted_data = column_data.clone();
            sorted_data.sort_by(|a, b| a.total_cmp(b));
            let median = Quantiles::median(&sorted_data);
            let q75 = Quantiles::quantile(&sorted_data, 0.75, QuantileMethod::Type8).unwrap_or(f64::NAN);
            let q25 = Quantiles::quantile(&sorted_data, 0.25, QuantileMethod::Type8).unwrap_or(f64::NAN);
            let iqr = q75 - q25;

            // Use centralized MAD calculation
            let mad_value = dispersion::Dispersion::median_absolute_deviation(&sorted_data, median);

            robust_statistics.push(RobustStats {
                variable: var_name.clone(),
                median,
                mad: mad_value,
                iqr,
            });
        }

        Ok(OutlierAnalysis {
            detected_outliers,
            robust_statistics,
        })
    }

    /// Check if data appears to be time series
    fn is_time_series_data(data: &[Vec<f64>]) -> bool {
        // Simple heuristic: if we have more observations than variables,
        // and data shows temporal patterns, consider it time series
        data.len() > data[0].len() * 2
    }

    /// Analyze time series characteristics
    fn analyze_time_series(data: &[Vec<f64>], var_names: &[String]) -> Result<TimeSeriesAnalysis, String> {
        let mut stationarity_tests = Vec::new();
        let mut decomposition = None;
        let mut spectral_analysis = None;

        for (i, var_name) in var_names.iter().enumerate() {
            let series: Vec<f64> = data.iter().map(|row| row[i]).collect();

            // Stationarity tests
            let adf_result = StationarityEngine::adf_test(
                &series,
                AdfRegressionType::Constant,
                None,
                true
            )?;
            stationarity_tests.push(StationarityTest {
                variable: var_name.clone(),
                test_name: "ADF".to_string(),
                statistic: adf_result.test_statistic,
                p_value: adf_result.p_value,
                is_stationary: adf_result.p_value < 0.05,
            });

            // Decomposition (for first variable only, as example)
            if i == 0 && series.len() > 10 {
                if let Ok(decomp) = TimeSeriesDecompositionEngine::decompose_additive(&series, 7) {
                    decomposition = Some(TimeSeriesDecomposition {
                        trend: decomp.trend,
                        seasonal: decomp.seasonal,
                        residual: decomp.residuals,
                        seasonality_period: Some(decomp.period),
                    });
                }
            }

            // Spectral analysis (for first variable only)
            if i == 0 {
                if let Ok(spectral) = SpectralEngine::periodogram(&series) {
                    spectral_analysis = Some(SpectralAnalysis {
                        frequencies: spectral.frequencies,
                        power_spectrum: spectral.power_spectrum,
                        dominant_frequency: spectral.dominant_frequency,
                        spectral_entropy: spectral.spectral_entropy,
                    });
                }
            }
        }

        Ok(TimeSeriesAnalysis {
            stationarity_tests,
            decomposition,
            spectral_analysis,
            forecasts: None, // Could add forecasting here
        })
    }

    /// Generate analysis recommendations
    fn generate_recommendations(
        data_quality: &DataQualityAssessment,
        distribution_analysis: &DistributionAnalysis,
        correlation_analysis: &CorrelationAnalysis,
        time_series_analysis: &Option<TimeSeriesAnalysis>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Data quality recommendations
        if data_quality.missing_values.total_missing > 0 {
            recommendations.push("Consider investigating patterns in missing data".to_string());
        }

        if data_quality.outliers.total_outliers > data_quality.outliers.outliers_by_variable.len() * 5 {
            recommendations.push("High number of outliers detected - consider robust statistical methods".to_string());
        }

        let normal_vars = data_quality.normality_tests.iter()
            .filter(|test| test.test_result.is_normal)
            .count();
        if normal_vars < data_quality.normality_tests.len() / 2 {
            recommendations.push("Many variables show non-normal distributions - consider transformations".to_string());
        }

        // Distribution recommendations
        let heavy_tail_distributions: Vec<&str> = distribution_analysis.fitted_distributions.iter()
            .filter(|fit| fit.distribution.contains("pareto") || fit.distribution.contains("burr") ||
                          fit.distribution.contains("johnson_su") || fit.distribution.contains("student"))
            .map(|fit| fit.distribution.as_str())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if !heavy_tail_distributions.is_empty() {
            recommendations.push(format!("Heavy-tail distributions detected: {} - consider robust methods",
                                       heavy_tail_distributions.join(", ")));
        }

        // Correlation recommendations
        if correlation_analysis.significant_correlations.len() > correlation_analysis.correlation_tests.len() / 2 {
            recommendations.push("Strong correlations detected - consider dimensionality reduction".to_string());
        }

        // Time series recommendations
        if let Some(ts_analysis) = time_series_analysis {
            let stationary_series = ts_analysis.stationarity_tests.iter()
                .filter(|test| test.is_stationary)
                .count();

            if stationary_series < ts_analysis.stationarity_tests.len() / 2 {
                recommendations.push("Many time series are non-stationary - consider differencing".to_string());
            }

            if ts_analysis.spectral_analysis.is_some() {
                recommendations.push("Spectral analysis available - check for periodic components".to_string());
            }
        }

        recommendations
    }
}