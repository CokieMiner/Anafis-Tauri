use crate::scientific::statistics::comprehensive_analysis::layer3_algorithms::time_series::TimeSeriesDecompositionEngine;
use crate::scientific::statistics::types::TrendAnalysis;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct TimeSeriesAnalysisResult {
    pub is_temporal: bool,
    pub trend_analysis: Option<TrendAnalysis>,
    pub seasonality_analysis: Option<SeasonalityAnalysis>,
    pub autocorrelation: Option<Vec<f64>>,
}

#[derive(Debug, Clone)]
pub struct SeasonalityAnalysis {
    pub seasonality_present: bool,
    pub period: Option<usize>,
    pub strength: f64,
    pub spectral_peaks: Option<Vec<(f64, f64)>>, // (frequency, power)
    pub dominant_frequency: Option<f64>,
}

#[derive(Debug)]
struct SpectralResult {
    peaks: Option<Vec<(f64, f64)>>, // (frequency, power)
}

/// Time Series Analysis Coordinator
/// Coordinates temporal pattern analysis
pub struct TimeSeriesAnalysisCoordinator;

impl TimeSeriesAnalysisCoordinator {
    /// Analyze time series patterns
    pub fn analyze(data: &[f64]) -> Result<TimeSeriesAnalysisResult, String> {
        if data.len() < 10 {
            return Err("Time series too short for meaningful analysis".to_string());
        }

        // Test for temporal patterns
        let is_temporal = Self::detect_temporal_patterns(data)?;

        if !is_temporal {
            return Ok(TimeSeriesAnalysisResult {
                is_temporal: false,
                trend_analysis: None,
                seasonality_analysis: Some(SeasonalityAnalysis {
                    seasonality_present: false,
                    period: None,
                    strength: 0.0,
                    spectral_peaks: None,
                    dominant_frequency: None,
                }),
                autocorrelation: None,
            });
        }

        // Trend analysis
        let trend_analysis = TimeSeriesDecompositionEngine::trend_test(data)?;

        // Seasonality analysis (assume daily data, look for weekly patterns)
        let seasonality_analysis = Self::analyze_seasonality(data)?;

        // Autocorrelation
        let max_lag = (data.len() / 4).min(50);
        let autocorrelation = TimeSeriesDecompositionEngine::autocorrelation(data, max_lag)?;

        Ok(TimeSeriesAnalysisResult {
            is_temporal: true,
            trend_analysis: Some(trend_analysis),
            seasonality_analysis: Some(seasonality_analysis),
            autocorrelation: Some(autocorrelation),
        })
    }

    /// Analyze multiple time series in parallel
    pub fn analyze_multiple_series(series: &[&[f64]]) -> Result<Vec<TimeSeriesAnalysisResult>, String> {
        if series.is_empty() {
            return Ok(Vec::new());
        }

        // Analyze all series in parallel, handling short series gracefully
        let results: Vec<Result<TimeSeriesAnalysisResult, String>> = series
            .par_iter()
            .map(|data| {
                if data.len() < 10 {
                    // Return default result for short series
                    Ok(TimeSeriesAnalysisResult {
                        is_temporal: false,
                        trend_analysis: None,
                        seasonality_analysis: Some(SeasonalityAnalysis {
                            seasonality_present: false,
                            period: None,
                            strength: 0.0,
                            spectral_peaks: None,
                            dominant_frequency: None,
                        }),
                        autocorrelation: None,
                    })
                } else {
                    Self::analyze(data)
                }
            })
            .collect();

        // Collect results, propagating any errors
        let mut final_results = Vec::with_capacity(results.len());
        for result in results {
            final_results.push(result?);
        }

        Ok(final_results)
    }

    /// Detect if data shows temporal patterns
    fn detect_temporal_patterns(data: &[f64]) -> Result<bool, String> {
        if data.len() < 20 {
            return Ok(false);
        }

        // Check for autocorrelation at lag 1
        let autocorr = TimeSeriesDecompositionEngine::autocorrelation(data, 1)?;
        let lag1_autocorr = autocorr.first().copied().unwrap_or(0.0);

        // Check for trend
        let trend_test = TimeSeriesDecompositionEngine::trend_test(data)?;

        // Consider temporal if significant autocorrelation or trend
        Ok(lag1_autocorr.abs() > 0.3 || trend_test.trend_present)
    }

    /// Analyze seasonality
    fn analyze_seasonality(data: &[f64]) -> Result<SeasonalityAnalysis, String> {
        if data.len() < 20 {
            return Ok(SeasonalityAnalysis {
                seasonality_present: false,
                period: None,
                strength: 0.0,
                spectral_peaks: None,
                dominant_frequency: None,
            });
        }

        // Perform spectral analysis
        let spectral_result = Self::spectral_analysis(data)?;

        // Detect dominant frequency and period
        let (dominant_frequency, period) = if let Some(peaks) = &spectral_result.peaks {
            if !peaks.is_empty() {
                // Find the peak with highest power
                let (freq, _) = peaks.iter()
                    .max_by(|a, b| match a.1.partial_cmp(&b.1) {
                        Some(ord) => ord,
                        None => std::cmp::Ordering::Equal,
                    })
                    .expect("peaks is not empty");
                let period_est = if *freq > 0.0 { (1.0 / freq).round() as usize } else { 0 };
                (*freq, if period_est > 0 { Some(period_est) } else { None })
            } else {
                (0.0, None)
            }
        } else {
            (0.0, None)
        };

        // Traditional seasonality test with detected period or default
        let test_period = period.unwrap_or(7); // Default to weekly if no period detected
        let strength = if period.is_some() && data.len() >= test_period * 2 {
            Self::calculate_seasonality_strength(data, test_period)?
        } else {
            0.0
        };

        // Determine if seasonality is present
        let seasonality_present = strength > 0.1 && period.is_some(); // Threshold for significance

        Ok(SeasonalityAnalysis {
            seasonality_present,
            period,
            strength,
            spectral_peaks: spectral_result.peaks,
            dominant_frequency: if dominant_frequency > 0.0 { Some(dominant_frequency) } else { None },
        })
    }

    /// Perform spectral analysis using periodogram
    fn spectral_analysis(data: &[f64]) -> Result<SpectralResult, String> {
        let n = data.len();
        if n < 10 {
            return Ok(SpectralResult { peaks: None });
        }

        // Remove linear trend (detrend)
        let detrended = Self::detrend_data(data)?;

        // Compute periodogram
        let periodogram = Self::compute_periodogram(&detrended)?;

        // Find peaks in the periodogram
        let peaks = Self::find_spectral_peaks(&periodogram, n)?;

        Ok(SpectralResult { peaks: Some(peaks) })
    }

    /// Detrend data by removing linear trend
    fn detrend_data(data: &[f64]) -> Result<Vec<f64>, String> {
        let n = data.len();
        let mut detrended = vec![0.0; n];

        // Simple linear detrending
        let x_mean = (n - 1) as f64 / 2.0;
        let y_mean = data.iter().sum::<f64>() / n as f64;

        let numerator: f64 = data.iter().enumerate()
            .map(|(i, &y)| (i as f64 - x_mean) * (y - y_mean))
            .sum();
        let denominator: f64 = data.iter().enumerate()
            .map(|(i, _)| (i as f64 - x_mean).powi(2))
            .sum();

        let slope = if denominator > 0.0 { numerator / denominator } else { 0.0 };
        let intercept = y_mean - slope * x_mean;

        for (i, &y) in data.iter().enumerate() {
            let trend = intercept + slope * i as f64;
            detrended[i] = y - trend;
        }

        Ok(detrended)
    }

    /// Compute periodogram (simplified FFT-based spectral estimation)
    fn compute_periodogram(data: &[f64]) -> Result<Vec<f64>, String> {
        let n = data.len();
        let mut periodogram = vec![0.0; n / 2 + 1];

        // Simple DFT-based periodogram (not optimized)
        for (k, periodogram_val) in periodogram.iter_mut().enumerate().skip(1).take(n/2) {
            let frequency = k as f64 / n as f64;

            let mut real = 0.0;
            let mut imag = 0.0;

            for (t, &x) in data.iter().enumerate() {
                let angle = -2.0 * std::f64::consts::PI * frequency * t as f64;
                real += x * angle.cos();
                imag += x * angle.sin();
            }

            *periodogram_val = (real * real + imag * imag) / n as f64;
        }

        Ok(periodogram)
    }

    /// Find peaks in the periodogram
    fn find_spectral_peaks(periodogram: &[f64], n: usize) -> Result<Vec<(f64, f64)>, String> {
        let mut peaks = Vec::new();
        let threshold = periodogram.iter().sum::<f64>() / periodogram.len() as f64 * 2.0; // Simple threshold

        for (k, &power) in periodogram.iter().enumerate() {
            if k == 0 || k >= periodogram.len() - 1 { continue; }

            // Check if it's a local maximum
            if power > periodogram[k - 1] && power > periodogram[k + 1] && power > threshold {
                let frequency = k as f64 / n as f64;
                peaks.push((frequency, power));
            }
        }

        // Sort by power (descending)
        peaks.sort_by(|a, b| match b.1.partial_cmp(&a.1) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Equal,
        });

        // Return top peaks
        Ok(peaks.into_iter().take(5).collect())
    }

    /// Calculate seasonality strength for a given period
    fn calculate_seasonality_strength(data: &[f64], period: usize) -> Result<f64, String> {
        if data.len() < period * 2 {
            return Ok(0.0);
        }

        // Simple seasonality test: compare within-period variance to between-period variance
        let mut period_means = vec![0.0; period];
        let mut period_counts = vec![0; period];

        for (i, &value) in data.iter().enumerate() {
            let p = i % period;
            period_means[p] += value;
            period_counts[p] += 1;
        }

        for i in 0..period {
            if period_counts[i] > 0 {
                period_means[i] /= period_counts[i] as f64;
            }
        }

        // Compute overall mean
        let overall_mean = data.iter().sum::<f64>() / data.len() as f64;

        // Compute between-period sum of squares
        let ssb = period_means.iter()
            .zip(period_counts.iter())
            .map(|(mean, &count)| count as f64 * (mean - overall_mean).powi(2))
            .sum::<f64>();

        // Compute within-period sum of squares
        let ssw = data.iter().enumerate()
            .map(|(i, &value)| {
                let p = i % period;
                (value - period_means[p]).powi(2)
            })
            .sum::<f64>();

        let total_ss = ssb + ssw;
        Ok(if total_ss > 0.0 { ssb / total_ss } else { 0.0 })
    }
}
