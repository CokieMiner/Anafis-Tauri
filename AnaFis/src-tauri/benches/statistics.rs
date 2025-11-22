//! Performance benchmarks for statistical functions
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use rand_pcg::Pcg64;
use anafis_lib::scientific::statistics::correlation::CorrelationHypothesisTestingEngine;
use anafis_lib::scientific::statistics::distributions::normality_tests::NormalityTests;

/// Generate synthetic normal data for benchmarking
fn generate_normal_data(n: usize, mean: f64, std_dev: f64, seed: u64) -> Vec<f64> {
    let mut rng = Pcg64::seed_from_u64(seed);
    let normal = rand_distr::Normal::new(mean, std_dev).unwrap();
    (0..n).map(|_| normal.sample(&mut rng)).collect()
}

/// Generate correlated data for benchmarking
fn generate_correlated_data(n: usize, correlation: f64, seed: u64) -> (Vec<f64>, Vec<f64>) {
    let mut rng = Pcg64::seed_from_u64(seed);
    let normal = rand_distr::Normal::new(0.0, 1.0).unwrap();

    let x: Vec<f64> = (0..n).map(|_| normal.sample(&mut rng)).collect();
    let mut y: Vec<f64> = Vec::with_capacity(n);

    for &xi in &x {
        let noise = normal.sample(&mut rng);
        let yi = correlation * xi + (1.0 - correlation.powi(2)).sqrt() * noise;
        y.push(yi);
    }

    (x, y)
}

fn bench_normality_tests(c: &mut Criterion) {
    let small_data = generate_normal_data(100, 0.0, 1.0, 42);
    let medium_data = generate_normal_data(1000, 0.0, 1.0, 42);
    let large_data = generate_normal_data(5000, 0.0, 1.0, 42);

    c.bench_function("normality_tests_small", |b| {
        b.iter(|| {
            let _result = NormalityTests::comprehensive_normality_tests(black_box(&small_data));
        })
    });

    c.bench_function("normality_tests_medium", |b| {
        b.iter(|| {
            let _result = NormalityTests::comprehensive_normality_tests(black_box(&medium_data));
        })
    });

    c.bench_function("normality_tests_large", |b| {
        b.iter(|| {
            let _result = NormalityTests::comprehensive_normality_tests(black_box(&large_data));
        })
    });
}

fn bench_correlation_analysis(c: &mut Criterion) {
    let (x_small, y_small) = generate_correlated_data(50, 0.5, 42);
    let (x_medium, y_medium) = generate_correlated_data(200, 0.5, 42);
    let (x_large, y_large) = generate_correlated_data(1000, 0.5, 42);

    let engine = CorrelationHypothesisTestingEngine::new();
    let mut rng = Pcg64::seed_from_u64(123);

    c.bench_function("correlation_analysis_small", |b| {
        b.iter(|| {
            let _result = engine.comprehensive_correlation_analysis(
                black_box(&x_small),
                black_box(&y_small),
                0,
                1,
                None,
                Some(1000),
                black_box(&mut rng),
            );
        })
    });

    c.bench_function("correlation_analysis_medium", |b| {
        b.iter(|| {
            let _result = engine.comprehensive_correlation_analysis(
                black_box(&x_medium),
                black_box(&y_medium),
                0,
                1,
                None,
                Some(1000),
                black_box(&mut rng),
            );
        })
    });

    c.bench_function("correlation_analysis_large", |b| {
        b.iter(|| {
            let _result = engine.comprehensive_correlation_analysis(
                black_box(&x_large),
                black_box(&y_large),
                0,
                1,
                None,
                Some(1000),
                black_box(&mut rng),
            );
        })
    });
}

fn bench_streaming_normality(c: &mut Criterion) {
    let data_iter = generate_normal_data(10000, 0.0, 1.0, 42).into_iter();
    let engine = CorrelationHypothesisTestingEngine::new();

    c.bench_function("streaming_normality_large", |b| {
        b.iter(|| {
            let data_iter = generate_normal_data(10000, 0.0, 1.0, 42).into_iter();
            let _result = engine.streaming_normality_tests(black_box(data_iter), Some(1000));
        })
    });
}

fn bench_power_analysis(c: &mut Criterion) {
    let engine = CorrelationHypothesisTestingEngine::new();

    c.bench_function("power_analysis_sample_size", |b| {
        b.iter(|| {
            let _result = engine.correlation_power_analysis(
                black_box(0.3),
                100,
                Some(0.05),
                Some(0.8),
            );
        })
    });

    c.bench_function("power_analysis_current_power", |b| {
        b.iter(|| {
            let _result = engine.correlation_power_analysis(
                black_box(0.3),
                100,
                Some(0.05),
                None,
            );
        })
    });
}

criterion_group!(
    benches,
    bench_normality_tests,
    bench_correlation_analysis,
    bench_streaming_normality,
    bench_power_analysis
);
criterion_main!(benches);