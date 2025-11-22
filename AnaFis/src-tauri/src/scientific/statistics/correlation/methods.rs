//! Basic correlation computation methods

use crate::scientific::statistics::descriptive::moments::StatisticalMoments;
use crate::scientific::statistics::descriptive::quantiles::Quantiles;

/// Correlation computation methods
pub struct CorrelationMethods;

impl CorrelationMethods {
    /// Compute Pearson correlation coefficient between two vectors
    /// Compute Pearson correlation coefficient between two vectors.
    /// If errors are provided, computes uncertainty using Monte Carlo simulation.
    pub fn pearson_correlation(x: &[f64], y: &[f64], x_err: Option<&[f64]>, y_err: Option<&[f64]>) -> Result<(f64, f64), String> {
        if x.len() != y.len() || x.len() < 2 {
            return Err("Vectors must have equal length and at least 2 elements".to_string());
        }

        // If errors are provided, use uncertainty propagation
        if x_err.is_some() || y_err.is_some() {
            let zeros = vec![0.0; x.len()];
            let xe = x_err.unwrap_or(&zeros);
            let ye = y_err.unwrap_or(&zeros);
            return Self::uncertainty_correlation(x, xe, y, ye, 1000, "pearson");
        }

        // Use stable variance calculations from StatisticalMoments
        let mean_x = x.mean();
        let mean_y = y.mean();
        let var_x = x.variance();
        let var_y = y.variance();

        if var_x <= 0.0 || var_y <= 0.0 {
            return Err("Cannot compute correlation: zero variance in data".to_string());
        }

        // Compute covariance using stable method
        let covariance: f64 = x.iter().zip(y.iter())
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum::<f64>() / (x.len() - 1) as f64;

        let correlation = covariance / (var_x * var_y).sqrt();
        
        // Clamp to [-1, 1] to handle floating point precision issues
        Ok((correlation.clamp(-1.0, 1.0), 0.0))
    }

    /// Compute Spearman rank correlation coefficient
    /// Compute Spearman rank correlation coefficient.
    /// If errors are provided, computes uncertainty using Monte Carlo simulation.
    pub fn spearman_correlation(x: &[f64], y: &[f64], x_err: Option<&[f64]>, y_err: Option<&[f64]>) -> Result<(f64, f64), String> {
        if x.len() != y.len() || x.len() < 2 {
            return Err("Vectors must have equal length and at least 2 elements".to_string());
        }

        // If errors are provided, use uncertainty propagation
        if x_err.is_some() || y_err.is_some() {
            let zeros = vec![0.0; x.len()];
            let xe = x_err.unwrap_or(&zeros);
            let ye = y_err.unwrap_or(&zeros);
            return Self::uncertainty_correlation(x, xe, y, ye, 1000, "spearman");
        }

        // Convert to ranks using centralized ranking function
        let x_ranks = crate::scientific::statistics::correlation::utils::rank_data(x);
        let y_ranks = crate::scientific::statistics::correlation::utils::rank_data(y);

        Self::pearson_correlation(&x_ranks, &y_ranks, None, None)
    }

    /// Compute Kendall's Tau-b correlation coefficient, an O(N log N) implementation.
    /// This version correctly handles ties in both x and y variables.
    /// Compute Kendall's Tau-b correlation coefficient.
    /// If errors are provided, computes uncertainty using Monte Carlo simulation.
    pub fn kendall_correlation(x: &[f64], y: &[f64], x_err: Option<&[f64]>, y_err: Option<&[f64]>) -> Result<(f64, f64), String> {
        if x.len() != y.len() {
            return Err("Vectors must have equal length".to_string());
        }
        
        // If errors are provided, use uncertainty propagation
        if x_err.is_some() || y_err.is_some() {
            let zeros = vec![0.0; x.len()];
            let xe = x_err.unwrap_or(&zeros);
            let ye = y_err.unwrap_or(&zeros);
            return Self::uncertainty_correlation(x, xe, y, ye, 1000, "kendall");
        }

        let initial_n = x.len();
        if initial_n < 2 {
            return Ok((0.0, 0.0));
        }

        // Filter out NaNs and create pairs
        let mut pairs: Vec<_> = x.iter().zip(y.iter())
            .filter(|(&xi, &yi)| !xi.is_nan() && !yi.is_nan())
            .map(|(&xi, &yi)| (xi, yi))
            .collect();

        let n = pairs.len();
        if n < 2 {
            return Ok((0.0, 0.0));
        }

        // Sort by x, then by y to handle ties consistently
        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then_with(|| a.1.partial_cmp(&b.1).unwrap()));

        // Count ties in x (n1) and ties in both x and y (n3)
        let mut n1 = 0i64;
        let mut n3 = 0i64;
        let mut i = 0;
        while i < n {
            let mut j = i + 1;
            while j < n && pairs[i].0 == pairs[j].0 {
                j += 1;
            }
            let tie_count = j - i;
            if tie_count > 1 {
                n1 += (tie_count * (tie_count - 1) / 2) as i64;

                // Count ties in y within this x-tie group
                let mut k = i;
                while k < j {
                    let mut l = k + 1;
                    while l < j && pairs[k].1 == pairs[l].1 {
                        l += 1;
                    }
                    let sub_tie_count = l - k;
                    if sub_tie_count > 1 {
                        n3 += (sub_tie_count * (sub_tie_count - 1) / 2) as i64;
                    }
                    k = l;
                }
            }
            i = j;
        }

        // Count ties in y (n2)
        // We can do this by sorting just the y values, or by using the pairs if we re-sort them by y
        // Re-sorting pairs by y is needed anyway for the merge sort step if we want to be efficient,
        // but standard merge sort for inversions usually assumes x is sorted.
        // Actually, to count n2 we just need y sorted.
        // To count inversions (swaps), we need y values in the order defined by sorted x.
        // `pairs` is currently sorted by x. So `pairs.map(|p| p.1)` gives us y in x-order.
        
        let mut ys_in_x_order: Vec<f64> = pairs.iter().map(|p| p.1).collect();
        
        // For n2, we need sorted y. We can make a copy to sort.
        let mut ys_sorted = ys_in_x_order.clone();
        ys_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mut n2 = 0i64;
        i = 0;
        while i < n {
            let mut j = i + 1;
            while j < n && ys_sorted[i] == ys_sorted[j] {
                j += 1;
            }
            let tie_count = j - i;
            if tie_count > 1 {
                n2 += (tie_count * (tie_count - 1) / 2) as i64;
            }
            i = j;
        }

        // Count discordant pairs (swaps) using merge sort on ys_in_x_order
        let mut temp = vec![0.0; n];
        let swaps = Self::kendall_merge_sort(&mut ys_in_x_order, &mut temp, 0, n - 1);

        let n0 = n as i64 * (n as i64 - 1) / 2;
        
        // concordant - discordant = n0 - n1 - n2 + n3 - 2 * swaps
        let s = n0 - n1 - n2 + n3 - 2 * swaps;
        
        let denominator = ((n0 - n1) as f64 * (n0 - n2) as f64).sqrt();
        if denominator == 0.0 {
            return Ok((0.0, 0.0));
        }
        
        let tau = s as f64 / denominator;
        
        Ok((tau.clamp(-1.0, 1.0), 0.0))
    }

    /// Helper for kendall_correlation: sorts and counts inversions.
    fn kendall_merge_sort(arr: &mut [f64], temp: &mut [f64], left: usize, right: usize) -> i64 {
        let mut inv_count = 0;
        if left < right {
            let mid = left + (right - left) / 2;
            inv_count += Self::kendall_merge_sort(arr, temp, left, mid);
            inv_count += Self::kendall_merge_sort(arr, temp, mid + 1, right);
            inv_count += Self::kendall_merge(arr, temp, left, mid + 1, right);
        }
        inv_count
    }

    /// Helper for kendall_correlation: merges two subarrays and counts inversions.
    fn kendall_merge(arr: &mut [f64], temp: &mut [f64], left: usize, mid: usize, right: usize) -> i64 {
        let mut i = left;
        let mut j = mid;
        let mut k = left;
        let mut inv_count = 0;

        while i < mid && j <= right {
            if arr[i] <= arr[j] {
                temp[k] = arr[i];
                k += 1;
                i += 1;
            } else {
                temp[k] = arr[j];
                k += 1;
                j += 1;
                inv_count += (mid - i) as i64;
            }
        }

        while i < mid {
            temp[k] = arr[i];
            k += 1;
            i += 1;
        }

        while j <= right {
            temp[k] = arr[j];
            k += 1;
            j += 1;
        }

        arr[left..(right + 1)].copy_from_slice(&temp[left..(right + 1)]);

        inv_count
    }

    /// Compute biweight midcorrelation (robust correlation) with configurable tuning
    pub fn biweight_midcorrelation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        Self::biweight_midcorrelation_tuned(x, y, 9.0) // Default tuning constant
    }

    /// Compute biweight midcorrelation with specified tuning constant
    pub fn biweight_midcorrelation_tuned(x: &[f64], y: &[f64], tuning_constant: f64) -> Result<f64, String> {
        if x.len() != y.len() || x.len() < 2 {
            return Err("Datasets must have equal length and at least 2 observations".to_string());
        }

        // Compute medians
        let median_x = Quantiles::nan_safe_median(x);
        let median_y = Quantiles::nan_safe_median(y);

        // Compute MAD (Median Absolute Deviation)
        let mad_x = Self::median_absolute_deviation(x, median_x);
        let mad_y = Self::median_absolute_deviation(y, median_y);

        if mad_x == 0.0 || mad_y == 0.0 {
            return Ok(0.0);
        }

        // Compute weights and weighted correlation
        let mut weighted_xy = 0.0;
        let mut weighted_x2 = 0.0;
        let mut weighted_y2 = 0.0;

        for (&xi, &yi) in x.iter().zip(y.iter()) {
            let u_x = (xi - median_x) / (tuning_constant * mad_x);
            let u_y = (yi - median_y) / (tuning_constant * mad_y);

            if u_x.abs() < 1.0 && u_y.abs() < 1.0 {
                let w_x = (1.0 - u_x * u_x).powi(2);
                let w_y = (1.0 - u_y * u_y).powi(2);
                let w = w_x * w_y;

                weighted_xy += w * (xi - median_x) * (yi - median_y);
                weighted_x2 += w * (xi - median_x).powi(2);
                weighted_y2 += w * (yi - median_y).powi(2);
            }
        }

        if weighted_x2 == 0.0 || weighted_y2 == 0.0 {
            return Ok(0.0);
        }

        Ok(weighted_xy / (weighted_x2 * weighted_y2).sqrt())
    }



    /// Helper function to compute median absolute deviation
    fn median_absolute_deviation(data: &[f64], median: f64) -> f64 {
        let deviations: Vec<f64> = data.iter()
            .map(|x| (x - median).abs())
            .collect();
        Quantiles::nan_safe_median(&deviations)
    }

    /// Compute percentage bend correlation (robust alternative to Pearson)
    pub fn percentage_bend_correlation(x: &[f64], y: &[f64]) -> Result<f64, String> {
        Self::percentage_bend_correlation_tuned(x, y, 0.2) // Default beta = 0.2
    }

    /// Compute percentage bend correlation with specified beta (robustness parameter)
    pub fn percentage_bend_correlation_tuned(x: &[f64], y: &[f64], beta: f64) -> Result<f64, String> {
        if x.len() != y.len() || x.len() < 2 {
            return Err("Datasets must have equal length and at least 2 observations".to_string());
        }

        if !(0.0..=0.5).contains(&beta) {
            return Err("Beta must be between 0 and 0.5".to_string());
        }

        let n = x.len();
        let n_f = n as f64;

        // Calculate the number of observations to bend (20% by default)
        let bend_count = (beta * n_f).ceil() as usize;

        // Sort data to find bent values
        let mut x_sorted = x.to_vec();
        let mut y_sorted = y.to_vec();
        x_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        y_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Bent values (Winsorized extremes)
        let x_lower = x_sorted[bend_count];
        let x_upper = x_sorted[n - bend_count - 1];
        let y_lower = y_sorted[bend_count];
        let y_upper = y_sorted[n - bend_count - 1];

        // Winsorize the data
        let x_winsorized: Vec<f64> = x.iter().map(|&xi| {
            if xi < x_lower { x_lower }
            else if xi > x_upper { x_upper }
            else { xi }
        }).collect();

        let y_winsorized: Vec<f64> = y.iter().map(|&yi| {
            if yi < y_lower { y_lower }
            else if yi > y_upper { y_upper }
            else { yi }
        }).collect();

        // Compute correlation on winsorized data
        Self::pearson_correlation(&x_winsorized, &y_winsorized, None, None).map(|(r, _)| r)
    }

    /// Compute autocorrelation function for time series data
    pub fn autocorrelation(data: &[f64], max_lag: usize) -> Result<Vec<f64>, String> {
        if data.len() < 2 {
            return Err("Need at least 2 observations for autocorrelation".to_string());
        }

        let n = data.len();
        let max_lag = max_lag.min(n - 1);

        // Compute mean and variance
        let mean = data.mean();
        let variance = data.variance();

        if variance == 0.0 {
            return Err("Cannot compute autocorrelation: zero variance in data".to_string());
        }

        let mut autocorrelations = Vec::with_capacity(max_lag);

        for lag in 1..=max_lag {
            // Standard ACF definition: r_k = \sum (x_t - \bar{x})(x_{t+k} - \bar{x}) / \sum (x_t - \bar{x})^2
            // This ensures |r_k| <= 1 and positive semi-definite autocorrelation matrix.
            let mut numerator = 0.0;
            for i in 0..(n - lag) {
                numerator += (data[i] - mean) * (data[i + lag] - mean);
            }

            // Denominator is the sum of squared deviations: variance * (n - 1)
            let denominator = variance * (n - 1) as f64;

            let autocorr = if denominator > 0.0 {
                numerator / denominator
            } else {
                0.0
            };
            autocorrelations.push(autocorr);
        }

        Ok(autocorrelations)
    }

    /// Compute correlation with uncertainty propagation using Monte Carlo simulation.
    /// 
    /// # Arguments
    /// * `x` - Vector of x values
    /// * `x_err` - Vector of x uncertainties (standard deviations)
    /// * `y` - Vector of y values
    /// * `y_err` - Vector of y uncertainties (standard deviations)
    /// * `n_sims` - Number of Monte Carlo simulations
    /// * `method` - Correlation method ("pearson", "spearman", "kendall")
    /// 
    /// # Returns
    /// * `(mean_correlation, std_dev_correlation)`
    pub fn uncertainty_correlation(
        x: &[f64], 
        x_err: &[f64], 
        y: &[f64], 
        y_err: &[f64], 
        n_sims: usize,
        method: &str
    ) -> Result<(f64, f64), String> {
        if x.len() != y.len() || x.len() != x_err.len() || y.len() != y_err.len() {
            return Err("All input vectors must have equal length".to_string());
        }
        if x.len() < 2 {
            return Err("Need at least 2 observations".to_string());
        }

        use rand_distr::{Normal, Distribution};
        use rayon::prelude::*;

        // Parallel Monte Carlo simulation
        let correlations: Vec<f64> = (0..n_sims).into_par_iter().map(|_| {
            let mut rng = rand::rng();
            
            // Sample new datasets based on errors
            let x_sim: Vec<f64> = x.iter().zip(x_err.iter()).map(|(&val, &err)| {
                if err > 0.0 {
                    let normal = Normal::new(val, err).unwrap();
                    normal.sample(&mut rng)
                } else {
                    val
                }
            }).collect();

            let y_sim: Vec<f64> = y.iter().zip(y_err.iter()).map(|(&val, &err)| {
                if err > 0.0 {
                    let normal = Normal::new(val, err).unwrap();
                    normal.sample(&mut rng)
                } else {
                    val
                }
            }).collect();

            // Compute correlation for this simulation
            match method {
                "pearson" => Self::pearson_correlation(&x_sim, &y_sim, None, None).map(|(r, _)| r).unwrap_or(0.0),
                "spearman" => Self::spearman_correlation(&x_sim, &y_sim, None, None).map(|(r, _)| r).unwrap_or(0.0),
                "kendall" => Self::kendall_correlation(&x_sim, &y_sim, None, None).map(|(r, _)| r).unwrap_or(0.0),
                _ => Self::pearson_correlation(&x_sim, &y_sim, None, None).map(|(r, _)| r).unwrap_or(0.0),
            }
        }).collect();

        // Compute mean and standard deviation of simulated correlations
        let mean_corr = correlations.mean();
        let std_corr = correlations.std_dev();

        Ok((mean_corr, std_corr))
    }
}