//! Random number generation and sampling methods
//!
//! This module provides high-quality random number generation
//! and various sampling techniques for statistical applications.

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

/// High-quality random number generation
pub struct RandomSampling;

impl RandomSampling {
    /// Create a seeded random number generator for reproducibility
    pub fn create_rng(seed: u64) -> Pcg64 {
        Pcg64::seed_from_u64(seed)
    }

    /// Generate uniform random sample [0,1)
    pub fn uniform_sample(rng: &mut Pcg64) -> f64 {
        rng.random::<f64>()
    }

    /// Generate uniform random sample in range [a,b)
    pub fn uniform_range_sample(rng: &mut Pcg64, a: f64, b: f64) -> f64 {
        a + (b - a) * rng.random::<f64>()
    }

    /// Generate normal random sample
    pub fn normal_sample(rng: &mut Pcg64, mean: f64, std: f64) -> f64 {
        use rand_distr::{Normal, Distribution};
        let normal = Normal::new(mean, std).unwrap();
        normal.sample(rng)
    }

    /// Shuffle a vector in place
    pub fn shuffle<T>(rng: &mut Pcg64, data: &mut [T]) {
        use rand::seq::SliceRandom;
        data.shuffle(rng);
    }

    /// Sample with replacement (bootstrap sampling)
    pub fn sample_with_replacement<T: Clone>(rng: &mut Pcg64, data: &[T], size: usize) -> Vec<T> {
        use rand_distr::{Uniform, Distribution};

        let dist = Uniform::new(0, data.len()).unwrap();
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            let idx = dist.sample(rng);
            result.push(data[idx].clone());
        }
        result
    }

    /// Sample without replacement
    pub fn sample_without_replacement<T: Clone>(rng: &mut Pcg64, data: &[T], size: usize) -> Vec<T> {
        let mut indices: Vec<usize> = (0..data.len()).collect();
        Self::shuffle(rng, &mut indices);
        indices.truncate(size);
        indices.into_iter().map(|i| data[i].clone()).collect()
    }

    /// Weighted sampling with replacement
    pub fn weighted_sample_with_replacement<T: Clone>(
        rng: &mut Pcg64,
        data: &[T],
        weights: &[f64],
        size: usize,
    ) -> Vec<T> {
        use rand_distr::weighted::WeightedIndex;
        use rand_distr::Distribution;

        // Create WeightedIndex for efficient sampling
        let dist = WeightedIndex::new(weights)
            .expect("Invalid weights for weighted sampling");

        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            let idx = dist.sample(rng);
            result.push(data[idx].clone());
        }
        result
    }
}