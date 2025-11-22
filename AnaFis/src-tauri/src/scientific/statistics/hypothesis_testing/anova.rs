//! ANOVA implementations
//!
//! Analysis of Variance (ANOVA) for comparing means across multiple groups.

use super::types::{StatsError, AnovaResult, TwoWayAnovaResult, NWayAnovaResult, FactorResult, InteractionResult};
use super::helpers::f_p_value;
use super::post_hoc::PostHocTesting;
use crate::scientific::statistics::primitives::linear_algebra::LinearAlgebra;
use ndarray::{Array2, Array1};

/// Helper struct for Type III sums of squares (2-way ANOVA)
#[derive(Debug)]
struct TypeIIIResults {
    ss_total: f64,
    ss_factor1: f64,
    ss_factor2: f64,
    ss_interaction: f64,
    ss_residual: f64,
}

/// Helper struct for N-way ANOVA Type III results
#[derive(Debug)]
struct NWayTypeIIIResults {
    factor_ss: Vec<f64>,        // Sum of squares for each main effect
    interaction_ss: Vec<f64>,   // Sum of squares for each interaction
}

/// ANOVA testing engine
pub struct AnovaTesting;

impl AnovaTesting {

    /// One-way ANOVA
    pub fn one_way_anova(groups: &[&[f64]]) -> Result<AnovaResult, StatsError> {
        if groups.len() < 2 {
            return Err(StatsError::InsufficientData { required: 2, found: groups.len() });
        }

        let mut all_data = Vec::new();
        let mut group_means = Vec::new();
        let mut group_sizes = Vec::new();

        for group in groups {
            if group.is_empty() {
                return Err(StatsError::EmptyData);
            }
            let mean = group.iter().sum::<f64>() / group.len() as f64;
            group_means.push(mean);
            group_sizes.push(group.len());
            all_data.extend_from_slice(group);
        }

        let n_total = all_data.len() as f64;
        let grand_mean = all_data.iter().sum::<f64>() / n_total;

        let ss_between = group_means.iter().zip(&group_sizes)
            .map(|(&mean, &size)| size as f64 * (mean - grand_mean).powi(2))
            .sum::<f64>();

        let ss_within = groups.iter().zip(&group_means)
            .map(|(group, &mean)| group.iter().map(|&x| (x - mean).powi(2)).sum::<f64>())
            .sum::<f64>();

        let df_between = (groups.len() - 1) as f64;
        let df_within = n_total - groups.len() as f64;

        let ms_between = ss_between / df_between;
        let ms_within = ss_within / df_within;
        let f_statistic = if ms_within > 0.0 { ms_between / ms_within } else { f64::INFINITY };
        let p_value = f_p_value(f_statistic, df_between, df_within)?;
        let eta_squared = if ss_between + ss_within > 0.0 { ss_between / (ss_between + ss_within) } else { 0.0 };

        let post_hoc_results = PostHocTesting::bonferroni_post_hoc(groups, &group_means, &group_sizes, ms_within, df_within)?;

        Ok(AnovaResult {
            test_type: "One-way ANOVA".to_string(),
            f_statistic,
            p_value,
            degrees_of_freedom_between: df_between,
            degrees_of_freedom_within: df_within,
            sum_of_squares_between: ss_between,
            sum_of_squares_within: ss_within,
            eta_squared,
            significant: p_value < 0.05,
            post_hoc_results: Some(post_hoc_results),
        })
    }

    /// Two-way ANOVA
    /// Two-way ANOVA using General Linear Model (GLM) approach with Type III sums of squares
    ///
    /// This implementation correctly handles both balanced and unbalanced designs by:
    /// 1. Building a design matrix for the full model: Y = μ + α_i + β_j + (αβ)_ij + ε
    /// 2. Using Type III sums of squares for proper hypothesis testing with Effect Coding
    /// 3. Computing F-statistics and p-values for main effects and interaction
    ///
    /// # Arguments
    /// * `data` - Vector of groups, where each group contains observations for a specific
    ///   combination of factor levels
    /// * `factor1_levels` - Level indices for factor 1 corresponding to each group
    /// * `factor2_levels` - Level indices for factor 2 corresponding to each group
    ///
    /// # Returns
    /// Two-way ANOVA results with F-statistics, p-values, and effect sizes
    pub fn two_way_anova(
        data: &[Vec<f64>],
        factor1_levels: &[usize],
        factor2_levels: &[usize]
    ) -> Result<TwoWayAnovaResult, StatsError> {
        if data.is_empty() {
            return Err(StatsError::EmptyData);
        }

        if data.len() != factor1_levels.len() || data.len() != factor2_levels.len() {
            return Err(StatsError::DimensionMismatch);
        }

        // Convert factor levels to string labels for the design matrix builder
        let factor1_strings: Vec<String> = factor1_levels.iter().map(|&x| format!("F1_{}", x)).collect();
        let factor2_strings: Vec<String> = factor2_levels.iter().map(|&x| format!("F2_{}", x)).collect();

        // Flatten all data and create factor data for design matrix
        let mut y = Vec::new();
        let mut factor1_data = Vec::new();
        let mut factor2_data = Vec::new();

        for (group_idx, group) in data.iter().enumerate() {
            let f1_level = factor1_strings[group_idx].clone();
            let f2_level = factor2_strings[group_idx].clone();

            for &value in group {
                y.push(value);
                factor1_data.push(f1_level.clone());
                factor2_data.push(f2_level.clone());
            }
        }

        let n_obs = y.len();
        if n_obs < 4 { // Need at least 2 levels for each factor with some data
            return Err(StatsError::InsufficientData { required: 4, found: n_obs });
        }

        // Build design matrix for 2x2 ANOVA with Reference coding
        let x = AnovaTesting::build_two_way_design_matrix(&factor1_data, &factor2_data)?;

        // Fit the full model using SVD-based regression
        let beta = AnovaTesting::fit_glm_svd(&x, &y)?;
        let y_hat = LinearAlgebra::matrix_vector_multiply(&x, &beta)?;
        let residuals = Array1::from_vec(y.to_vec()) - y_hat;
        let _ss_residual = residuals.iter().map(|&r| r * r).sum::<f64>();

        // Compute Type III sums of squares using Effect Coding
        let type_iii_ss = AnovaTesting::compute_type_iii_sums_of_squares_effect(&x, &y, &[factor1_data, factor2_data])?;

        // Degrees of freedom
        let n_factor1 = factor1_levels.iter().max().map(|&x| x + 1).unwrap_or(0);
        let n_factor2 = factor2_levels.iter().max().map(|&x| x + 1).unwrap_or(0);
        let df_factor1 = (n_factor1 - 1) as f64;
        let df_factor2 = (n_factor2 - 1) as f64;
        let df_interaction = df_factor1 * df_factor2;
        let df_residual = (n_obs as f64) - 1.0 - df_factor1 - df_factor2 - df_interaction;

        // Mean squares
        let ms_factor1 = type_iii_ss.ss_factor1 / df_factor1;
        let ms_factor2 = type_iii_ss.ss_factor2 / df_factor2;
        let ms_interaction = type_iii_ss.ss_interaction / df_interaction;
        let ms_residual = type_iii_ss.ss_residual / df_residual;

        // F-statistics
        let f_factor1 = if ms_residual > 0.0 { ms_factor1 / ms_residual } else { f64::INFINITY };
        let f_factor2 = if ms_residual > 0.0 { ms_factor2 / ms_residual } else { f64::INFINITY };
        let f_interaction = if ms_residual > 0.0 { ms_interaction / ms_residual } else { f64::INFINITY };

        // P-values
        let p_factor1 = f_p_value(f_factor1, df_factor1, df_residual)?;
        let p_factor2 = f_p_value(f_factor2, df_factor2, df_residual)?;
        let p_interaction = f_p_value(f_interaction, df_interaction, df_residual)?;

        // Effect sizes (partial eta squared)
        let eta_squared_factor1 = if type_iii_ss.ss_total > 0.0 {
            type_iii_ss.ss_factor1 / (type_iii_ss.ss_factor1 + type_iii_ss.ss_residual)
        } else { 0.0 };
        let eta_squared_factor2 = if type_iii_ss.ss_total > 0.0 {
            type_iii_ss.ss_factor2 / (type_iii_ss.ss_factor2 + type_iii_ss.ss_residual)
        } else { 0.0 };
        let eta_squared_interaction = if type_iii_ss.ss_total > 0.0 {
            type_iii_ss.ss_interaction / (type_iii_ss.ss_interaction + type_iii_ss.ss_residual)
        } else { 0.0 };

        Ok(TwoWayAnovaResult {
            f_statistic_factor1: f_factor1,
            f_statistic_factor2: f_factor2,
            f_statistic_interaction: f_interaction,
            p_value_factor1: p_factor1,
            p_value_factor2: p_factor2,
            p_value_interaction: p_interaction,
            degrees_of_freedom_factor1: df_factor1,
            degrees_of_freedom_factor2: df_factor2,
            degrees_of_freedom_interaction: df_interaction,
            degrees_of_freedom_residual: df_residual,
            eta_squared_factor1,
            eta_squared_factor2,
            eta_squared_interaction,
            significant_factor1: p_factor1 < 0.05,
            significant_factor2: p_factor2 < 0.05,
            significant_interaction: p_interaction < 0.05,
        })
    }

    /// N-way factorial ANOVA using General Linear Model (GLM) approach with Type III sums of squares
    ///
    /// This implementation supports arbitrary numbers of factors with proper Type III SS
    /// for hypothesis testing. Uses Effect Coding for balanced interpretation of effects.
    ///
    /// Note: This implementation currently calculates all Main Effects and all Pairwise (2-way)
    /// Interactions. Higher-order interactions (3-way, etc.) are currently treated as part
    /// of the error term to prevent overfitting and maintain interpretability.
    ///
    /// # Arguments
    /// * `data` - Vector of groups, where each group contains observations for a specific
    ///   combination of factor levels
    /// * `factor_data` - Vector of factor level vectors, one for each factor
    /// * `factor_names` - Optional names for the factors (defaults to "Factor 1", "Factor 2", etc.)
    ///
    /// # Returns
    /// N-way ANOVA results with F-statistics, p-values, and effect sizes for all main effects and interactions
    pub fn n_way_anova(
        data: &[Vec<f64>],
        factor_data: &[Vec<String>],
        factor_names: Option<&[String]>,
    ) -> Result<NWayAnovaResult, StatsError> {
        if data.is_empty() {
            return Err(StatsError::EmptyData);
        }

        let n_factors = factor_data.len();
        if n_factors < 2 {
            return Err(StatsError::AnovaError("N-way ANOVA requires at least 2 factors".to_string()));
        }

        if n_factors > 5 {
            return Err(StatsError::AnovaError("N-way ANOVA currently supports up to 5 factors".to_string()));
        }

        // Validate input dimensions
        let n_groups = data.len();
        for (i, factor) in factor_data.iter().enumerate() {
            if factor.len() != n_groups {
                return Err(StatsError::AnovaError(format!("Factor {} has {} levels but there are {} groups", i + 1, factor.len(), n_groups)));
            }
        }

        // Set default factor names if not provided
        let default_names: Vec<String> = (0..n_factors).map(|i| format!("Factor {}", i + 1)).collect();
        let factor_names = factor_names.unwrap_or(&default_names);

        // Flatten all data and create combined factor data for design matrix
        let mut y = Vec::new();
        let mut combined_factors = vec![Vec::new(); n_factors];

        for (group_idx, group) in data.iter().enumerate() {
            for &value in group {
                y.push(value);
                for (factor_idx, factor_vec) in combined_factors.iter_mut().enumerate() {
                    factor_vec.push(factor_data[factor_idx][group_idx].clone());
                }
            }
        }

        let n_obs = y.len();
        if n_obs < 4 { // Need at least some data
            return Err(StatsError::InsufficientData { required: 4, found: n_obs });
        }

        // Build design matrix for N-way ANOVA with Effect Coding
        let x = AnovaTesting::build_n_way_design_matrix(&combined_factors)?;

        // Fit the full model using SVD-based regression
        let beta = AnovaTesting::fit_glm_svd(&x, &y)?;
        let y_hat = LinearAlgebra::matrix_vector_multiply(&x, &beta)?;
        let residuals = Array1::from_vec(y.to_vec()) - y_hat;
        let ss_residual = residuals.iter().map(|&r| r * r).sum::<f64>();

        // Total sum of squares
        let y_mean = y.iter().sum::<f64>() / n_obs as f64;
        let ss_total = y.iter().map(|&yi| (yi - y_mean).powi(2)).sum::<f64>();

        // Compute Type III sums of squares for all main effects and interactions
        let type_iii_ss = AnovaTesting::compute_n_way_type_iii_sums_of_squares(&x, &y, &combined_factors)?;

        // Degrees of freedom
        let df_residual = (n_obs as f64) - 1.0 - (x.ncols() as f64 - 1.0); // -1 for intercept

        // Safety check for rank deficiency
        if df_residual <= 0.0 {
            return Err(StatsError::AnovaError("Degrees of freedom <= 0. Design matrix is likely rank-deficient or N is too small.".to_string()));
        }

        // Build factor results
        let mut factor_results = Vec::new();
        for (i, &ss) in type_iii_ss.factor_ss.iter().enumerate() {
            let n_levels = factor_data[i].iter().collect::<std::collections::HashSet<_>>().len();
            let df = (n_levels - 1) as f64;
            let ms = ss / df;
            let ms_residual = ss_residual / df_residual;
            let f_stat = if ms_residual > 0.0 { ms / ms_residual } else { f64::INFINITY };
            let p_value = f_p_value(f_stat, df, df_residual)?;
            let eta_squared = if ss_total > 0.0 { ss / (ss + ss_residual) } else { 0.0 };

            factor_results.push(FactorResult {
                factor_name: factor_names[i].clone(),
                f_statistic: f_stat,
                p_value,
                degrees_of_freedom: df,
                sum_of_squares: ss,
                eta_squared,
                significant: p_value < 0.05,
            });
        }

        // Build interaction results (currently limited to 2-way interactions)
        let mut interaction_results = Vec::new();
        for (i, &ss) in type_iii_ss.interaction_ss.iter().enumerate() {
            // For now, only handle 2-way interactions
            // TODO: Extend to higher-order interactions if needed
            if n_factors == 2 && i == 0 {
                let df = factor_results[0].degrees_of_freedom * factor_results[1].degrees_of_freedom;
                let ms = ss / df;
                let ms_residual = ss_residual / df_residual;
                let f_stat = if ms_residual > 0.0 { ms / ms_residual } else { f64::INFINITY };
                let p_value = f_p_value(f_stat, df, df_residual)?;
                let eta_squared = if ss_total > 0.0 { ss / (ss + ss_residual) } else { 0.0 };

                interaction_results.push(InteractionResult {
                    interaction_name: format!("{} × {}", factor_names[0], factor_names[1]),
                    factors_involved: vec![factor_names[0].clone(), factor_names[1].clone()],
                    f_statistic: f_stat,
                    p_value,
                    degrees_of_freedom: df,
                    sum_of_squares: ss,
                    eta_squared,
                    significant: p_value < 0.05,
                });
            }
        }

        Ok(NWayAnovaResult {
            test_type: format!("{}-way ANOVA", n_factors),
            factor_results,
            interaction_results,
            degrees_of_freedom_residual: df_residual,
            total_sum_of_squares: ss_total,
            residual_sum_of_squares: ss_residual,
        })
    }

    /// Fit a General Linear Model using SVD-based least squares for numerical stability
    fn fit_glm_svd(x: &Array2<f64>, y: &[f64]) -> Result<Array1<f64>, StatsError> {
        let y_array = Array1::from_vec(y.to_vec());
        
        // Use SVD-based least squares instead of normal equations for better numerical stability
        LinearAlgebra::least_squares(x, &y_array)
            .map_err(StatsError::LinearAlgebraError)
    }

    /// Compute Type III sums of squares for factorial ANOVA using Effect Coding
    ///
    /// Type III SS tests each effect while controlling for all other effects.
    /// With Effect Coding, this is done by comparing the full model to reduced models
    /// that drop one effect at a time.
    fn compute_type_iii_sums_of_squares_effect(
        x_full: &Array2<f64>,
        y: &[f64],
        factor_data: &[Vec<String>],
    ) -> Result<TypeIIIResults, StatsError> {
        let n_factors = factor_data.len();
        if n_factors != 2 {
            return Err(StatsError::AnovaError("Type III SS computation currently supports exactly 2 factors".to_string()));
        }

        // Fit full model
        let beta_full = AnovaTesting::fit_glm_svd(x_full, y)?;
        let y_hat_full = LinearAlgebra::matrix_vector_multiply(x_full, &beta_full)
            .map_err(StatsError::LinearAlgebraError)?;
        let residuals_full = Array1::from_vec(y.to_vec()) - y_hat_full;
        let ss_residual_full = residuals_full.iter().map(|&r| r * r).sum::<f64>();

        // Total sum of squares
        let y_mean = y.iter().sum::<f64>() / y.len() as f64;
        let ss_total = y.iter().map(|&yi| (yi - y_mean).powi(2)).sum::<f64>();

        // Factor 1 SS: Compare full model to model without factor 1
        let x_no_f1 = AnovaTesting::build_single_factor_design_matrix(&factor_data[1])?;
        let beta_no_f1 = AnovaTesting::fit_glm_svd(&x_no_f1, y)?;
        let y_hat_no_f1 = LinearAlgebra::matrix_vector_multiply(&x_no_f1, &beta_no_f1)
            .map_err(StatsError::LinearAlgebraError)?;
        let residuals_no_f1 = Array1::from_vec(y.to_vec()) - y_hat_no_f1;
        let ss_residual_no_f1 = residuals_no_f1.iter().map(|&r| r * r).sum::<f64>();
        let ss_factor1 = ss_residual_no_f1 - ss_residual_full;

        // Factor 2 SS: Compare full model to model without factor 2
        let x_no_f2 = AnovaTesting::build_single_factor_design_matrix(&factor_data[0])?;
        let beta_no_f2 = AnovaTesting::fit_glm_svd(&x_no_f2, y)?;
        let y_hat_no_f2 = LinearAlgebra::matrix_vector_multiply(&x_no_f2, &beta_no_f2)
            .map_err(StatsError::LinearAlgebraError)?;
        let residuals_no_f2 = Array1::from_vec(y.to_vec()) - y_hat_no_f2;
        let ss_residual_no_f2 = residuals_no_f2.iter().map(|&r| r * r).sum::<f64>();
        let ss_factor2 = ss_residual_no_f2 - ss_residual_full;

        // Interaction SS: Compare full model to additive model (no interaction)
        let x_additive = AnovaTesting::build_two_factor_additive_design_matrix(&factor_data[0], &factor_data[1])?;
        let beta_additive = AnovaTesting::fit_glm_svd(&x_additive, y)?;
        let y_hat_additive = LinearAlgebra::matrix_vector_multiply(&x_additive, &beta_additive)
            .map_err(StatsError::LinearAlgebraError)?;
        let residuals_additive = Array1::from_vec(y.to_vec()) - y_hat_additive;
        let ss_residual_additive = residuals_additive.iter().map(|&r| r * r).sum::<f64>();
        let ss_interaction = ss_residual_additive - ss_residual_full;

        Ok(TypeIIIResults {
            ss_total,
            ss_factor1: ss_factor1.max(0.0), // Ensure non-negative
            ss_factor2: ss_factor2.max(0.0), // Ensure non-negative
            ss_interaction: ss_interaction.max(0.0), // Ensure non-negative
            ss_residual: ss_residual_full,
        })
    }
    ///
    /// Type III SS tests each effect while controlling for all other effects.
    /// This is done by comparing the full model to reduced models that drop one effect at a time.
    fn compute_n_way_type_iii_sums_of_squares(
        x_full: &Array2<f64>,
        y: &[f64],
        factor_data: &[Vec<String>],
    ) -> Result<NWayTypeIIIResults, StatsError> {
        let n_factors = factor_data.len();

        // Fit full model
        let beta_full = AnovaTesting::fit_glm_svd(x_full, y)?;
        let y_hat_full = LinearAlgebra::matrix_vector_multiply(x_full, &beta_full)
            .map_err(StatsError::LinearAlgebraError)?;
        let residuals_full = Array1::from_vec(y.to_vec()) - y_hat_full;
        let ss_residual_full = residuals_full.iter().map(|&r| r * r).sum::<f64>();

        // Total sum of squares
        let y_mean = y.iter().sum::<f64>() / y.len() as f64;
        let _ss_total = y.iter().map(|&yi| (yi - y_mean).powi(2)).sum::<f64>();

        // Compute SS for each main effect by dropping one factor at a time
        let mut factor_ss = Vec::new();
        for exclude_idx in 0..n_factors {
            let x_reduced = AnovaTesting::build_design_matrix_without_factor(x_full, factor_data, exclude_idx)?;
            let beta_reduced = AnovaTesting::fit_glm_svd(&x_reduced, y)?;
            let y_hat_reduced = LinearAlgebra::matrix_vector_multiply(&x_reduced, &beta_reduced)
                .map_err(StatsError::LinearAlgebraError)?;
            let residuals_reduced = Array1::from_vec(y.to_vec()) - y_hat_reduced;
            let ss_residual_reduced = residuals_reduced.iter().map(|&r| r * r).sum::<f64>();
            let ss_effect = ss_residual_reduced - ss_residual_full;
            factor_ss.push(ss_effect.max(0.0)); // Ensure non-negative
        }

        // For now, only compute 2-way interactions
        let mut interaction_ss = Vec::new();
        if n_factors >= 2 {
            for i in 0..n_factors {
                for _j in (i + 1)..n_factors {
                    // Build additive model without this interaction
                    let x_additive = AnovaTesting::build_additive_design_matrix(factor_data)?;
                    let beta_additive = AnovaTesting::fit_glm_svd(&x_additive, y)?;
                    let y_hat_additive = LinearAlgebra::matrix_vector_multiply(&x_additive, &beta_additive)
                        .map_err(StatsError::LinearAlgebraError)?;
                    let residuals_additive = Array1::from_vec(y.to_vec()) - y_hat_additive;
                    let ss_residual_additive = residuals_additive.iter().map(|&r| r * r).sum::<f64>();
                    let ss_interaction = ss_residual_additive - ss_residual_full;
                    interaction_ss.push(ss_interaction.max(0.0)); // Ensure non-negative
                }
            }
        }

        Ok(NWayTypeIIIResults {
            factor_ss,
            interaction_ss,
        })
    }

    /// Build design matrix for 2x2 ANOVA with Reference coding
    fn build_two_way_design_matrix(factor1_data: &[String], factor2_data: &[String]) -> Result<Array2<f64>, StatsError> {
        let n_obs = factor1_data.len();
        if n_obs != factor2_data.len() {
            return Err(StatsError::DimensionMismatch);
        }

        // Get unique levels
        let f1_levels: std::collections::HashSet<_> = factor1_data.iter().cloned().collect();
        let f2_levels: std::collections::HashSet<_> = factor2_data.iter().cloned().collect();
        
        if f1_levels.len() != 2 || f2_levels.len() != 2 {
            return Err(StatsError::AnovaError("Currently only supports 2x2 designs".to_string()));
        }

        let mut f1_levels_vec: Vec<_> = f1_levels.into_iter().collect();
        let mut f2_levels_vec: Vec<_> = f2_levels.into_iter().collect();
        f1_levels_vec.sort();
        f2_levels_vec.sort();

        // Reference coding: first level is reference (0), second level is 1
        let mut design_matrix = Array2::zeros((n_obs, 4)); // intercept + f1 + f2 + interaction

        for i in 0..n_obs {
            // Intercept
            design_matrix[[i, 0]] = 1.0;
            
            // Factor 1
            if factor1_data[i] == f1_levels_vec[1] {
                design_matrix[[i, 1]] = 1.0;
            }
            
            // Factor 2
            if factor2_data[i] == f2_levels_vec[1] {
                design_matrix[[i, 2]] = 1.0;
            }
            
            // Interaction
            design_matrix[[i, 3]] = design_matrix[[i, 1]] * design_matrix[[i, 2]];
        }

        Ok(design_matrix)
    }

    /// Build design matrix for a single factor with Reference Coding (intercept + factor)
    fn build_single_factor_design_matrix(factor_data: &[String]) -> Result<Array2<f64>, StatsError> {
        let n_obs = factor_data.len();
        
        // Get unique levels and sort them
        let mut levels: Vec<String> = factor_data.iter().cloned().collect::<std::collections::HashSet<_>>()
            .into_iter().collect();
        levels.sort();
        
        let n_levels = levels.len();
        if n_levels < 2 {
            return Err(StatsError::AnovaError("Factor must have at least 2 levels".to_string()));
        }
        
        // Create level-to-index mapping
        let level_to_idx: std::collections::HashMap<_, _> = levels.iter()
            .enumerate()
            .map(|(i, level)| (level.clone(), i))
            .collect();
        
        // Design matrix: intercept + (n_levels - 1) factor columns
        let n_cols = 1 + (n_levels - 1);
        let mut design_matrix = Array2::zeros((n_obs, n_cols));
        
        // Intercept column (all 1s)
        for i in 0..n_obs {
            design_matrix[[i, 0]] = 1.0;
        }
        
        // Reference coding for factor
        for (row, level) in factor_data.iter().enumerate() {
            if let Some(&level_idx) = level_to_idx.get(level) {
                // Reference level (index 0) gets 0 in all columns
                if level_idx > 0 {
                    design_matrix[[row, level_idx]] = 1.0;
                }
            } else {
                return Err(StatsError::AnovaError(format!("Unknown factor level: {}", level)));
            }
        }
        
        Ok(design_matrix)
    }

    /// Build design matrix for two factors with Effect Coding, no interaction (intercept + factor1 + factor2)
    fn build_two_factor_additive_design_matrix(
        factor1_data: &[String], 
        factor2_data: &[String]
    ) -> Result<Array2<f64>, StatsError> {
        let n_obs = factor1_data.len();
        if n_obs != factor2_data.len() {
            return Err(StatsError::DimensionMismatch);
        }
        
        // Build factor1 matrix
        let f1_matrix = AnovaTesting::build_single_factor_design_matrix(factor1_data)?;
        let f1_cols = f1_matrix.ncols() - 1; // Exclude intercept
        
        // Build factor2 matrix  
        let f2_matrix = AnovaTesting::build_single_factor_design_matrix(factor2_data)?;
        let f2_cols = f2_matrix.ncols() - 1; // Exclude intercept
        
        // Combine: intercept from f1 + factor1 columns + factor2 columns
        let n_cols = 1 + f1_cols + f2_cols;
        let mut design_matrix = Array2::zeros((n_obs, n_cols));
        
        // Intercept (from f1_matrix)
        design_matrix.column_mut(0).assign(&f1_matrix.column(0));
        
        // Factor1 columns
        for col in 0..f1_cols {
            design_matrix.column_mut(1 + col).assign(&f1_matrix.column(1 + col));
        }
        
        // Factor2 columns
        for col in 0..f2_cols {
            design_matrix.column_mut(1 + f1_cols + col).assign(&f2_matrix.column(1 + col));
        }
        
        Ok(design_matrix)
    }

    /// Concatenate matrices horizontally
    fn concatenate_matrices_horizontally(matrices: &[Array2<f64>]) -> Result<Array2<f64>, StatsError> {
        if matrices.is_empty() {
            return Err(StatsError::InvalidParameter("No matrices to concatenate".to_string()));
        }

        let n_rows = matrices[0].nrows();
        for matrix in matrices {
            if matrix.nrows() != n_rows {
                return Err(StatsError::DimensionMismatch);
            }
        }

        let total_cols: usize = matrices.iter().map(|m| m.ncols()).sum();
        let mut result = Array2::zeros((n_rows, total_cols));

        let mut col_offset = 0;
        for matrix in matrices {
            let cols = matrix.ncols();
            result.slice_mut(ndarray::s![.., col_offset..col_offset + cols])
                .assign(matrix);
            col_offset += cols;
        }

        Ok(result)
    }

    /// Build design matrix for N-way ANOVA with Effect Coding
    fn build_n_way_design_matrix(factor_data: &[Vec<String>]) -> Result<Array2<f64>, StatsError> {
        let n_obs = factor_data[0].len();
        let n_factors = factor_data.len();

        // Start with intercept column
        let mut design_matrix = Array2::ones((n_obs, 1));

        // Add main effects for each factor
        for factor in factor_data {
            let factor_matrix = AnovaTesting::encode_categorical_factor_effect(factor)?;
            design_matrix = AnovaTesting::concatenate_matrices_horizontally(&[design_matrix, factor_matrix])?;
        }

        // Add interactions (for now, only 2-way interactions)
        if n_factors >= 2 {
            for i in 0..n_factors {
                for j in (i + 1)..n_factors {
                    let interaction_matrix = AnovaTesting::build_interaction_matrix(&factor_data[i], &factor_data[j])?;
                    design_matrix = AnovaTesting::concatenate_matrices_horizontally(&[design_matrix, interaction_matrix])?;
                }
            }
        }

        Ok(design_matrix)
    }

    /// Encode a categorical factor using Effect Coding (-1, 0, 1)
    fn encode_categorical_factor_effect(factor: &[String]) -> Result<Array2<f64>, StatsError> {
        if factor.is_empty() {
            return Err(StatsError::EmptyData);
        }

        // Get unique levels and sort them for consistent ordering
        let mut levels: Vec<String> = factor.iter().cloned().collect::<std::collections::HashSet<_>>()
            .into_iter().collect();
        levels.sort();

        let n_samples = factor.len();
        let n_levels = levels.len();

        if n_levels < 2 {
            return Err(StatsError::AnovaError("Factor must have at least 2 levels".to_string()));
        }

        // Effect coding: n_levels - 1 columns
        let n_cols = n_levels - 1;
        let mut matrix = Array2::zeros((n_samples, n_cols));

        // Create level-to-index mapping
        let level_to_idx: std::collections::HashMap<_, _> = levels.iter()
            .enumerate()
            .map(|(i, level)| (level.clone(), i))
            .collect();

        for (row, level) in factor.iter().enumerate() {
            if let Some(&level_idx) = level_to_idx.get(level) {
                if level_idx == 0 {
                    // Reference level: -1 in all columns
                    for col in 0..n_cols {
                        matrix[[row, col]] = -1.0;
                    }
                } else {
                    // Non-reference level: +1 in its column, 0 elsewhere
                    matrix[[row, level_idx - 1]] = 1.0;
                }
            } else {
                return Err(StatsError::AnovaError(format!("Unknown factor level: {}", level)));
            }
        }

        Ok(matrix)
    }

    /// Build interaction matrix for two factors
    fn build_interaction_matrix(factor1: &[String], factor2: &[String]) -> Result<Array2<f64>, StatsError> {
        if factor1.len() != factor2.len() {
            return Err(StatsError::DimensionMismatch);
        }

        let f1_matrix = AnovaTesting::encode_categorical_factor_effect(factor1)?;
        let f2_matrix = AnovaTesting::encode_categorical_factor_effect(factor2)?;

        let n_rows = f1_matrix.nrows();
        let n_cols = f1_matrix.ncols() * f2_matrix.ncols();
        let mut interaction_matrix = Array2::zeros((n_rows, n_cols));

        let mut col_idx = 0;
        for f1_col in 0..f1_matrix.ncols() {
            for f2_col in 0..f2_matrix.ncols() {
                for row in 0..n_rows {
                    interaction_matrix[[row, col_idx]] = f1_matrix[[row, f1_col]] * f2_matrix[[row, f2_col]];
                }
                col_idx += 1;
            }
        }

        Ok(interaction_matrix)
    }

    /// Build design matrix without a specific factor (for Type III SS computation)
    fn build_design_matrix_without_factor(
        _x_full: &Array2<f64>,
        factor_data: &[Vec<String>],
        exclude_factor_idx: usize,
    ) -> Result<Array2<f64>, StatsError> {
        let n_obs = factor_data[0].len();
        let n_factors = factor_data.len();

        // Start with intercept column
        let mut design_matrix = Array2::ones((n_obs, 1));

        // Add main effects for all factors except the excluded one
        for (i, factor) in factor_data.iter().enumerate() {
            if i != exclude_factor_idx {
                let factor_matrix = AnovaTesting::encode_categorical_factor_effect(factor)?;
                design_matrix = AnovaTesting::concatenate_matrices_horizontally(&[design_matrix, factor_matrix])?;
            }
        }

        // Add interactions, excluding those involving the excluded factor
        if n_factors >= 2 {
            for i in 0..n_factors {
                for j in (i + 1)..n_factors {
                    if i != exclude_factor_idx && j != exclude_factor_idx {
                        let interaction_matrix = AnovaTesting::build_interaction_matrix(&factor_data[i], &factor_data[j])?;
                        design_matrix = AnovaTesting::concatenate_matrices_horizontally(&[design_matrix, interaction_matrix])?;
                    }
                }
            }
        }

        Ok(design_matrix)
    }

    /// Build additive design matrix (main effects only, no interactions)
    fn build_additive_design_matrix(factor_data: &[Vec<String>]) -> Result<Array2<f64>, StatsError> {
        let n_obs = factor_data[0].len();

        // Start with intercept column
        let mut design_matrix = Array2::ones((n_obs, 1));

        // Add main effects for each factor
        for factor in factor_data {
            let factor_matrix = AnovaTesting::encode_categorical_factor_effect(factor)?;
            design_matrix = AnovaTesting::concatenate_matrices_horizontally(&[design_matrix, factor_matrix])?;
        }

        Ok(design_matrix)
    }
}
