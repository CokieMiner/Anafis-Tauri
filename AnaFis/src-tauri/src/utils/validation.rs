// src-tauri/src/utils/validation.rs
use crate::utils::error::AnaFisError;
use blake3;
use std::collections::HashSet;
use validator::Validate;
use serde::{Serialize, Deserialize};

/// Validation for variable names using validator crate
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct VariableInput {
    #[validate(length(min = 1, message = "Variable name cannot be empty"))]
    pub name: String,

    pub value: f64,

    #[validate(range(min = 0.0, message = "Uncertainty cannot be negative"))]
    pub uncertainty: f64,
}

lazy_static::lazy_static! {
    static ref VARIABLE_NAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
}

/// Validate a formula string
pub fn validate_formula(formula: &str) -> Result<(), AnaFisError> {
    let trimmed = formula.trim();
    if trimmed.is_empty() {
        return Err(AnaFisError::Validation("Formula cannot be empty".into()));
    }

    // Additional formula validation can be added here
    // e.g., check for balanced parentheses, valid operators, etc.

    Ok(())
}

/// Validate a variable using the validator crate
pub fn validate_variable(variable: &VariableInput) -> Result<(), AnaFisError> {
    variable.validate()
        .map_err(|e| AnaFisError::Validation(format!("Variable validation failed: {e}")))?;
    Ok(())
}

/// Validate a collection of variables for uniqueness and individual validity
pub fn validate_variables(variables: &[VariableInput]) -> Result<(), AnaFisError> {
    // Check for duplicate names
    let names: HashSet<&str> = variables.iter().map(|v| v.name.as_str()).collect();
    if names.len() != variables.len() {
        return Err(AnaFisError::Validation("Duplicate variable names found".into()));
    }

    // Validate each variable
    for variable in variables {
        validate_variable(variable)?;
    }

    Ok(())
}

/// Generate a fast, secure cache key using BLAKE3 hashing
pub fn generate_cache_key(formula: &str, variables: &[VariableInput]) -> String {
    let mut hasher = blake3::Hasher::new();

    // Add formula to hash
    hasher.update(formula.as_bytes());

    // Sort variables by name for consistent hashing
    let mut sorted_vars: Vec<_> = variables.iter().collect();
    sorted_vars.sort_by(|a, b| a.name.cmp(&b.name));

    // Add each variable to hash
    for var in sorted_vars {
        hasher.update(var.name.as_bytes());
        hasher.update(&var.value.to_le_bytes());
        hasher.update(&var.uncertainty.to_le_bytes());
    }

    // Return hex representation of hash
    hasher.finalize().to_hex().to_string()
}
