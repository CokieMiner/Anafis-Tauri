//! Common utility functions for data validation and preprocessing

/// Validate that all variables in a dataset have the same number of observations
pub fn validate_variable_lengths(data: &[Vec<f64>]) -> Result<usize, String> {
    if data.is_empty() {
        return Err("No variables provided".to_string());
    }

    let n_obs = data[0].len();
    for (i, var) in data.iter().enumerate() {
        if var.len() != n_obs {
            return Err(format!(
                "Variable {} has {} observations, expected {} like the first variable",
                i, var.len(), n_obs
            ));
        }
    }

    Ok(n_obs)
}