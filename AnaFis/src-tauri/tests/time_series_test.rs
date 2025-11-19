#[cfg(test)]
mod tests {
    use anafis_lib::scientific::statistics::comprehensive_analysis::layer3_algorithms::time_series::{ProphetEngine, ProphetConfig, GrowthModel, Holiday};

    #[test]
    fn test_prophet_basic_forecasting() {
        // Create simple test data with trend and seasonality
        let mut data = Vec::new();
        for i in 0..50 {
            let trend = i as f64 * 0.1;
            let seasonal = (i as f64 * 2.0 * std::f64::consts::PI / 7.0).sin() * 2.0; // Weekly pattern
            let noise = (i as f64 * 0.1).sin() * 0.5; // Some noise
            data.push(trend + seasonal + noise + 10.0);
        }

        let config = ProphetConfig {
            seasonality_periods: Some(vec![7]),
            changepoint_prior_scale: Some(0.05),
            seasonality_prior_scale: Some(10.0),
            holidays: None,
            growth_model: Some(GrowthModel::Linear),
            auto_tune: Some(false),
        };
        let result = ProphetEngine::fit_prophet(&data, 10, config);
        assert!(result.is_ok());

        let forecast = result.unwrap();
        assert_eq!(forecast.forecasts.len(), 10);
        assert_eq!(forecast.trend_component.len(), data.len());
        assert_eq!(forecast.seasonal_component.len(), data.len());
        assert_eq!(forecast.holiday_component.len(), data.len());

        // Forecasts should be reasonable (not NaN or infinite)
        for &f in &forecast.forecasts {
            assert!(f.is_finite());
        }
    }

    #[test]
    fn test_prophet_insufficient_data() {
        let data = vec![1.0, 2.0, 3.0]; // Too few data points
        let config = ProphetConfig::default();
        let result = ProphetEngine::fit_prophet(&data, 5, config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient data"));
    }

    #[test]
    fn test_prophet_with_api() {
        let data = (0..30).map(|i| 10.0 + i as f64 * 0.5 + (i as f64 * 0.3).sin()).collect::<Vec<f64>>();
        let config = ProphetConfig::default();
        let result = ProphetEngine::fit_prophet(&data, 5, config);
        assert!(result.is_ok());

        let forecast = result.unwrap();
        assert_eq!(forecast.forecasts.len(), 5);
    }

    #[test]
    fn test_prophet_logistic_growth() {
        let mut data = Vec::new();
        for i in 0..50 {
            let t = i as f64;
            // Logistic growth towards capacity of 100
            let logistic = 100.0 / (1.0 + 50.0 * (-0.1 * (t - 25.0)).exp());
            let noise = (i as f64 * 0.1).sin() * 2.0;
            data.push(logistic + noise);
        }

        let config = ProphetConfig {
            seasonality_periods: Some(vec![7]),
            changepoint_prior_scale: Some(0.05),
            seasonality_prior_scale: Some(10.0),
            holidays: None,
            growth_model: Some(GrowthModel::Logistic { capacity: 100.0 }),
            auto_tune: Some(false),
        };
        let result = ProphetEngine::fit_prophet(&data, 10, config);
        assert!(result.is_ok());

        let forecast = result.unwrap();
        assert_eq!(forecast.forecasts.len(), 10);

        // Forecasts should approach capacity
        for &f in &forecast.forecasts {
            assert!(f.is_finite());
            assert!((0.0..=120.0).contains(&f)); // Should be reasonable
        }
    }

    #[test]
    fn test_prophet_with_holidays() {
        let mut data = (0..50).map(|i| 10.0 + i as f64 * 0.1).collect::<Vec<f64>>();

        // Add holiday effects
        let holidays = vec![
            Holiday {
                name: "Christmas".to_string(),
                dates: vec![25], // Day 25
                prior_scale: 5.0,
            }
        ];

        // Add holiday effect to data
        if 25 < data.len() {
            data[25] += 15.0; // Big spike on Christmas
        }

        let config = ProphetConfig {
            seasonality_periods: Some(vec![7]),
            changepoint_prior_scale: Some(0.05),
            seasonality_prior_scale: Some(10.0),
            holidays: Some(holidays),
            growth_model: Some(GrowthModel::Linear),
            auto_tune: Some(false),
        };
        let result = ProphetEngine::fit_prophet(&data, 5, config);
        assert!(result.is_ok());

        let forecast = result.unwrap();
        assert_eq!(forecast.forecasts.len(), 5);
        assert_eq!(forecast.holiday_component.len(), data.len());

        // Holiday component should have non-zero values around day 25
        assert!(forecast.holiday_component[25].abs() > 0.0);
    }

    #[test]
    fn test_linear_regression() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0]; // Perfect linear relationship

        let (slope, intercept) = ProphetEngine::linear_regression(&x, &y).unwrap();
        assert!((slope - 1.0).abs() < 1e-10);
        assert!((intercept - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_trend_fitting() {
        let time_index = (0..20).map(|i| i as f64).collect::<Vec<f64>>();
        let data = time_index.iter().map(|&t| 2.0 * t + 5.0).collect::<Vec<f64>>();

        let trend = ProphetEngine::fit_trend(&time_index, &data, 0.05, &GrowthModel::Linear).unwrap();
        assert_eq!(trend.len(), data.len());

        // Should fit the linear trend reasonably well
        for (i, &t_val) in trend.iter().enumerate() {
            let expected = 2.0 * i as f64 + 5.0;
            assert!((t_val - expected).abs() < 1.0); // Allow some tolerance
        }
    }
}