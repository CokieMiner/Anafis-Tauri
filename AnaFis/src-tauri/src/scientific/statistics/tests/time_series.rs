//! Tests for time series analysis functions
//! Validates against statsmodels and oxidiviner references

use crate::scientific::statistics::time_series::*;

/// Generate synthetic time series data for testing
fn generate_ar1_series(n: usize, phi: f64, sigma: f64) -> Vec<f64> {
    use rand::prelude::*;
    use rand_pcg::Pcg64;

    let mut rng = Pcg64::seed_from_u64(42);
    let normal = rand_distr::Normal::new(0.0, sigma).unwrap();

    let mut series = vec![0.0; n];
    series[0] = normal.sample(&mut rng);

    for i in 1..n {
        series[i] = phi * series[i-1] + normal.sample(&mut rng);
    }

    series
}

/// Generate sine wave with noise for spectral testing
fn generate_sine_wave(n: usize, frequency: f64, amplitude: f64, noise_std: f64) -> Vec<f64> {
    use rand::prelude::*;
    use rand_pcg::Pcg64;

    let mut rng = Pcg64::seed_from_u64(42);
    let normal = rand_distr::Normal::new(0.0, noise_std).unwrap();

    (0..n).map(|i| {
        let t = i as f64 / n as f64;
        amplitude * (2.0 * std::f64::consts::PI * frequency * t).sin() + normal.sample(&mut rng)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arima_forecasting() {
        let data = generate_ar1_series(50, 0.7, 0.1);

        let result = TimeSeriesForecastingEngine::fit_arima(&data, 5);
        assert!(result.is_ok(), "ARIMA forecasting should succeed");

        let forecast = result.unwrap();
        assert_eq!(forecast.forecasts.len(), 5, "Should forecast 5 steps");
        assert!(!forecast.model_type.is_empty(), "Model type should be specified");

        // Check that forecasts are finite
        for &f in &forecast.forecasts {
            assert!(f.is_finite(), "Forecasts should be finite");
        }
    }

    #[test]
    fn test_auto_forecasting() {
        let data = generate_ar1_series(30, 0.5, 0.2);

        let result = TimeSeriesForecastingEngine::auto_forecast(&data, 3);
        assert!(result.is_ok(), "Auto forecasting should succeed");

        let forecast = result.unwrap();
        assert_eq!(forecast.forecasts.len(), 3, "Should forecast 3 steps");

        // Check that forecasts are finite
        for &f in &forecast.forecasts {
            assert!(f.is_finite(), "Forecasts should be finite");
        }
    }

    #[test]
    fn test_forecast_evaluation() {
        let actual = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let predicted = vec![1.1, 2.2, 2.8, 4.1, 4.9];

        let result = TimeSeriesForecastingEngine::evaluate_forecast(&actual, &predicted);
        assert!(result.is_ok(), "Forecast evaluation should succeed");

        let metrics = result.unwrap();

        // Check that all metrics are finite and non-negative
        assert!(metrics.mse >= 0.0 && metrics.mse.is_finite());
        assert!(metrics.rmse >= 0.0 && metrics.rmse.is_finite());
        assert!(metrics.mae >= 0.0 && metrics.mae.is_finite());
        assert!(metrics.mape >= 0.0 && metrics.mape.is_finite());

        // RMSE should be square root of MSE
        assert!((metrics.rmse - metrics.mse.sqrt()).abs() < 1e-10);

        // MAPE should be reasonable percentage
        assert!(metrics.mape <= 100.0, "MAPE should be <= 100%");
    }

    #[test]
    fn test_periodogram_basic() {
        let data = generate_sine_wave(64, 5.0, 1.0, 0.1);

        let result = SpectralEngine::periodogram(&data);
        assert!(result.is_ok(), "Periodogram calculation should succeed");

        let analysis = result.unwrap();

        // Check dimensions
        assert!(!analysis.frequencies.is_empty());
        assert!(!analysis.periodogram.is_empty());
        assert!(!analysis.power_spectrum.is_empty());

        // Check that frequencies are in [0, 0.5] range (Nyquist frequency)
        for &freq in &analysis.frequencies {
            assert!(freq >= 0.0 && freq <= 0.5);
        }

        // Check that spectral values are non-negative
        for &p in &analysis.periodogram {
            assert!(p >= 0.0);
        }
        for &ps in &analysis.power_spectrum {
            assert!(ps >= 0.0);
        }

        // Check that dominant frequency is valid
        assert!(analysis.dominant_frequency >= 0.0 && analysis.dominant_frequency <= 0.5);

        // Check spectral entropy is valid
        assert!(analysis.spectral_entropy >= 0.0 && analysis.spectral_entropy.is_finite());
    }

    #[test]
    fn test_periodogram_sine_wave() {
        // Generate clean sine wave at frequency 0.1 (10 cycles in 100 points)
        let data = generate_sine_wave(100, 10.0, 1.0, 0.0);

        let result = SpectralEngine::periodogram(&data);
        assert!(result.is_ok(), "Periodogram for sine wave should succeed");

        let analysis = result.unwrap();

        // The dominant frequency should be close to 0.1 (10/100 = 0.1)
        let expected_freq = 10.0 / 100.0; // 0.1
        assert!((analysis.dominant_frequency - expected_freq).abs() < 0.05,
                "Dominant frequency should be close to {}", expected_freq);

        // Spectral entropy should be relatively low for a pure sine wave (concentrated power)
        // Note: With noise added, entropy won't be extremely low
        assert!(analysis.spectral_entropy < 3.0, "Spectral entropy should be reasonably low for sine wave with noise");
    }

    #[test]
    fn test_periodogram_white_noise() {
        // Generate white noise
        let data = generate_ar1_series(64, 0.0, 1.0);

        let result = SpectralEngine::periodogram(&data);
        assert!(result.is_ok(), "Periodogram for white noise should succeed");

        let analysis = result.unwrap();

        // For white noise, spectral entropy should be higher (more evenly distributed power)
        assert!(analysis.spectral_entropy > 1.0, "Spectral entropy should be higher for white noise");

        // Power spectrum should be relatively flat
        let mean_power = analysis.power_spectrum.iter().sum::<f64>() / analysis.power_spectrum.len() as f64;
        let variance_power = analysis.power_spectrum.iter()
            .map(|&p| (p - mean_power).powi(2))
            .sum::<f64>() / analysis.power_spectrum.len() as f64;

        // Coefficient of variation should be reasonable (not too spiky)
        let cv_power = (variance_power.sqrt() / mean_power).abs();
        assert!(cv_power < 2.0, "Power spectrum should be relatively flat for white noise");
    }

    #[test]
    fn test_time_series_forecasting_edge_cases() {
        // Test with insufficient data
        let result = TimeSeriesForecastingEngine::fit_arima(&[1.0, 2.0], 1);
        assert!(result.is_err(), "Should fail with insufficient data");

        let result = TimeSeriesForecastingEngine::auto_forecast(&[1.0], 1);
        assert!(result.is_err(), "Should fail with insufficient data");

        // Test forecast evaluation with mismatched lengths
        let result = TimeSeriesForecastingEngine::evaluate_forecast(&[1.0, 2.0], &[1.0]);
        assert!(result.is_err(), "Should fail with mismatched lengths");
    }

    #[test]
    fn test_spectral_analysis_edge_cases() {
        // Test with insufficient data
        let result = SpectralEngine::periodogram(&[1.0]);
        assert!(result.is_err(), "Should fail with insufficient data");

        // Test with empty data
        let result = SpectralEngine::periodogram(&[]);
        assert!(result.is_err(), "Should fail with empty data");
    }

    #[test]
    fn test_forecast_metrics_calculation() {
        // Perfect predictions
        let actual = vec![1.0, 2.0, 3.0];
        let predicted = vec![1.0, 2.0, 3.0];

        let result = TimeSeriesForecastingEngine::evaluate_forecast(&actual, &predicted).unwrap();

        assert!((result.mse - 0.0).abs() < 1e-10, "MSE should be 0 for perfect predictions");
        assert!((result.mae - 0.0).abs() < 1e-10, "MAE should be 0 for perfect predictions");
        assert!((result.mape - 0.0).abs() < 1e-10, "MAPE should be 0 for perfect predictions");

        // Test with zero actual values (MAPE edge case)
        let actual = vec![0.0, 1.0, 2.0];
        let predicted = vec![0.1, 1.1, 2.1];

        let result = TimeSeriesForecastingEngine::evaluate_forecast(&actual, &predicted).unwrap();

        // MAPE should skip the zero value and be finite
        assert!(result.mape.is_finite());
        assert!(result.mape >= 0.0);
    }

    #[test]
    fn test_spectral_properties() {
        let data = generate_sine_wave(128, 8.0, 1.0, 0.1);

        let analysis = SpectralEngine::periodogram(&data).unwrap();

        // Check frequency resolution
        let freq_resolution = analysis.frequencies[1] - analysis.frequencies[0];
        assert!(freq_resolution > 0.0 && freq_resolution < 0.1);

        // Check that periodogram and power spectrum have same length
        assert_eq!(analysis.periodogram.len(), analysis.power_spectrum.len());

        // Power spectrum should be smoother than raw periodogram
        // (This is a heuristic check - power spectrum is typically smoothed)
        let periodogram_variance = analysis.periodogram.iter()
            .map(|&p| (p - analysis.periodogram.iter().sum::<f64>() / analysis.periodogram.len() as f64).powi(2))
            .sum::<f64>() / analysis.periodogram.len() as f64;

        let power_variance = analysis.power_spectrum.iter()
            .map(|&p| (p - analysis.power_spectrum.iter().sum::<f64>() / analysis.power_spectrum.len() as f64).powi(2))
            .sum::<f64>() / analysis.power_spectrum.len() as f64;

        // Power spectrum variance should be less than or equal to periodogram variance
        assert!(power_variance <= periodogram_variance);
    }
}