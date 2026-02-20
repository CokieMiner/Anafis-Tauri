#![cfg(test)]
use crate::scientific::curve_fitting::commands::{evaluate_model_grid, fit_custom_odr};
use crate::scientific::curve_fitting::types::{
    GridEvaluationRequest, IndependentVariableInput, OdrFitRequest,
};

fn repeat_corr(point_count: usize, matrix: &[Vec<f64>]) -> Vec<Vec<Vec<f64>>> {
    (0..point_count).map(|_| matrix.to_vec()).collect()
}

#[test]
fn test_fit_custom_odr_linear_model_no_correlation() {
    let x: Vec<f64> = (0..50).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|&xi| xi.mul_add(2.5, -4.0)).collect();

    let request = OdrFitRequest {
        model_formula: "a*x + b".to_string(),
        dependent_variable: "y".to_string(),
        independent_variables: vec![IndependentVariableInput {
            name: "x".to_string(),
            values: x,
            uncertainties: Some(vec![0.1; 50]),
        }],
        observed_values: y,
        observed_uncertainties: Some(vec![0.2; 50]),
        parameter_names: vec!["a".to_string(), "b".to_string()],
        initial_guess: Some(vec![1.0, 0.0]),
        max_iterations: Some(120),
        point_correlations: None,
    };

    let result = fit_custom_odr(request).unwrap();
    assert!(result.success);
    assert!((result.parameter_values[0] - 2.5).abs() < 1e-8);
    assert!((result.parameter_values[1] + 4.0).abs() < 1e-8);
    assert!(result.r_squared > 0.999_999_999);
}

#[test]
fn test_fit_custom_odr_with_independent_correlations() {
    let mut x1 = Vec::new();
    let mut x2 = Vec::new();
    let mut y = Vec::new();

    for i in 0..40 {
        let a = f64::from(i) * 0.25;
        let b = (f64::from(i) * 0.2).sin();
        x1.push(a);
        x2.push(b);
        y.push(1.2f64.mul_add(a, -(0.8 * b)) + 3.0);
    }

    let corr = vec![
        vec![1.0, 0.35, 0.0],
        vec![0.35, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
    ];

    let request = OdrFitRequest {
        model_formula: "p*x1 + q*x2 + r".to_string(),
        dependent_variable: "y".to_string(),
        independent_variables: vec![
            IndependentVariableInput {
                name: "x1".to_string(),
                values: x1,
                uncertainties: Some(vec![0.05; 40]),
            },
            IndependentVariableInput {
                name: "x2".to_string(),
                values: x2,
                uncertainties: Some(vec![0.04; 40]),
            },
        ],
        observed_values: y,
        observed_uncertainties: Some(vec![0.08; 40]),
        parameter_names: vec!["p".to_string(), "q".to_string(), "r".to_string()],
        initial_guess: Some(vec![0.0, 0.0, 0.0]),
        max_iterations: Some(200),
        point_correlations: Some(repeat_corr(40, &corr)),
    };

    let result = fit_custom_odr(request).unwrap();
    assert!(result.success);
    assert!((result.parameter_values[0] - 1.2).abs() < 1e-6);
    assert!((result.parameter_values[1] + 0.8).abs() < 1e-6);
    assert!((result.parameter_values[2] - 3.0).abs() < 1e-6);
}

#[test]
fn test_fit_custom_odr_with_cross_xy_correlation() {
    let x: Vec<f64> = (0..30).map(|i| f64::from(i) * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&xi| xi.mul_add(2.0, 1.0)).collect();

    let corr = vec![vec![1.0, 0.7], vec![0.7, 1.0]];

    let request = OdrFitRequest {
        model_formula: "a*x + b".to_string(),
        dependent_variable: "y".to_string(),
        independent_variables: vec![IndependentVariableInput {
            name: "x".to_string(),
            values: x,
            uncertainties: Some(vec![0.03; 30]),
        }],
        observed_values: y,
        observed_uncertainties: Some(vec![0.05; 30]),
        parameter_names: vec!["a".to_string(), "b".to_string()],
        initial_guess: Some(vec![1.0, 0.0]),
        max_iterations: Some(160),
        point_correlations: Some(repeat_corr(30, &corr)),
    };

    let result = fit_custom_odr(request).unwrap();
    assert!(result.success);
    assert!(result.chi_squared.is_finite());
    assert!((result.parameter_values[0] - 2.0).abs() < 1e-6);
    assert!((result.parameter_values[1] - 1.0).abs() < 1e-6);
}

#[test]
fn test_fit_custom_odr_zero_uncertainty_clamp() {
    let x: Vec<f64> = (0..25).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|&xi| xi.mul_add(-1.5, 6.0)).collect();

    let request = OdrFitRequest {
        model_formula: "m*x + c".to_string(),
        dependent_variable: "y".to_string(),
        independent_variables: vec![IndependentVariableInput {
            name: "x".to_string(),
            values: x,
            uncertainties: Some(vec![0.0; 25]),
        }],
        observed_values: y,
        observed_uncertainties: Some(vec![0.0; 25]),
        parameter_names: vec!["m".to_string(), "c".to_string()],
        initial_guess: Some(vec![0.0, 0.0]),
        max_iterations: Some(200),
        point_correlations: None,
    };

    let result = fit_custom_odr(request).unwrap();
    assert!(result.success);
    assert!(result
        .message
        .unwrap_or_default()
        .to_lowercase()
        .contains("clamped"));
}

#[test]
fn test_fit_custom_odr_invalid_correlation_shape() {
    let x: Vec<f64> = (0..10).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|&xi| xi.mul_add(3.0, 2.0)).collect();

    // dim should be 2 (x,y), but here it's 1x1
    let bad_corr = vec![vec![vec![1.0]]; 10];

    let request = OdrFitRequest {
        model_formula: "a*x + b".to_string(),
        dependent_variable: "y".to_string(),
        independent_variables: vec![IndependentVariableInput {
            name: "x".to_string(),
            values: x,
            uncertainties: Some(vec![0.1; 10]),
        }],
        observed_values: y,
        observed_uncertainties: Some(vec![0.1; 10]),
        parameter_names: vec!["a".to_string(), "b".to_string()],
        initial_guess: Some(vec![1.0, 0.0]),
        max_iterations: Some(100),
        point_correlations: Some(bad_corr),
    };

    let err = fit_custom_odr(request).unwrap_err();
    assert!(err.contains("invalid shape"));
}

#[test]
fn test_fit_custom_odr_nonlinear_gaussian_like() {
    let x: Vec<f64> = (-40..=40).map(|i| f64::from(i) * 0.05).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|&xi| 2.0f64.mul_add((-0.7 * xi * xi).exp(), 0.5))
        .collect();

    let request = OdrFitRequest {
        model_formula: "a*exp(-b*x^2)+c".to_string(),
        dependent_variable: "y".to_string(),
        independent_variables: vec![IndependentVariableInput {
            name: "x".to_string(),
            values: x,
            uncertainties: Some(vec![0.02; 81]),
        }],
        observed_values: y,
        observed_uncertainties: Some(vec![0.03; 81]),
        parameter_names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        initial_guess: Some(vec![1.0, 0.2, 0.0]),
        max_iterations: Some(600),
        point_correlations: None,
    };

    let result = fit_custom_odr(request).unwrap();
    assert!(result.success);
    assert!((result.parameter_values[0] - 2.0).abs() < 1e-3);
    assert!((result.parameter_values[1] - 0.7).abs() < 1e-3);
    assert!((result.parameter_values[2] - 0.5).abs() < 1e-3);
}

#[test]
fn test_fit_custom_odr_multivariable_full_covariance() {
    let mut x1 = Vec::new();
    let mut x2 = Vec::new();
    let mut x3 = Vec::new();
    let mut y = Vec::new();

    for i in 0..35 {
        let a = f64::from(i) * 0.3;
        let b = (f64::from(i) * 0.17).cos();
        let c = (f64::from(i) * 0.11).sin();
        x1.push(a);
        x2.push(b);
        x3.push(c);
        y.push(0.7f64.mul_add(c, 0.9f64.mul_add(a, -(1.1 * b))) + 4.0);
    }

    // Order: [x1, x2, x3, y]
    let corr = vec![
        vec![1.0, 0.2, -0.1, 0.15],
        vec![0.2, 1.0, 0.3, -0.2],
        vec![-0.1, 0.3, 1.0, 0.1],
        vec![0.15, -0.2, 0.1, 1.0],
    ];

    let request = OdrFitRequest {
        model_formula: "p*x1 + q*x2 + r*x3 + s".to_string(),
        dependent_variable: "y".to_string(),
        independent_variables: vec![
            IndependentVariableInput {
                name: "x1".to_string(),
                values: x1,
                uncertainties: Some(vec![0.05; 35]),
            },
            IndependentVariableInput {
                name: "x2".to_string(),
                values: x2,
                uncertainties: Some(vec![0.04; 35]),
            },
            IndependentVariableInput {
                name: "x3".to_string(),
                values: x3,
                uncertainties: Some(vec![0.03; 35]),
            },
        ],
        observed_values: y,
        observed_uncertainties: Some(vec![0.06; 35]),
        parameter_names: vec![
            "p".to_string(),
            "q".to_string(),
            "r".to_string(),
            "s".to_string(),
        ],
        initial_guess: Some(vec![0.0, 0.0, 0.0, 0.0]),
        max_iterations: Some(300),
        point_correlations: Some(repeat_corr(35, &corr)),
    };

    let result = fit_custom_odr(request).unwrap();
    assert!(result.success);
    assert!((result.parameter_values[0] - 0.9).abs() < 1e-6);
    assert!((result.parameter_values[1] + 1.1).abs() < 1e-6);
    assert!((result.parameter_values[2] - 0.7).abs() < 1e-6);
    assert!((result.parameter_values[3] - 4.0).abs() < 1e-6);
}

#[test]
fn test_fit_custom_odr_rejects_non_psd_correlation_matrix() {
    let x1: Vec<f64> = (0..12).map(|i| f64::from(i) * 0.2).collect();
    let x2: Vec<f64> = (0..12).map(|i| (f64::from(i) * 0.3).sin()).collect();
    let y: Vec<f64> = x1
        .iter()
        .zip(x2.iter())
        .map(|(&a, &b)| 1.5f64.mul_add(a, -(0.4 * b)) + 2.0)
        .collect();

    // Symmetric with unit diagonal but not PSD:
    // det = 1 + 2*r12*r13*r23 - r12^2 - r13^2 - r23^2 < 0 for this choice.
    let non_psd_corr = vec![
        vec![1.0, 0.9, 0.9],
        vec![0.9, 1.0, -0.9],
        vec![0.9, -0.9, 1.0],
    ];

    let request = OdrFitRequest {
        model_formula: "a*x1 + b*x2 + c".to_string(),
        dependent_variable: "y".to_string(),
        independent_variables: vec![
            IndependentVariableInput {
                name: "x1".to_string(),
                values: x1,
                uncertainties: Some(vec![0.05; 12]),
            },
            IndependentVariableInput {
                name: "x2".to_string(),
                values: x2,
                uncertainties: Some(vec![0.05; 12]),
            },
        ],
        observed_values: y,
        observed_uncertainties: Some(vec![0.05; 12]),
        parameter_names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        initial_guess: Some(vec![1.0, 1.0, 1.0]),
        max_iterations: Some(120),
        point_correlations: Some(repeat_corr(12, &non_psd_corr)),
    };

    let err = fit_custom_odr(request).unwrap_err();
    assert!(err.to_lowercase().contains("positive semidefinite"));
}

#[test]
fn test_evaluate_model_grid_rejects_too_high_resolution() {
    let request = GridEvaluationRequest {
        model_formula: "a*x + b*y + c".to_string(),
        independent_names: vec!["x".to_string(), "y".to_string()],
        parameter_names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        parameter_values: vec![1.0, 2.0, 3.0],
        x_range: (0.0, 1.0),
        y_range: (0.0, 1.0),
        resolution: 2001,
    };

    let err = evaluate_model_grid(request).unwrap_err();
    assert!(err.to_lowercase().contains("resolution too high"));
}
