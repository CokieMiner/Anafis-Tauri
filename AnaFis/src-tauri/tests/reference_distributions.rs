//! Reference tests for distribution functions
//! Validates our implementations against SciPy reference values

use serde::{Deserialize, Serialize};
use std::fs::File;
use anafis_lib::scientific::statistics::distributions::distribution_functions;

#[derive(Debug, Deserialize, Serialize)]
struct DistributionCDFTest {
    distribution: String,
    params: Vec<f64>,
    x: f64,
    cdf: f64,
    #[serde(default)]
    pdf: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct DistributionQuantileTest {
    distribution: String,
    params: Vec<f64>,
    p: f64,
    quantile: f64,
}

fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance || (a - b).abs() / b.abs().max(1.0) < tolerance
}

#[test]
fn test_normal_cdf_reference() {
    let file = File::open("tests/data/distribution_tests.json")
        .expect("Failed to open distribution_tests.json");
    let tests: Vec<serde_json::Value> = serde_json::from_reader(file)
        .expect("Failed to parse JSON");

    let mut passed = 0;
    let mut failed = 0;

    for test_value in tests {
        if let Ok(test) = serde_json::from_value::<DistributionCDFTest>(test_value.clone()) {
            if test.distribution == "normal" && test.params.len() == 2 {
                let mean = test.params[0];
                let std = test.params[1];
                
                let computed_cdf = distribution_functions::normal_cdf(test.x, mean, std);
                
                if approx_eq(computed_cdf, test.cdf, 1e-10) {
                    passed += 1;
                } else {
                    failed += 1;
                    eprintln!(
                        "Normal CDF mismatch: x={}, mean={}, std={}, computed={}, expected={}, diff={}",
                        test.x, mean, std, computed_cdf, test.cdf, (computed_cdf - test.cdf).abs()
                    );
                }
            }
        }
    }

    println!("Normal CDF tests: {} passed, {} failed", passed, failed);
    assert_eq!(failed, 0, "Some normal CDF tests failed");
}

#[test]
fn test_normal_quantile_reference() {
    let file = File::open("tests/data/distribution_tests.json")
        .expect("Failed to open distribution_tests.json");
    let tests: Vec<serde_json::Value> = serde_json::from_reader(file)
        .expect("Failed to parse JSON");

    let mut passed = 0;
    let mut failed = 0;

    for test_value in tests {
        if let Ok(test) = serde_json::from_value::<DistributionQuantileTest>(test_value.clone()) {
            if test.distribution == "normal" && test.params.len() == 2 {
                let mean = test.params[0];
                let std = test.params[1];
                
                // For standard normal, we can test directly
                if mean == 0.0 && std == 1.0 {
                    let computed_quantile = distribution_functions::normal_quantile(test.p);
                    
                    if approx_eq(computed_quantile, test.quantile, 1e-8) {
                        passed += 1;
                    } else {
                        failed += 1;
                        eprintln!(
                            "Normal quantile mismatch: p={}, computed={}, expected={}, diff={}",
                            test.p, computed_quantile, test.quantile, (computed_quantile - test.quantile).abs()
                        );
                    }
                }
            }
        }
    }

    println!("Normal quantile tests: {} passed, {} failed", passed, failed);
    assert_eq!(failed, 0, "Some normal quantile tests failed");
}

#[test]
fn test_t_distribution_reference() {
    let file = File::open("tests/data/distribution_tests.json")
        .expect("Failed to open distribution_tests.json");
    let tests: Vec<serde_json::Value> = serde_json::from_reader(file)
        .expect("Failed to parse JSON");

    let mut passed = 0;
    let mut failed = 0;

    for test_value in tests {
        if let Ok(test) = serde_json::from_value::<DistributionCDFTest>(test_value.clone()) {
            if test.distribution == "t" && test.params.len() == 1 {
                let df = test.params[0];
                
                let computed_cdf = distribution_functions::student_t_cdf(test.x, 0.0, 1.0, df);
                
                if approx_eq(computed_cdf, test.cdf, 1e-8) {
                    passed += 1;
                } else {
                    failed += 1;
                    eprintln!(
                        "t-distribution CDF mismatch: x={}, df={}, computed={}, expected={}, diff={}",
                        test.x, df, computed_cdf, test.cdf, (computed_cdf - test.cdf).abs()
                    );
                }
            }
        }
    }

    println!("t-distribution CDF tests: {} passed, {} failed", passed, failed);
    assert_eq!(failed, 0, "Some t-distribution CDF tests failed");
}

#[test]
fn test_chi_square_distribution_reference() {
    let file = File::open("tests/data/distribution_tests.json")
        .expect("Failed to open distribution_tests.json");
    let tests: Vec<serde_json::Value> = serde_json::from_reader(file)
        .expect("Failed to parse JSON");

    let mut passed = 0;
    let mut failed = 0;

    for test_value in tests {
        if let Ok(test) = serde_json::from_value::<DistributionCDFTest>(test_value.clone()) {
            if test.distribution == "chi_square" && test.params.len() == 1 {
                let df = test.params[0];
                
                let computed_cdf = distribution_functions::chi_squared_cdf(test.x, df);
                
                if approx_eq(computed_cdf, test.cdf, 1e-8) {
                    passed += 1;
                } else {
                    failed += 1;
                    eprintln!(
                        "Chi-square CDF mismatch: x={}, df={}, computed={}, expected={}, diff={}",
                        test.x, df, computed_cdf, test.cdf, (computed_cdf - test.cdf).abs()
                    );
                }
            }
        }
    }

    println!("Chi-square CDF tests: {} passed, {} failed", passed, failed);
    assert_eq!(failed, 0, "Some chi-square CDF tests failed");
}
