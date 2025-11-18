use crate::scientific::statistics::types::*;

/// Prophet Forecasting Engine
/// Implements Facebook Prophet-style forecasting with trend, seasonality, and holiday effects
pub struct ProphetEngine;

impl ProphetEngine {
    /// Fit Prophet model and generate forecasts
    pub fn fit_prophet(data: &[f64], forecast_steps: usize, config: ProphetConfig) -> Result<ProphetForecast, String> {
        if data.len() < 10 {
            return Err("Insufficient data for Prophet modeling (need at least 10 observations)".to_string());
        }

        // Extract parameters from config
        let periods = config.seasonality_periods.unwrap_or_else(|| vec![7, 365]); // Weekly and yearly seasonality
        let changepoint_scale = config.changepoint_prior_scale.unwrap_or(0.05);
        let seasonality_scale = config.seasonality_prior_scale.unwrap_or(10.0);
        let growth = config.growth_model.unwrap_or(GrowthModel::Linear);
        let tune_params = config.auto_tune.unwrap_or(false);
        let holidays = config.holidays.as_deref();

        // Auto-tune parameters if requested
        let (final_changepoint_scale, final_seasonality_scale) = if tune_params {
            Self::auto_tune_parameters(data, &periods, &growth)?
        } else {
            (changepoint_scale, seasonality_scale)
        };

        // Create time index (0, 1, 2, ..., n-1)
        let time_index: Vec<f64> = (0..data.len()).map(|i| i as f64).collect();

        // 1. Fit trend component with changepoints using PELT
        let trend_component = Self::fit_trend(&time_index, data, final_changepoint_scale, &growth)?;

        // 2. Fit seasonal components
        let seasonal_component = Self::fit_seasonal(&time_index, data, &trend_component, &periods, final_seasonality_scale)?;

        // 3. Fit holiday component
        let holiday_component = Self::fit_holidays(&time_index, data, &trend_component, &seasonal_component, holidays)?;

        // 4. Generate forecasts
        let forecasts = Self::generate_forecasts(
            data.len(),
            forecast_steps,
            &trend_component,
            &seasonal_component,
            &holiday_component,
            &periods,
            &growth,
        )?;

        let model_info = format!(
            "Prophet model: {} observations, {} forecast steps, seasonality periods: {:?}, growth: {:?}, auto-tune: {}",
            data.len(),
            forecast_steps,
            periods,
            growth,
            tune_params
        );

        Ok(ProphetForecast {
            forecasts,
            trend_component,
            seasonal_component,
            holiday_component,
            model_info,
        })
    }

    /// Fit piecewise linear/logistic trend with changepoint detection using PELT
    pub fn fit_trend(time_index: &[f64], data: &[f64], changepoint_prior_scale: f64, growth_model: &GrowthModel) -> Result<Vec<f64>, String> {
        // Use PELT algorithm for changepoint detection
        let changepoints = Self::pelt_changepoint_detection(data, changepoint_prior_scale)?;

        // Fit piecewise trend based on growth model
        let trend = match growth_model {
            GrowthModel::Linear => Self::fit_piecewise_linear_trend(time_index, data, &changepoints)?,
            GrowthModel::Logistic { capacity } => Self::fit_piecewise_logistic_trend(time_index, data, &changepoints, *capacity)?,
        };

        Ok(trend)
    }

    /// PELT (Pruned Exact Linear Time) changepoint detection algorithm
    fn pelt_changepoint_detection(data: &[f64], penalty: f64) -> Result<Vec<usize>, String> {
        let n = data.len();
        if n < 10 {
            return Ok(Vec::new()); // Not enough data for meaningful changepoint detection
        }

        // Precompute cumulative sums for efficient cost calculation
        let mut cumsum = vec![0.0; n + 1];
        let mut cumsum_sq = vec![0.0; n + 1];

        for i in 1..=n {
            cumsum[i] = cumsum[i - 1] + data[i - 1];
            cumsum_sq[i] = cumsum_sq[i - 1] + data[i - 1] * data[i - 1];
        }

        // PELT algorithm
        let mut changepoints = Vec::new();
        let mut f = vec![0.0; n + 1]; // Cost function
        let mut cp = vec![0; n + 1]; // Last changepoint

        // Initialize
        f[0] = -penalty;

        for i in 1..=n {
            let mut min_cost = f64::INFINITY;
            let mut best_cp = 0;

            // Try all possible previous changepoints
            #[allow(clippy::needless_range_loop)]
            for j in 0..i {
                let segment_cost = Self::segment_cost(j, i, &cumsum, &cumsum_sq, n);
                let total_cost = f[j] + segment_cost + penalty;

                if total_cost < min_cost {
                    min_cost = total_cost;
                    best_cp = j;
                }
            }

            f[i] = min_cost;
            cp[i] = best_cp;
        }

        // Backtrack to find changepoints
        let mut current = n;
        while current > 0 {
            let prev = cp[current];
            if prev > 0 {
                changepoints.push(prev);
            }
            current = prev;
        }

        changepoints.reverse();
        Ok(changepoints)
    }

    /// Calculate cost of a segment [start, end) using sum of squared errors
    fn segment_cost(start: usize, end: usize, cumsum: &[f64], cumsum_sq: &[f64], _n: usize) -> f64 {
        let n_seg = end - start;
        if n_seg <= 1 {
            return 0.0;
        }

        let sum_y = cumsum[end] - cumsum[start];
        let sum_y_sq = cumsum_sq[end] - cumsum_sq[start];
        let mean = sum_y / n_seg as f64;

        // Sum of squared errors = sum(y^2) - 2*mean*sum(y) + n*mean^2
        sum_y_sq - 2.0 * mean * sum_y + n_seg as f64 * mean * mean
    }

    /// Fit piecewise linear trend
    fn fit_piecewise_linear_trend(time_index: &[f64], data: &[f64], changepoints: &[usize]) -> Result<Vec<f64>, String> {
        let n = data.len();
        let mut trend = vec![0.0; n];

        if changepoints.is_empty() {
            // Simple linear trend
            let (slope, intercept) = Self::linear_regression(time_index, data)?;
            for (i, &t) in time_index.iter().enumerate() {
                trend[i] = intercept + slope * t;
            }
        } else {
            // Piecewise linear trend
            let mut segments = vec![0];
            segments.extend_from_slice(changepoints);
            segments.push(n);

            for window in segments.windows(2) {
                let start_idx = window[0];
                let end_idx = window[1];

                if start_idx >= end_idx {
                    continue;
                }

                let segment_time: Vec<f64> = time_index[start_idx..end_idx].to_vec();
                let segment_data: Vec<f64> = data[start_idx..end_idx].to_vec();

                let (slope, intercept) = Self::linear_regression(&segment_time, &segment_data)?;

                for i in start_idx..end_idx {
                    trend[i] = intercept + slope * time_index[i];
                }
            }
        }

        Ok(trend)
    }

    /// Fit piecewise logistic trend
    fn fit_piecewise_logistic_trend(time_index: &[f64], data: &[f64], changepoints: &[usize], capacity: f64) -> Result<Vec<f64>, String> {
        let n = data.len();
        let mut trend = vec![0.0; n];

        if changepoints.is_empty() {
            // Simple logistic trend
            let params = Self::fit_logistic_growth(time_index, data, capacity)?;
            for (i, &t) in time_index.iter().enumerate() {
                trend[i] = Self::logistic_function(t, params.0, params.1, params.2);
            }
        } else {
            // Piecewise logistic trend
            let mut segments = vec![0];
            segments.extend_from_slice(changepoints);
            segments.push(n);

            for window in segments.windows(2) {
                let start_idx = window[0];
                let end_idx = window[1];

                if start_idx >= end_idx {
                    continue;
                }

                let segment_time: Vec<f64> = time_index[start_idx..end_idx].to_vec();
                let segment_data: Vec<f64> = data[start_idx..end_idx].to_vec();

                let params = Self::fit_logistic_growth(&segment_time, &segment_data, capacity)?;

                for i in start_idx..end_idx {
                    trend[i] = Self::logistic_function(time_index[i], params.0, params.1, params.2);
                }
            }
        }

        Ok(trend)
    }

    /// Fit logistic growth curve: L / (1 + exp(-k(t - t0)))
    fn fit_logistic_growth(time: &[f64], data: &[f64], capacity: f64) -> Result<(f64, f64, f64), String> {
        // Use nonlinear least squares approximation
        // Start with reasonable initial guesses
        let mut k = 0.1; // growth rate
        let mut t0 = time[time.len() / 2]; // inflection point
        let l = capacity; // capacity

        // Simple gradient descent optimization
        let learning_rate = 0.01;
        let n_iterations = 100;

        for _ in 0..n_iterations {
            let mut grad_k = 0.0;
            let mut grad_t0 = 0.0;

            for (&t, &y) in time.iter().zip(data.iter()) {
                let pred = Self::logistic_function(t, l, k, t0);
                let error = pred - y;

                let exp_term = (-k * (t - t0)).exp();
                let denominator = 1.0 + exp_term;
                let d_pred_dk = -l * (t - t0) * exp_term / (denominator * denominator);
                let d_pred_dt0 = l * k * exp_term / (denominator * denominator);

                grad_k += 2.0 * error * d_pred_dk;
                grad_t0 += 2.0 * error * d_pred_dt0;
            }

            k -= learning_rate * grad_k;
            t0 -= learning_rate * grad_t0;

            // Constrain parameters to reasonable ranges
            k = k.clamp(0.001, 10.0);
            t0 = t0.clamp(time[0], time[time.len() - 1]);
        }

        Ok((l, k, t0))
    }

    /// Logistic function: L / (1 + exp(-k(t - t0)))
    fn logistic_function(t: f64, l: f64, k: f64, t0: f64) -> f64 {
        l / (1.0 + (-k * (t - t0)).exp())
    }

    /// Fit holiday effects
    fn fit_holidays(
        _time_index: &[f64],
        data: &[f64],
        trend: &[f64],
        seasonal: &[f64],
        holidays: Option<&[Holiday]>,
    ) -> Result<Vec<f64>, String> {
        let n = data.len();
        let mut holiday_component = vec![0.0; n];

        if let Some(holidays) = holidays {
            // Detrend and deseasonalize the data
            let mut residuals: Vec<f64> = data.iter()
                .zip(trend.iter())
                .zip(seasonal.iter())
                .map(|((y, t), s)| y - t - s)
                .collect();

            // Fit holiday effects using ridge regression
            for holiday in holidays {
                let holiday_effect = Self::fit_holiday_effect(&residuals, &holiday.dates, holiday.prior_scale)?;

                // Add holiday effect to the component
                for &date_idx in &holiday.dates {
                    if date_idx < n {
                        holiday_component[date_idx] += holiday_effect;
                        // Also add effect to nearby days (holiday influence window)
                        let window = 3; // 3 days before and after
                        for offset in 1..=window {
                            if date_idx >= offset {
                                holiday_component[date_idx - offset] += holiday_effect * (1.0 - offset as f64 / (window + 1) as f64);
                            }
                            if date_idx + offset < n {
                                holiday_component[date_idx + offset] += holiday_effect * (1.0 - offset as f64 / (window + 1) as f64);
                            }
                        }
                    }
                }

                // Remove holiday effect from residuals for next holiday
                for &date_idx in &holiday.dates {
                    if date_idx < residuals.len() {
                        residuals[date_idx] -= holiday_effect;
                    }
                }
            }
        }

        Ok(holiday_component)
    }

    /// Fit a single holiday effect
    fn fit_holiday_effect(residuals: &[f64], dates: &[usize], prior_scale: f64) -> Result<f64, String> {
        if dates.is_empty() {
            return Ok(0.0);
        }

        // Simple average of residuals on holiday dates
        let mut sum = 0.0;
        let mut count = 0;

        for &date_idx in dates {
            if date_idx < residuals.len() {
                sum += residuals[date_idx];
                count += 1;
            }
        }

        if count == 0 {
            return Ok(0.0);
        }

        let effect = sum / count as f64;

        // Apply regularization (shrink towards zero)
        let regularized_effect = effect * prior_scale / (prior_scale + (count as f64).sqrt());

        Ok(regularized_effect)
    }

    /// Auto-tune hyperparameters using cross-validation
    fn auto_tune_parameters(data: &[f64], periods: &[usize], growth_model: &GrowthModel) -> Result<(f64, f64), String> {
        let n = data.len();
        if n < 20 {
            return Ok((0.05, 10.0)); // Default values for small datasets
        }

        // Parameter grid for changepoint_prior_scale and seasonality_prior_scale
        let cp_scales = [0.001, 0.01, 0.05, 0.1, 0.5];
        let season_scales = [0.1, 1.0, 10.0, 50.0, 100.0];

        let mut best_cp_scale = 0.05;
        let mut best_season_scale = 10.0;
        let mut best_score = f64::INFINITY;

        // Time series cross-validation
        let n_folds = 3;
        let fold_size = n / n_folds;

        for &cp_scale in &cp_scales {
            for &season_scale in &season_scales {
                let mut cv_scores = Vec::new();

                for fold in 0..n_folds {
                    let test_start = fold * fold_size;
                    let test_end = if fold == n_folds - 1 { n } else { (fold + 1) * fold_size };
                    let train_end = test_start;

                    if train_end < 10 {
                        continue; // Not enough training data
                    }

                    // Train on first part
                    let train_data = &data[..train_end];
                    let forecast_steps = test_end - train_end;

                    let result = Self::fit_prophet_internal(
                        train_data,
                        forecast_steps,
                        periods,
                        cp_scale,
                        season_scale,
                        None, // no holidays for tuning
                        growth_model,
                    );

                    if let Ok(forecast) = result {
                        // Calculate MSE on test set
                        let mut mse = 0.0;
                        let mut count = 0;
                        for i in 0..forecast.len().min(test_end - train_end) {
                            let predicted = forecast[i];
                            let actual = data[train_end + i];
                            mse += (predicted - actual).powi(2);
                            count += 1;
                        }
                        if count > 0 {
                            cv_scores.push(mse / count as f64);
                        }
                    }
                }

                if !cv_scores.is_empty() {
                    let avg_score = cv_scores.iter().sum::<f64>() / cv_scores.len() as f64;
                    if avg_score < best_score {
                        best_score = avg_score;
                        best_cp_scale = cp_scale;
                        best_season_scale = season_scale;
                    }
                }
            }
        }

        Ok((best_cp_scale, best_season_scale))
    }

    /// Internal fit_prophet without auto-tuning (to avoid recursion)
    fn fit_prophet_internal(
        data: &[f64],
        forecast_steps: usize,
        periods: &[usize],
        changepoint_scale: f64,
        seasonality_scale: f64,
        holidays: Option<&[Holiday]>,
        growth_model: &GrowthModel,
    ) -> Result<Vec<f64>, String> {
        if data.len() < 10 {
            return Err("Insufficient data".to_string());
        }

        let time_index: Vec<f64> = (0..data.len()).map(|i| i as f64).collect();

        // Fit components
        let trend_component = Self::fit_trend(&time_index, data, changepoint_scale, growth_model)?;
        let seasonal_component = Self::fit_seasonal(&time_index, data, &trend_component, periods, seasonality_scale)?;
        let holiday_component = Self::fit_holidays(&time_index, data, &trend_component, &seasonal_component, holidays)?;

        // Generate forecasts
        Self::generate_forecasts(
            data.len(),
            forecast_steps,
            &trend_component,
            &seasonal_component,
            &holiday_component,
            periods,
            growth_model,
        )
    }

    fn fit_seasonal(
        _time_index: &[f64],
        data: &[f64],
        trend: &[f64],
        periods: &[usize],
        prior_scale: f64,
    ) -> Result<Vec<f64>, String> {
        let n = data.len();
        let mut seasonal = vec![0.0; n];

        // Detrend the data
        let detrended: Vec<f64> = data.iter().zip(trend.iter())
            .map(|(y, t)| y - t)
            .collect();

        // Fit each seasonal component
        for &period in periods {
            if period >= n {
                continue; // Skip periods longer than the data
            }

            let seasonal_component = Self::fit_fourier_seasonal(&detrended, period, prior_scale)?;
            for i in 0..n {
                seasonal[i] += seasonal_component[i];
            }
        }

        Ok(seasonal)
    }

    /// Fit Fourier series for a single seasonal component
    fn fit_fourier_seasonal(detrended: &[f64], period: usize, prior_scale: f64) -> Result<Vec<f64>, String> {
        let n = detrended.len();

        // Number of Fourier terms (typically 3-10 for Prophet)
        let n_terms = (period as f64).log2().clamp(3.0, 10.0) as usize;

        // Create design matrix for Fourier series
        let mut design_matrix = Vec::new();

        for i in 0..n {
            let t = i as f64 / period as f64; // Normalized time [0, 1)
            let mut row = Vec::new();

            // Add Fourier terms: sin(2πkt/P) and cos(2πkt/P) for k=1 to n_terms
            for k in 1..=n_terms {
                let angle = 2.0 * std::f64::consts::PI * k as f64 * t;
                row.push(angle.sin());
                row.push(angle.cos());
            }

            design_matrix.push(row);
        }

        // Fit using regularized least squares (ridge regression)
        let seasonal_component = Self::ridge_regression(&design_matrix, detrended, prior_scale)?;

        Ok(seasonal_component)
    }

    /// Ridge regression for seasonal fitting
    fn ridge_regression(design_matrix: &[Vec<f64>], y: &[f64], lambda: f64) -> Result<Vec<f64>, String> {
        let n = design_matrix.len();
        let p = design_matrix[0].len();

        if n == 0 || p == 0 {
            return Ok(vec![0.0; n]);
        }

        // Compute X^T X + λI
        let mut xtx = vec![vec![0.0; p]; p];
        for (j, xtx_row) in xtx.iter_mut().enumerate().take(p) {
            for (k, xtx_val) in xtx_row.iter_mut().enumerate().take(p) {
                let mut sum = 0.0;
                for row in design_matrix.iter().take(n) {
                    sum += row[j] * row[k];
                }
                if j == k {
                    sum += lambda; // Ridge penalty
                }
                *xtx_val = sum;
            }
        }

        // Compute X^T y
        let mut xty = vec![0.0; p];
        for (j, xty_val) in xty.iter_mut().enumerate().take(p) {
            for (i, row) in design_matrix.iter().enumerate().take(n) {
                *xty_val += row[j] * y[i];
            }
        }

        // Solve (X^T X + λI) β = X^T y using simple Gaussian elimination
        let beta = Self::solve_linear_system(&xtx, &xty)?;

        // Compute predictions
        let mut predictions = vec![0.0; n];
        for (i, prediction) in predictions.iter_mut().enumerate().take(n) {
            for (j, &beta_val) in beta.iter().enumerate().take(p) {
                *prediction += design_matrix[i][j] * beta_val;
            }
        }

        Ok(predictions)
    }

    /// Generate forecasts by extending trend and seasonal components
    fn generate_forecasts(
        n_observed: usize,
        forecast_steps: usize,
        trend: &[f64],
        seasonal: &[f64],
        _holiday: &[f64],
        periods: &[usize],
        growth_model: &GrowthModel,
    ) -> Result<Vec<f64>, String> {
        let mut forecasts = Vec::with_capacity(forecast_steps);

        // Get last trend value and slope for extrapolation
        let last_trend = *trend.last().ok_or("No trend data")?;
        let trend_slope = if trend.len() >= 2 {
            trend[trend.len() - 1] - trend[trend.len() - 2]
        } else {
            0.0
        };

        // For logistic growth, we need the last fitted parameters
        let logistic_params = if let GrowthModel::Logistic { capacity: _capacity } = growth_model {
            // Refit logistic on last segment to get current parameters
            let recent_data = if trend.len() > 10 { &trend[trend.len() - 10..] } else { trend };
            let time_recent: Vec<f64> = (0..recent_data.len()).map(|i| (n_observed - recent_data.len() + i) as f64).collect();
            Some(Self::fit_logistic_growth(&time_recent, recent_data, *_capacity)?)
        } else {
            None
        };

        for step in 0..forecast_steps {
            let t = (n_observed + step) as f64;

            // Trend component
            let trend_forecast = match growth_model {
                GrowthModel::Linear => last_trend + trend_slope * (step + 1) as f64,
                GrowthModel::Logistic { capacity: _capacity } => {
                    if let Some((l, k, t0)) = logistic_params {
                        Self::logistic_function(t, l, k, t0)
                    } else {
                        last_trend + trend_slope * (step + 1) as f64
                    }
                }
            };

            // Seasonal component (sum of all seasonal patterns)
            let mut seasonal_forecast = 0.0;
            for &period in periods {
                if period > 0 {
                    let phase = (t / period as f64).fract(); // Fractional part for periodic extension
                    let seasonal_idx = ((phase * period as f64) as usize).min(seasonal.len().saturating_sub(1));
                    seasonal_forecast += seasonal.get(seasonal_idx).copied().unwrap_or(0.0);
                }
            }

            // Holiday component (assume no holidays in forecast)
            let holiday_forecast = 0.0;

            forecasts.push(trend_forecast + seasonal_forecast + holiday_forecast);
        }

        Ok(forecasts)
    }

    /// Simple linear regression
    pub fn linear_regression(x: &[f64], y: &[f64]) -> Result<(f64, f64), String> {
        let n = x.len() as f64;
        if n < 2.0 {
            return Ok((0.0, y.iter().sum::<f64>() / n.max(1.0)));
        }

        let sum_x = x.iter().sum::<f64>();
        let sum_y = y.iter().sum::<f64>();
        let sum_xy = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum::<f64>();
        let sum_x2 = x.iter().map(|xi| xi * xi).sum::<f64>();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        Ok((slope, intercept))
    }

    /// Solve linear system using Gaussian elimination (simple implementation)
    fn solve_linear_system(a: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>, String> {
        let n = a.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        // Create augmented matrix [A|b]
        let mut aug = vec![vec![0.0; n + 1]; n];
        for i in 0..n {
            for j in 0..n {
                aug[i][j] = a[i][j];
            }
            aug[i][n] = b[i];
        }

        // Forward elimination
        for i in 0..n {
            // Find pivot
            let mut max_row = i;
            for k in (i + 1)..n {
                if aug[k][i].abs() > aug[max_row][i].abs() {
                    max_row = k;
                }
            }

            // Swap rows
            aug.swap(i, max_row);

            // Check for singular matrix
            if aug[i][i].abs() < 1e-12 {
                return Err("Matrix is singular or nearly singular".to_string());
            }

            // Eliminate
            for k in (i + 1)..n {
                let factor = aug[k][i] / aug[i][i];
                for j in i..=n {
                    aug[k][j] -= factor * aug[i][j];
                }
            }
        }

        // Back substitution
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            x[i] = aug[i][n];
            for j in (i + 1)..n {
                x[i] -= aug[i][j] * x[j];
            }
            x[i] /= aug[i][i];
        }

        Ok(x)
    }
}