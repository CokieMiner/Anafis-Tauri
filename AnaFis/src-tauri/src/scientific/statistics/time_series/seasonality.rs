//! Seasonality analysis
//!
//! This module provides comprehensive seasonality analysis for time series data
//! including seasonal decomposition, seasonal adjustment, and seasonal forecasting.

use crate::scientific::statistics::descriptive::StatisticalMoments;
use crate::scientific::statistics::CorrelationMethods;
use std::collections::HashMap;

/// Seasonality analysis result
#[derive(Debug, Clone)]
pub struct SeasonalityResult {
    /// Detected seasonal period
    pub period: usize,
    /// Seasonal indices (multiplicative or additive)
    pub seasonal_indices: Vec<f64>,
    /// Seasonal component type
    pub component_type: SeasonalComponent,
    /// Strength of seasonality (0-1)
    pub seasonality_strength: f64,
    /// P-value for seasonality significance
    pub p_value: f64,
    /// Whether seasonality is statistically significant
    pub significant: bool,
}

/// Types of seasonal components
#[derive(Debug, Clone, PartialEq)]
pub enum SeasonalComponent {
    /// Multiplicative seasonality: Y = T * S * R
    Multiplicative,
    /// Additive seasonality: Y = T + S + R
    Additive,
}

/// Seasonal decomposition result
#[derive(Debug, Clone)]
pub struct SeasonalDecomposition {
    /// Original time series
    pub original: Vec<f64>,
    /// Trend component
    pub trend: Vec<f64>,
    /// Seasonal component
    pub seasonal: Vec<f64>,
    /// Residual component
    pub residual: Vec<f64>,
    /// Seasonality result
    pub seasonality_info: SeasonalityResult,
}

/// Seasonality analysis engine
pub struct SeasonalityAnalysis;

impl SeasonalityAnalysis {
    /// Analyze seasonality in time series data
    pub fn analyze_seasonality(data: &[f64], max_period: Option<usize>) -> Result<SeasonalityResult, String> {
        if data.len() < 6 {
            return Err("Need at least 6 data points for seasonality analysis".to_string());
        }

        let max_period = max_period.unwrap_or(data.len() / 2);

        // Try different periods and find the best one
        let mut best_result: Option<SeasonalityResult> = None;
        let mut best_strength = 0.0;

        for period in 2..=max_period.min(data.len() / 2) {
            if let Ok(result) = Self::test_seasonality(data, period) {
                if result.seasonality_strength > best_strength {
                    best_strength = result.seasonality_strength;
                    best_result = Some(result);
                }
            }
        }

        best_result.ok_or_else(|| "No significant seasonality detected".to_string())
    }

    /// Test for seasonality at a specific period
    pub fn test_seasonality(data: &[f64], period: usize) -> Result<SeasonalityResult, String> {
        if period < 2 || period >= data.len() {
            return Err("Invalid period for seasonality test".to_string());
        }

        // Calculate seasonal indices
        let seasonal_indices = Self::calculate_seasonal_indices(data, period)?;

        // Determine component type
        let component_type = Self::determine_component_type(data, &seasonal_indices, period);

        // Calculate seasonality strength
        let seasonality_strength = Self::calculate_seasonality_strength(data, &seasonal_indices, period, &component_type);

        // Calculate statistical significance
        let p_value = Self::seasonality_significance_test(data, period);

        Ok(SeasonalityResult {
            period,
            seasonal_indices,
            component_type,
            seasonality_strength,
            p_value,
            significant: p_value < 0.05 && seasonality_strength > 0.3,
        })
    }

    /// Perform seasonal decomposition (delegates to TimeSeriesDecompositionEngine)
    pub fn seasonal_decomposition(data: &[f64], period: usize) -> Result<SeasonalDecomposition, String> {
        use crate::scientific::statistics::time_series::decomposition::TimeSeriesDecompositionEngine;
        
        let components = TimeSeriesDecompositionEngine::decompose_additive(data, period)?;
        
        // Analyze seasonality using our specialized method
        let seasonality_info = Self::test_seasonality(data, period)?;

        Ok(SeasonalDecomposition {
            original: data.to_vec(),
            trend: components.trend,
            seasonal: components.seasonal,
            residual: components.residuals,
            seasonality_info,
        })
    }

    /// Seasonally adjust time series data
    pub fn seasonal_adjustment(data: &[f64], period: usize) -> Result<Vec<f64>, String> {
        let decomposition = Self::seasonal_decomposition(data, period)?;

        let adjusted: Vec<f64> = match decomposition.seasonality_info.component_type {
            SeasonalComponent::Additive => {
                data.iter().zip(decomposition.seasonal.iter())
                    .map(|(y, s)| y - s)
                    .collect()
            },
            SeasonalComponent::Multiplicative => {
                data.iter().zip(decomposition.seasonal.iter())
                    .map(|(y, s)| if *s != 0.0 { y / s } else { *y })
                    .collect()
            },
        };

        Ok(adjusted)
    }

    /// Calculate seasonal indices using ratio-to-moving-average method
    fn calculate_seasonal_indices(data: &[f64], period: usize) -> Result<Vec<f64>, String> {
        let _n = data.len();

        // Calculate centered moving average
        let ma = {
            use crate::scientific::statistics::time_series::trend::TrendAnalysisEngine;
            TrendAnalysisEngine::centered_moving_average(data, period)?
        };

        // Calculate ratios (for multiplicative) or differences (for additive)
        let ratios: Vec<f64> = data.iter().zip(ma.iter())
            .map(|(y, m)| if *m != 0.0 { y / m } else { 1.0 })
            .collect();

        // Group by season and calculate averages
        let mut seasonal_groups: HashMap<usize, Vec<f64>> = HashMap::new();

        for (i, ratio) in ratios.iter().enumerate() {
            let season = i % period;
            seasonal_groups.entry(season).or_default().push(*ratio);
        }

        // Calculate seasonal indices
        let mut seasonal_indices = vec![0.0; period];
        for (season, seasonal_index) in seasonal_indices.iter_mut().enumerate() {
            if let Some(group) = seasonal_groups.get(&season) {
                let avg = group.iter().sum::<f64>() / group.len() as f64;
                *seasonal_index = avg;
            }
        }

        // Normalize seasonal indices (average should be 1.0 for multiplicative)
        let avg_index = seasonal_indices.iter().sum::<f64>() / period as f64;
        if avg_index != 0.0 {
            for index in &mut seasonal_indices {
                *index /= avg_index;
            }
        }

        Ok(seasonal_indices)
    }

    /// Determine whether to use additive or multiplicative seasonality
    fn determine_component_type(data: &[f64], seasonal_indices: &[f64], _period: usize) -> SeasonalComponent {
        // Calculate coefficient of variation for original data and seasonal indices
        let data_cv = data.std_dev() / data.mean().abs();
        let seasonal_cv = seasonal_indices.std_dev() / seasonal_indices.mean().abs();

        // If seasonal variation is large relative to overall variation, use multiplicative
        if seasonal_cv > data_cv * 0.5 {
            SeasonalComponent::Multiplicative
        } else {
            SeasonalComponent::Additive
        }
    }

    /// Calculate strength of seasonality
    fn calculate_seasonality_strength(data: &[f64], seasonal_indices: &[f64], period: usize, _component_type: &SeasonalComponent) -> f64 {
        let n = data.len();

        // Calculate seasonal component for the entire series
        let seasonal_component: Vec<f64> = (0..n)
            .map(|i| seasonal_indices[i % period])
            .collect();

        // Calculate correlation between seasonal component and detrended series
        let detrended = Self::detrend_series(data).unwrap_or_else(|_| data.to_vec());

        let correlation = CorrelationMethods::pearson_correlation(&seasonal_component, &detrended, None, None)
            .map(|(r, _)| r)
            .unwrap_or(0.0);
        correlation.abs().clamp(0.0, 1.0)
    }

    /// Test statistical significance of seasonality
    fn seasonality_significance_test(data: &[f64], period: usize) -> f64 {
        // Use Friedman test or similar non-parametric test
        // Simplified implementation: compare variance of seasonal means to overall variance

        let n = data.len();
        let mut seasonal_groups: HashMap<usize, Vec<f64>> = HashMap::new();

        for (i, &value) in data.iter().enumerate() {
            let season = i % period;
            seasonal_groups.entry(season).or_default().push(value);
        }

        // Calculate seasonal means
        let seasonal_means: Vec<f64> = (0..period)
            .filter_map(|s| seasonal_groups.get(&s))
            .map(|group| group.iter().sum::<f64>() / group.len() as f64)
            .collect();

        if seasonal_means.len() < period {
            return 1.0; // Not enough data
        }

        // Calculate F-statistic for seasonality
        let overall_mean = data.iter().sum::<f64>() / n as f64;
        let ss_between = seasonal_means.iter()
            .zip(seasonal_groups.values())
            .map(|(mean, group)| group.len() as f64 * (mean - overall_mean).powi(2))
            .sum::<f64>();

        let ss_within = seasonal_groups.values()
            .flat_map(|group| group.iter().map(|&x| (x - seasonal_means[seasonal_groups.keys().position(|&k| std::ptr::eq(group, seasonal_groups.get(&k).unwrap())).unwrap()]).powi(2)))
            .sum::<f64>();

        let df_between = period - 1;
        let df_within = n - period;

        if df_within > 0 && ss_within > 0.0 {
            let f_stat = (ss_between / df_between as f64) / (ss_within / df_within as f64);
            // P-value approximation
            let cdf = crate::scientific::statistics::distributions::distribution_functions::fisher_snedecor_cdf(f_stat, df_between as f64, df_within as f64);
            1.0 - cdf
        } else {
            1.0
        }
    }

    /// Detrend series using linear regression
    fn detrend_series(data: &[f64]) -> Result<Vec<f64>, String> {
        use crate::scientific::statistics::time_series::trend::TrendAnalysisEngine;

        let trend_result = TrendAnalysisEngine::linear_trend(data)?;
        let detrended: Vec<f64> = data.iter().zip(trend_result.trend_values.iter())
            .map(|(y, t)| y - t)
            .collect();

        Ok(detrended)
    }

    /// Forecast using seasonal model
    pub fn seasonal_forecast(data: &[f64], period: usize, steps_ahead: usize) -> Result<Vec<f64>, String> {
        let decomposition = Self::seasonal_decomposition(data, period)?;

        // Simple forecast: extend trend and repeat seasonal pattern
        let last_trend = *decomposition.trend.last().ok_or("No trend data")?;
        let trend_slope = if decomposition.trend.len() >= 2 {
            let n = decomposition.trend.len();
            decomposition.trend[n - 1] - decomposition.trend[n - 2]
        } else {
            0.0
        };

        let mut forecast = Vec::with_capacity(steps_ahead);

        for i in 0..steps_ahead {
            let trend_value = last_trend + trend_slope * (i + 1) as f64;
            let seasonal_value = decomposition.seasonal[(data.len() + i) % period];

            let predicted = match decomposition.seasonality_info.component_type {
                SeasonalComponent::Additive => trend_value + seasonal_value,
                SeasonalComponent::Multiplicative => trend_value * seasonal_value,
            };

            forecast.push(predicted);
        }

        Ok(forecast)
    }

    /// Calculate seasonal statistics
    pub fn seasonal_statistics(data: &[f64], period: usize) -> Result<HashMap<String, f64>, String> {
        let mut stats = HashMap::new();

        let seasonality_result = Self::test_seasonality(data, period)?;

        stats.insert("period".to_string(), period as f64);
        stats.insert("seasonality_strength".to_string(), seasonality_result.seasonality_strength);
        stats.insert("p_value".to_string(), seasonality_result.p_value);
        stats.insert("significant".to_string(), if seasonality_result.significant { 1.0 } else { 0.0 });

        // Add seasonal indices
        for (i, &index) in seasonality_result.seasonal_indices.iter().enumerate() {
            stats.insert(format!("seasonal_index_{}", i), index);
        }

        Ok(stats)
    }
}