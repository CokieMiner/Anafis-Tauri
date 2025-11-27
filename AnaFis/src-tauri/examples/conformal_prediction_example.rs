//! Example demonstrating conformal prediction for distribution-free uncertainty quantification
//!
//! This example shows how to use conformal prediction to create prediction intervals
//! with guaranteed coverage probabilities without making distributional assumptions.

use anafis_lib::scientific::statistics::uncertainty::conformal::{
    ConformalConfig, ConformalPredictor,
};

fn main() -> Result<(), String> {
    println!("=== Conformal Prediction Example ===\n");

    // Simulated calibration data (e.g., historical measurements)
    let cal_data = vec![
        1.2, 2.3, 2.9, 4.1, 5.0, 5.8, 7.2, 8.1, 8.9, 10.3, 11.5, 12.1, 13.4, 14.2, 15.1, 16.0,
        17.3, 18.5, 19.2, 20.1,
    ];

    // Model predictions on calibration set (e.g., from a regression model)
    let cal_predictions = vec![
        1.0, 2.5, 3.0, 4.0, 5.2, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0, 19.0, 20.0,
    ];

    // Create a conformal predictor with 90% coverage guarantee
    println!("Creating conformal predictor with 90% coverage (alpha = 0.1)");
    let config = ConformalConfig::new(0.1);
    let mut predictor = ConformalPredictor::new(config)?;

    // Calibrate the predictor
    println!("Calibrating with {} calibration points", cal_data.len());
    predictor.calibrate(&cal_data, &cal_predictions)?;

    println!("\n--- Calibration Complete ---");
    println!("Nonconformity scores: {:?}", predictor.calibration_scores());

    // Test predictions
    let test_predictions = vec![5.5, 12.5, 18.5];

    println!("\n--- Making Predictions with Guaranteed Coverage ---\n");
    for pred in test_predictions {
        let interval = predictor.predict_interval_detailed(pred)?;
        println!("Prediction: {:.2}", pred);
        println!("  Interval: [{:.2}, {:.2}]", interval.lower, interval.upper);
        println!("  Width: {:.2}", interval.upper - interval.lower);
        println!("  Coverage guarantee: {:.1}%", interval.coverage * 100.0);
        println!();
    }

    // Validate coverage on test set
    println!("--- Coverage Validation ---\n");

    let test_data = vec![4.8, 5.2, 11.9, 12.7, 18.2, 19.1];
    let test_preds = vec![5.0, 5.0, 12.0, 12.0, 18.0, 18.0];

    let coverage = predictor.empirical_coverage(&test_data, &test_preds)?;
    println!("Test set size: {}", test_data.len());
    println!("Empirical coverage: {:.1}%", coverage * 100.0);
    println!("Theoretical guarantee: ≥ 90%");

    if coverage >= 0.9 {
        println!("✓ Coverage guarantee satisfied!");
    } else {
        println!("⚠ Coverage below guarantee (finite sample effect)");
    }

    Ok(())
}
