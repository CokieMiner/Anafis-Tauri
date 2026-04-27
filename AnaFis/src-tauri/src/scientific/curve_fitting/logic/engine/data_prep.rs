//! Data preparation and unified covariance construction for ODR.
//!
//! This module transforms raw user input (variables, uncertainties, and correlations)
//! into a unified numerical space suitable for the ODR engine. It handles:
//! - Validation of identifiers and value finiteness.
//! - Sanitization and clamping of uncertainties to avoid singular weights.
//! - Construction of unified point-wise covariance matrices (sigma) from
//!   independent and dependent variable inputs.
//! - Metadata propagation for GUM-compliant degrees-of-freedom tracking.

use super::{
    CORRELATION_TOLERANCE, MIN_VARIANCE, OdrError, OdrFitRequest, OdrResult, PSD_EIGEN_TOLERANCE,
    PointCovariances, PreparedData, UncertaintyType, VariableInput, validate_identifier,
};
use nalgebra::DMatrix;

/// Prepares data for ODR fitting by combining all observed variables into a single unified space.
#[allow(
    clippy::too_many_lines,
    reason = "Validation and metadata propagation are kept in one pass"
)]
pub fn prepare_data(request: &OdrFitRequest) -> OdrResult<PreparedData> {
    if request.layers.is_empty() {
        return Err(OdrError::Validation(
            "At least one model layer is required".to_owned(),
        ));
    }

    if request.dependent_variables.is_empty() {
        return Err(OdrError::Validation(
            "At least one dependent variable observation is required".to_owned(),
        ));
    }

    let point_count = request.dependent_variables[0].values.len();
    if point_count < 2 {
        return Err(OdrError::Validation(
            "At least two observations are required for fitting".to_owned(),
        ));
    }

    let use_poisson = request.use_poisson_weighting.unwrap_or(false);

    let mut variable_names =
        Vec::with_capacity(request.independent_variables.len() + request.dependent_variables.len());
    let mut variable_values =
        Vec::with_capacity(request.independent_variables.len() + request.dependent_variables.len());
    let mut variable_sigmas =
        Vec::with_capacity(request.independent_variables.len() + request.dependent_variables.len());
    let mut had_uncertainty_clamp = false;
    let mut had_low_count_poisson = false;
    let mut inferred_type_a_dof_count = 0_usize;
    let mut variable_uncertainty_dofs: Vec<Option<f64>> =
        Vec::with_capacity(request.independent_variables.len() + request.dependent_variables.len());
    let mut had_zero_count_poisson = false;
    let mut clamped_variance_count = 0_usize;

    let mut process_variable = |var: &VariableInput, is_dependent: bool| -> OdrResult<()> {
        if var.values.len() != point_count {
            return Err(OdrError::Validation(format!(
                "Variable '{}' length mismatch: expected {}, got {}",
                var.name,
                point_count,
                var.values.len()
            )));
        }

        let name = var.name.trim().to_lowercase();
        validate_identifier(&name, "variable")?;

        if variable_names.contains(&name) {
            return Err(OdrError::Validation(format!(
                "Duplicate expected variable name mapping: {name}"
            )));
        }

        variable_names.push(name);
        variable_values.push(sanitize_values(&var.values, &var.name)?);

        let resolved_dof = match (var.uncertainty_type, var.uncertainty_degrees_of_freedom) {
            (_, Some(dof)) if !dof.is_finite() || dof <= 0.0 => {
                return Err(OdrError::Validation(format!(
                    "Uncertainty degrees of freedom for '{}' must be finite and > 0",
                    var.name
                )));
            }
            (_, Some(dof)) => Some(dof),
            (Some(UncertaintyType::TypeA), None) => {
                inferred_type_a_dof_count += 1;
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "point_count converted to f64 for degrees-of-freedom metadata"
                )]
                {
                    Some((point_count.saturating_sub(1)) as f64)
                }
            }
            _ => None,
        };

        if let Some(uncertainties) = &var.uncertainties {
            if uncertainties.len() != point_count {
                return Err(OdrError::Validation(format!(
                    "Uncertainty length mismatch for '{}': expected {}, got {}",
                    var.name,
                    point_count,
                    uncertainties.len()
                )));
            }
            let (sigma, clamped_count) = sanitize_uncertainties(uncertainties, &var.name)?;
            if clamped_count > 0 {
                had_uncertainty_clamp = true;
                clamped_variance_count += clamped_count;
            }
            variable_sigmas.push(sigma);
        } else if is_dependent && use_poisson {
            let mut sigma = Vec::with_capacity(point_count);
            for (idx, val) in var.values.iter().enumerate() {
                if *val < 0.0 {
                    return Err(OdrError::Validation(format!(
                        "Poisson weighting requires non-negative counts for '{}' at index {idx}",
                        var.name
                    )));
                }

                let variance = (*val).max(MIN_VARIANCE);
                if *val < 20.0 {
                    had_low_count_poisson = true;
                }
                if *val <= 0.0 {
                    had_zero_count_poisson = true;
                    had_uncertainty_clamp = true;
                    clamped_variance_count += 1;
                }
                sigma.push(variance.sqrt());
            }
            variable_sigmas.push(sigma);
        } else {
            variable_sigmas.push(vec![0.0; point_count]);
        }
        variable_uncertainty_dofs.push(resolved_dof);

        Ok(())
    };

    for var in &request.independent_variables {
        process_variable(var, false)?;
    }
    for var in &request.dependent_variables {
        process_variable(var, true)?;
    }

    let point_covariances = build_point_covariances(
        point_count,
        &variable_sigmas,
        request.point_correlations.as_deref(),
    )?;

    Ok(PreparedData {
        variable_names,
        variable_values,
        point_covariances,
        point_count,
        had_uncertainty_clamp,
        had_low_count_poisson,
        had_zero_count_poisson,
        clamped_variance_count,
        inferred_type_a_dof_count,
        variable_uncertainty_dofs,
        welch_satterthwaite_dof: None,
    })
}

/// Validates and ensures all values are finite.
pub fn sanitize_values(values: &[f64], label: &str) -> OdrResult<Vec<f64>> {
    let mut sanitized = Vec::with_capacity(values.len());
    for (idx, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OdrError::Validation(format!(
                "Non-finite value in {label} at index {idx}"
            )));
        }
        sanitized.push(*value);
    }
    Ok(sanitized)
}

/// Validates uncertainties and clamps near-zero values.
pub fn sanitize_uncertainties(values: &[f64], label: &str) -> OdrResult<(Vec<f64>, usize)> {
    let mut sanitized = Vec::with_capacity(values.len());
    let mut clamped_count = 0_usize;
    let sigma_min = MIN_VARIANCE.sqrt();

    for (idx, &val) in values.iter().enumerate() {
        if !val.is_finite() {
            return Err(OdrError::Validation(format!(
                "Non-finite uncertainty found in variable '{label}' at index {idx}"
            )));
        }
        if val < 0.0 {
            return Err(OdrError::Validation(format!(
                "Negative uncertainty found in variable '{label}' at index {idx}"
            )));
        }
        if val < sigma_min {
            sanitized.push(sigma_min);
            clamped_count += 1;
        } else {
            sanitized.push(val);
        }
    }

    Ok((sanitized, clamped_count))
}

/// Constructs the full covariance matrix for each measurement point in the unified variable space.
pub fn build_point_covariances(
    point_count: usize,
    variable_sigmas: &[Vec<f64>],
    point_correlations: Option<&[Vec<Vec<f64>>]>,
) -> OdrResult<PointCovariances> {
    let dim = variable_sigmas.len();

    if let Some(correlations) = point_correlations
        && correlations.len() != point_count
    {
        return Err(OdrError::Validation(format!(
            "point_correlations length mismatch: expected {}, got {}",
            point_count,
            correlations.len()
        )));
    }

    if point_correlations.is_none() && point_count > 0 {
        let all_homogeneous = variable_sigmas.iter().all(|sigmas| {
            let first = sigmas[0];
            sigmas.iter().all(|&s| s.to_bits() == first.to_bits())
        });

        if all_homogeneous {
            let mut sigma = vec![vec![0.0; dim]; dim];
            for idx in 0..dim {
                sigma[idx][idx] = variable_sigmas[idx][0] * variable_sigmas[idx][0];
            }
            return Ok(PointCovariances::Shared(sigma));
        }
    }

    let mut covariances = Vec::with_capacity(point_count);

    for point in 0..point_count {
        let mut sigmas = vec![0.0; dim];
        for var_idx in 0..dim {
            sigmas[var_idx] = variable_sigmas[var_idx][point];
        }

        let covariance = if let Some(correlations) = point_correlations {
            let corr = &correlations[point];
            validate_point_correlation_matrix(corr, dim, point)?;

            let mut sigma = vec![vec![0.0; dim]; dim];
            for row in 0..dim {
                for col in 0..dim {
                    sigma[row][col] = corr[row][col] * sigmas[row] * sigmas[col];
                }
            }
            sigma
        } else {
            let mut sigma = vec![vec![0.0; dim]; dim];
            for idx in 0..dim {
                sigma[idx][idx] = sigmas[idx] * sigmas[idx];
            }
            sigma
        };

        covariances.push(covariance);
    }

    Ok(PointCovariances::PerPoint(covariances))
}

/// Validates if a point correlation matrix is symmetric and has unit diagonal.
pub fn validate_point_correlation_matrix(
    matrix: &[Vec<f64>],
    dim: usize,
    point: usize,
) -> OdrResult<()> {
    if matrix.len() != dim {
        return Err(OdrError::Validation(format!(
            "point_correlations[{point}] has invalid shape: expected {dim} rows, got {}",
            matrix.len()
        )));
    }

    for row in matrix {
        if row.len() != dim {
            return Err(OdrError::Validation(format!(
                "point_correlations[{point}] has invalid shape: expected {dim} columns"
            )));
        }
    }

    for (row_idx, row_values) in matrix.iter().enumerate().take(dim) {
        let diagonal = row_values[row_idx];
        if !diagonal.is_finite() {
            return Err(OdrError::Validation(format!(
                "point_correlations[{point}][{row_idx}][{row_idx}] must be finite"
            )));
        }
        if (diagonal - 1.0).abs() > CORRELATION_TOLERANCE {
            return Err(OdrError::Validation(format!(
                "point_correlations[{point}][{row_idx}][{row_idx}] must be 1"
            )));
        }

        for (col_idx, value) in row_values.iter().copied().enumerate().take(dim) {
            if !value.is_finite() {
                return Err(OdrError::Validation(format!(
                    "point_correlations[{point}][{row_idx}][{col_idx}] must be finite"
                )));
            }
            if !(-1.0 - CORRELATION_TOLERANCE..=1.0 + CORRELATION_TOLERANCE).contains(&value) {
                return Err(OdrError::Validation(format!(
                    "point_correlations[{point}][{row_idx}][{col_idx}] must be in [-1, 1]"
                )));
            }

            let symmetric = matrix[col_idx][row_idx];
            if (value - symmetric).abs() > CORRELATION_TOLERANCE {
                return Err(OdrError::Validation(format!(
                    "point_correlations[{point}] must be symmetric"
                )));
            }
        }
    }

    if !is_positive_semidefinite(matrix) {
        return Err(OdrError::Validation(format!(
            "point_correlations[{point}] must be positive semidefinite"
        )));
    }

    Ok(())
}

/// Checks if a matrix is Positive Semi-Definiteness using eigenvalue decomposition.
pub fn is_positive_semidefinite(matrix: &[Vec<f64>]) -> bool {
    let dim = matrix.len();
    if dim == 0 {
        return true;
    }

    if dim == 1 {
        let v = matrix[0][0];
        return v.is_finite() && v >= -PSD_EIGEN_TOLERANCE;
    }

    if dim == 2 {
        let a = matrix[0][0];
        let b = matrix[0][1];
        let c = matrix[1][0];
        let d = matrix[1][1];
        if !(a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()) {
            return false;
        }
        if (b - c).abs() > CORRELATION_TOLERANCE {
            return false;
        }
        let det = a.mul_add(d, -(b * c));
        return a >= -PSD_EIGEN_TOLERANCE
            && d >= -PSD_EIGEN_TOLERANCE
            && det >= -PSD_EIGEN_TOLERANCE;
    }

    if dim == 3 {
        let m00 = matrix[0][0];
        let m01 = matrix[0][1];
        let m02 = matrix[0][2];
        let m11 = matrix[1][1];
        let m12 = matrix[1][2];
        let m22 = matrix[2][2];
        if !(m00.is_finite()
            && m01.is_finite()
            && m02.is_finite()
            && m11.is_finite()
            && m12.is_finite()
            && m22.is_finite())
        {
            return false;
        }
        if (matrix[0][1] - matrix[1][0]).abs() > CORRELATION_TOLERANCE
            || (matrix[0][2] - matrix[2][0]).abs() > CORRELATION_TOLERANCE
            || (matrix[1][2] - matrix[2][1]).abs() > CORRELATION_TOLERANCE
        {
            return false;
        }

        let principal_01 = m00.mul_add(m11, -(m01 * m01));
        let principal_02 = m00.mul_add(m22, -(m02 * m02));
        let principal_12 = m11.mul_add(m22, -(m12 * m12));
        #[allow(
            clippy::suspicious_operation_groupings,
            reason = "Expanded 3x3 symmetric determinant expression"
        )]
        let det3 = m02.mul_add(
            m01.mul_add(m12, -(m02 * m11)),
            m00.mul_add(
                m11.mul_add(m22, -(m12 * m12)),
                -(m01 * m01.mul_add(m22, -(m02 * m12))),
            ),
        );

        return m00 >= -PSD_EIGEN_TOLERANCE
            && m11 >= -PSD_EIGEN_TOLERANCE
            && m22 >= -PSD_EIGEN_TOLERANCE
            && principal_01 >= -PSD_EIGEN_TOLERANCE
            && principal_02 >= -PSD_EIGEN_TOLERANCE
            && principal_12 >= -PSD_EIGEN_TOLERANCE
            && det3 >= -PSD_EIGEN_TOLERANCE;
    }

    let mut flat = Vec::with_capacity(dim * dim);
    for row in matrix {
        flat.extend(row.iter().copied());
    }

    let m = DMatrix::from_row_slice(dim, dim, &flat);
    let eigen = m.symmetric_eigen();
    eigen
        .eigenvalues
        .iter()
        .all(|value| value.is_finite() && *value >= -PSD_EIGEN_TOLERANCE)
}
