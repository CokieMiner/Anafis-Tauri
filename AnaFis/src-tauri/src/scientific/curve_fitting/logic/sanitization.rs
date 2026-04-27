use std::collections::HashSet;

use super::{OdrError, OdrResult};

/// Validates if an identifier is a valid symbol name.
///
/// # Errors
/// Returns `OdrError::Validation` if the identifier is empty or contains invalid characters.
pub fn validate_identifier(identifier: &str, label: &str) -> OdrResult<()> {
    if identifier.trim().is_empty() {
        return Err(OdrError::Validation(format!(
            "{label} names cannot be empty or only whitespace"
        )));
    }

    let mut chars = identifier.chars();
    let first = chars
        .next()
        .ok_or_else(|| OdrError::Validation(format!("{label} names cannot be empty")))?;
    if !(first.is_alphabetic() || first == '_') {
        return Err(OdrError::Validation(format!(
            "Invalid {label} '{identifier}': first character must be a letter or '_'"
        )));
    }

    if !chars.all(|c| c.is_alphanumeric() || c == '_') {
        return Err(OdrError::Validation(format!(
            "Invalid {label} '{identifier}': use only letters, digits, and '_'"
        )));
    }

    Ok(())
}

/// Normalizes a list of identifiers by trimming and converting to lowercase.
///
/// # Errors
/// Returns `OdrError::Validation` if any identifier is invalid or duplicate.
pub fn normalize_identifiers(raw: &[String], label: &str) -> OdrResult<Vec<String>> {
    if raw.is_empty() {
        return Err(OdrError::Validation(format!(
            "At least one {label} is required"
        )));
    }

    let mut normalized = Vec::with_capacity(raw.len());
    let mut seen = HashSet::new();

    for name in raw {
        let trimmed = name.trim();
        validate_identifier(trimmed, label)?;

        let lower = trimmed.to_lowercase();
        if !seen.insert(lower.clone()) {
            return Err(OdrError::Validation(format!(
                "Duplicate {label} names are not allowed (case-insensitive collision on '{name}')"
            )));
        }
        normalized.push(lower);
    }

    Ok(normalized)
}

/// Validates that the sets of independent variables and parameters are disjoint.
///
/// # Errors
/// Returns `OdrError::Validation` if a symbol is both an independent variable and a parameter.
pub fn validate_symbol_sets(independent: &[String], parameters: &[String]) -> OdrResult<()> {
    let independent_set: HashSet<&str> = independent.iter().map(String::as_str).collect();
    let parameter_set: HashSet<&str> = parameters.iter().map(String::as_str).collect();

    if let Some(symbol) = independent_set.intersection(&parameter_set).next() {
        return Err(OdrError::Validation(format!(
            "Symbol '{symbol}' is used both as independent variable and parameter"
        )));
    }

    Ok(())
}
