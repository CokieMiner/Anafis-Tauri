use std::sync::Arc;

use super::engine::CompiledModel;

pub fn flatten_residuals_and_fitted(
    layer_residuals: &[Vec<f64>],
    layer_fitted_values: &[Vec<f64>],
    total_residuals: usize,
) -> (Vec<f64>, Vec<f64>) {
    let mut flat_residuals = Vec::with_capacity(total_residuals);
    let mut flat_fitted = Vec::with_capacity(total_residuals);

    for residuals in layer_residuals {
        flat_residuals.extend_from_slice(residuals);
    }
    for fitted in layer_fitted_values {
        flat_fitted.extend_from_slice(fitted);
    }

    (flat_residuals, flat_fitted)
}

pub fn compute_rmse(residuals: &[f64], rmse_points: usize) -> f64 {
    let residual_sum_of_squares: f64 = residuals.iter().map(|value| value * value).sum();
    #[allow(
        clippy::cast_precision_loss,
        reason = "Residual count casting to f64 for RMSE calculation"
    )]
    {
        (residual_sum_of_squares / rmse_points as f64).sqrt()
    }
}

pub fn compute_global_r_squared(
    models: &[Arc<CompiledModel>],
    variable_names: &[String],
    variable_values: &[Vec<f64>],
    residuals: &[f64],
    total_residuals: usize,
) -> f64 {
    let mut flat_targets = Vec::with_capacity(total_residuals);
    for model in models {
        if let Some(dep_idx) = variable_names
            .iter()
            .position(|name| name == &model.dependent_name)
        {
            flat_targets.extend_from_slice(&variable_values[dep_idx]);
        }
    }

    if flat_targets.is_empty() {
        return 1.0;
    }

    #[allow(
        clippy::cast_precision_loss,
        reason = "Point count casting to f64 for mean calculation"
    )]
    let mean_y = flat_targets.iter().sum::<f64>() / flat_targets.len() as f64;
    let total_sum_of_squares: f64 = flat_targets.iter().map(|value| (value - mean_y).powi(2)).sum();
    let residual_sum_of_squares: f64 = residuals.iter().map(|value| value * value).sum();

    if total_sum_of_squares > 0.0 {
        1.0 - residual_sum_of_squares / total_sum_of_squares
    } else {
        1.0
    }
}

pub fn compute_per_layer_r_squared(
    models: &[Arc<CompiledModel>],
    variable_names: &[String],
    variable_values: &[Vec<f64>],
    layer_residuals: &[Vec<f64>],
) -> Vec<f64> {
    (0..models.len())
        .map(|layer_idx| {
            let model = &models[layer_idx];
            let dep_idx = variable_names.iter().position(|name| name == &model.dependent_name);
            dep_idx.map_or(f64::NAN, |dep_idx| {
                let targets = &variable_values[dep_idx];
                let residuals = &layer_residuals[layer_idx];
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "Point count casting to f64 for mean calculation"
                )]
                let mean_y = targets.iter().sum::<f64>() / targets.len() as f64;
                let ss_tot: f64 = targets.iter().map(|v| (v - mean_y).powi(2)).sum();
                let ss_res: f64 = residuals.iter().map(|v| v.powi(2)).sum();
                if ss_tot > 0.0 {
                    1.0 - ss_res / ss_tot
                } else {
                    1.0
                }
            })
        })
        .collect()
}
