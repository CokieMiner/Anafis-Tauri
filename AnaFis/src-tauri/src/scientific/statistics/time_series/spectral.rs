//! Spectral analysis module for time series
//!
//! This module provides frequency domain analysis including Fast Fourier Transform
//! and periodogram-based spectral density estimation.

use std::f64::consts::PI;
use rustfft::{FftPlanner, num_complex::Complex};
use crate::scientific::statistics::descriptive::StatisticalMoments;
use ndarray::Array2;

/// Spectral analysis results
#[derive(Debug, Clone)]
pub struct SpectralAnalysis {
    /// Frequencies (in cycles per unit time)
    pub frequencies: Vec<f64>,
    /// Power spectral density
    pub power_spectrum: Vec<f64>,
    /// Periodogram values
    pub periodogram: Vec<f64>,
    /// Dominant frequency
    pub dominant_frequency: f64,
    /// Spectral entropy (measure of spectral complexity)
    pub spectral_entropy: f64,
}

/// Spectral analysis engine
pub struct SpectralEngine;

impl SpectralEngine {
    /// Compute periodogram using FFT
    ///
    /// The periodogram provides an estimate of the spectral density of a time series.
    /// It uses the Fast Fourier Transform for efficient computation.
    pub fn periodogram(data: &[f64]) -> Result<SpectralAnalysis, String> {
        if data.len() < 2 {
            return Err("Periodogram requires at least 2 data points".to_string());
        }

        // Remove mean to focus on fluctuations
        let mean = data.mean();
        let detrended: Vec<f64> = data.iter().map(|&x| x - mean).collect();

        // Apply window function (Hanning window) to reduce spectral leakage
        let windowed = Self::apply_hanning_window(&detrended);

        // Perform FFT
        let fft_result = Self::fft(&windowed)?;

        // Compute periodogram (squared magnitude of FFT, normalized)
        let n = data.len() as f64;
        let periodogram: Vec<f64> = fft_result.iter()
            .take(data.len() / 2 + 1) // Only positive frequencies
            .map(|c| c.norm_sqr() / n)
            .collect();

        // Generate frequency vector
        let frequencies: Vec<f64> = (0..periodogram.len())
            .map(|i| i as f64 / n)
            .collect();

        // Power spectrum (smoothed periodogram)
        let power_spectrum = Self::smooth_periodogram(&periodogram, 5)?;

        // Find dominant frequency
        let dominant_frequency = Self::find_dominant_frequency(&frequencies, &power_spectrum);

        // Calculate spectral entropy
        let spectral_entropy = Self::calculate_spectral_entropy(&power_spectrum);

        Ok(SpectralAnalysis {
            frequencies,
            power_spectrum,
            periodogram,
            dominant_frequency,
            spectral_entropy,
        })
    }

    /// Apply Hanning window to reduce spectral leakage
    fn apply_hanning_window(data: &[f64]) -> Vec<f64> {
        let n = data.len();
        let mut windowed = Vec::with_capacity(n);

        for (i, &item) in data.iter().enumerate() {
            let window_value = 0.5 * (1.0 - (2.0 * PI * i as f64 / (n - 1) as f64).cos());
            windowed.push(item * window_value);
        }

        windowed
    }

    /// Perform Fast Fourier Transform
    fn fft(data: &[f64]) -> Result<Vec<Complex<f64>>, String> {
        let mut planner = FftPlanner::<f64>::new();
        let fft = planner.plan_fft_forward(data.len());

        let mut buffer: Vec<Complex<f64>> = data.iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        fft.process(&mut buffer);
        Ok(buffer)
    }

    /// Smooth periodogram using moving average
    fn smooth_periodogram(periodogram: &[f64], window_size: usize) -> Result<Vec<f64>, String> {
        if window_size == 0 || window_size > periodogram.len() {
            return Err("Invalid window size for smoothing".to_string());
        }

        let mut smoothed = Vec::with_capacity(periodogram.len());

        for i in 0..periodogram.len() {
            let start = i.saturating_sub(window_size / 2);
            let end = (i + window_size / 2 + 1).min(periodogram.len());

            let sum: f64 = periodogram[start..end].iter().sum();
            let count = end - start;
            smoothed.push(sum / count as f64);
        }

        Ok(smoothed)
    }

    /// Find the dominant frequency in the power spectrum
    fn find_dominant_frequency(frequencies: &[f64], power_spectrum: &[f64]) -> f64 {
        let mut max_power = 0.0;
        let mut dominant_freq = 0.0;

        // Skip DC component (frequency 0)
        for (i, &power) in power_spectrum.iter().enumerate().skip(1) {
            if power > max_power {
                max_power = power;
                dominant_freq = frequencies[i];
            }
        }

        dominant_freq
    }

    /// Calculate spectral entropy as a measure of spectral complexity
    fn calculate_spectral_entropy(power_spectrum: &[f64]) -> f64 {
        let total_power: f64 = power_spectrum.iter().sum();

        if total_power <= 0.0 {
            return 0.0;
        }

        let normalized: Vec<f64> = power_spectrum.iter()
            .map(|&p| p / total_power)
            .collect();

        let entropy: f64 = normalized.iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| -p * p.log2())
            .sum();

        entropy
    }

    /// Detect periodic components in the time series
    pub fn detect_periodic_components(data: &[f64], threshold: f64) -> Result<Vec<PeriodicComponent>, String> {
        let spectral = Self::periodogram(data)?;

        let mut components = Vec::new();
        let mean_power = spectral.power_spectrum.iter().sum::<f64>() / spectral.power_spectrum.len() as f64;

        for (i, &power) in spectral.power_spectrum.iter().enumerate().skip(1) {
            if power > threshold * mean_power {
                let frequency = spectral.frequencies[i];
                let period = if frequency > 0.0 { 1.0 / frequency } else { f64::INFINITY };
                let strength = power / spectral.power_spectrum[0].max(1.0); // Relative to DC component

                components.push(PeriodicComponent {
                    frequency,
                    period,
                    strength,
                    power,
                });
            }
        }

        // Sort by strength (descending)
        components.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());

        Ok(components)
    }

    /// Generate Fourier basis matrix for seasonality modeling
    /// Returns a matrix with sine and cosine terms for the specified harmonics
    ///
    /// # Arguments
    /// * `times` - Time values (can be indices or actual timestamps)
    /// * `period` - Period of the seasonality
    /// * `n_harmonics` - Number of harmonics to include
    ///
    /// # Returns
    /// Matrix of shape (n_times, 2 * n_harmonics) containing [sin(kt), cos(kt), sin(2kt), cos(2kt), ...]
    pub fn generate_fourier_basis(times: &[f64], period: f64, n_harmonics: usize) -> Result<Array2<f64>, String> {
        if times.is_empty() {
            return Err("Empty times array".to_string());
        }
        if period <= 0.0 {
            return Err("Period must be positive".to_string());
        }
        if n_harmonics == 0 {
            return Err("Must have at least 1 harmonic".to_string());
        }

        let n_times = times.len();
        let n_features = 2 * n_harmonics; // sin and cos for each harmonic
        let mut fourier_matrix = Array2::zeros((n_times, n_features));

        for (i, &t) in times.iter().enumerate() {
            let t_norm = 2.0 * PI * t / period;

            for k in 1..=n_harmonics {
                let k = k as f64;
                let sin_idx = 2 * (k as usize - 1);
                let cos_idx = 2 * (k as usize - 1) + 1;

                fourier_matrix[[i, sin_idx]] = (k * t_norm).sin();
                fourier_matrix[[i, cos_idx]] = (k * t_norm).cos();
            }
        }

        Ok(fourier_matrix)
    }
}

/// Detected periodic component
#[derive(Debug, Clone)]
pub struct PeriodicComponent {
    /// Frequency in cycles per unit time
    pub frequency: f64,
    /// Period (1/frequency)
    pub period: f64,
    /// Relative strength compared to DC component
    pub strength: f64,
    /// Raw power at this frequency
    pub power: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_periodogram_basic() {
        let data = vec![1.0, 2.0, 3.0, 2.0, 1.0, 2.0, 3.0, 2.0];
        let result = SpectralEngine::periodogram(&data).unwrap();

        assert_eq!(result.frequencies.len(), result.periodogram.len());
        assert!(result.dominant_frequency >= 0.0);
        assert!(result.spectral_entropy >= 0.0);
    }

    #[test]
    fn test_periodogram_sine_wave() {
        // Generate a sine wave with known frequency
        let n = 64;
        let frequency = 0.1; // 10 cycles in 64 points = frequency of 0.15625
        let data: Vec<f64> = (0..n).map(|i| (2.0 * PI * frequency * i as f64).sin()).collect();

        let result = SpectralEngine::periodogram(&data).unwrap();

        // The dominant frequency should be close to our input frequency
        assert!((result.dominant_frequency - frequency).abs() < 0.05);
    }

    #[test]
    fn test_detect_periodic_components() {
        let data = vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]; // Frequency = 0.25
        let components = SpectralEngine::detect_periodic_components(&data, 1.5).unwrap();

        assert!(!components.is_empty());
        assert!(components[0].frequency > 0.0);
    }
}