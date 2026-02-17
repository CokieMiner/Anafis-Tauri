// src-tauri/src/unit_conversion/commands.rs

use crate::error::{conversion_error, internal_error, validation_error, CommandResult};
use crate::unit_conversion::core::{
    ConversionPreview, ConversionRequest, ConversionResult, Dimension, UnitConverter, UnitInfo,
    UNIT_CONVERTER,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct DimensionalAnalysisResult {
    pub unit_formula: String,
    pub dimensional_formula: String,
    pub si_factor: f64,
    pub is_valid: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompatibilityAnalysisResult {
    pub unit1: String,
    pub unit2: String,
    pub are_compatible: bool,
    pub unit1_formula: String,
    pub unit2_formula: String,
    pub conversion_factor: Option<f64>,
    pub analysis_details: String,
}

fn with_converter<T>(operation: impl FnOnce(&UnitConverter) -> T) -> CommandResult<T> {
    let converter = UNIT_CONVERTER
        .lock()
        .map_err(|e| internal_error(format!("Failed to lock converter: {e}")))?;
    Ok(operation(&converter))
}

fn with_converter_result<T>(
    operation: impl FnOnce(&UnitConverter) -> Result<T, String>,
) -> CommandResult<T> {
    let converter = UNIT_CONVERTER
        .lock()
        .map_err(|e| internal_error(format!("Failed to lock converter: {e}")))?;
    operation(&converter).map_err(conversion_error)
}

fn with_converter_string_result<T>(
    operation: impl FnOnce(&UnitConverter) -> Result<T, String>,
) -> Result<T, String> {
    let converter = UNIT_CONVERTER
        .lock()
        .map_err(|e| format!("Failed to lock converter: {e}"))?;
    operation(&converter)
}

// ===== BASIC CONVERSION COMMANDS =====

#[command]
pub async fn convert_value(request: ConversionRequest) -> CommandResult<ConversionResult> {
    with_converter_result(|converter| converter.convert(&request))
}

#[command]
pub async fn get_conversion_preview(
    from_unit: String,
    to_unit: String,
) -> CommandResult<ConversionPreview> {
    with_converter(|converter| converter.get_conversion_preview(&from_unit, &to_unit))
}

#[command]
pub async fn check_unit_compatibility(from_unit: String, to_unit: String) -> CommandResult<bool> {
    with_converter(|converter| converter.check_unit_compatibility(&from_unit, &to_unit))
}

#[command]
pub async fn get_available_units() -> CommandResult<HashMap<String, UnitInfo>> {
    with_converter(|converter| converter.get_available_units())
}

// ===== QUICK CONVERSION FOR MENU BUTTONS =====

#[command]
pub async fn quick_convert_value(
    value: f64,
    from_unit: String,
    to_unit: String,
) -> CommandResult<Option<f64>> {
    with_converter(|converter| converter.quick_convert(value, &from_unit, &to_unit))
}

#[command]
pub async fn get_conversion_factor(from_unit: String, to_unit: String) -> CommandResult<f64> {
    let dummy_request = ConversionRequest {
        value: 1.0,
        from_unit,
        to_unit,
    };

    with_converter_result(|converter| {
        converter
            .convert(&dummy_request)
            .map(|result| result.conversion_factor)
    })
}

// ===== ADVANCED DIMENSIONAL ANALYSIS =====

#[command]
pub async fn parse_unit_formula(unit_formula: String) -> Result<DimensionalAnalysisResult, String> {
    match with_converter_string_result(|converter| converter.parse_unit(&unit_formula)) {
        Ok(parsed) => Ok(DimensionalAnalysisResult {
            unit_formula: unit_formula.clone(),
            dimensional_formula: format_dimension(&parsed.dimension),
            si_factor: parsed.si_factor,
            is_valid: true,
            error_message: None,
        }),
        Err(e) => Ok(DimensionalAnalysisResult {
            unit_formula: unit_formula.clone(),
            dimensional_formula: String::new(),
            si_factor: 0.0,
            is_valid: false,
            error_message: Some(e),
        }),
    }
}

#[command]
pub async fn analyze_dimensional_compatibility(
    unit1: String,
    unit2: String,
) -> Result<CompatibilityAnalysisResult, String> {
    let parse1 = with_converter_string_result(|converter| converter.parse_unit(&unit1));
    let parse2 = with_converter_string_result(|converter| converter.parse_unit(&unit2));

    match (parse1, parse2) {
        (Ok(parsed1), Ok(parsed2)) => {
            let compatible = parsed1.dimension.is_compatible(&parsed2.dimension);
            let conversion_factor = if compatible {
                Some(parsed1.si_factor / parsed2.si_factor)
            } else {
                None
            };

            let analysis_details = if compatible {
                format!(
                    "✓ Units are dimensionally compatible\n• {} has dimensional formula: {}\n• {} has dimensional formula: {}\n• Conversion factor: {:.6e}\n• Both units can be converted between each other",
                    unit1, format_dimension(&parsed1.dimension),
                    unit2, format_dimension(&parsed2.dimension),
                    conversion_factor.unwrap()
                )
            } else {
                format!(
                    "✗ Units are dimensionally incompatible\n• {} has dimensional formula: {}\n• {} has dimensional formula: {}\n• These represent different physical quantities and cannot be converted",
                    unit1, format_dimension(&parsed1.dimension),
                    unit2, format_dimension(&parsed2.dimension)
                )
            };

            Ok(CompatibilityAnalysisResult {
                unit1: unit1.clone(),
                unit2: unit2.clone(),
                are_compatible: compatible,
                unit1_formula: format_dimension(&parsed1.dimension),
                unit2_formula: format_dimension(&parsed2.dimension),
                conversion_factor,
                analysis_details,
            })
        }
        (Err(e1), _) => Ok(CompatibilityAnalysisResult {
            unit1: unit1.clone(),
            unit2: unit2.clone(),
            are_compatible: false,
            unit1_formula: String::new(),
            unit2_formula: String::new(),
            conversion_factor: None,
            analysis_details: format!("Error parsing {unit1}: {e1}"),
        }),
        (_, Err(e2)) => Ok(CompatibilityAnalysisResult {
            unit1: unit1.clone(),
            unit2: unit2.clone(),
            are_compatible: false,
            unit1_formula: String::new(),
            unit2_formula: String::new(),
            conversion_factor: None,
            analysis_details: format!("Error parsing {unit2}: {e2}"),
        }),
    }
}

#[command]
pub async fn get_unit_dimensional_formula(unit: String) -> CommandResult<String> {
    let converter = UNIT_CONVERTER
        .lock()
        .map_err(|e| internal_error(format!("Failed to lock converter: {e}")))?;

    match converter.parse_unit(&unit) {
        Ok(parsed) => Ok(format_dimension(&parsed.dimension)),
        Err(e) => Err(validation_error(e, Some("unit".to_string()))),
    }
}

// ===== UTILITY COMMANDS =====

#[command]
pub async fn validate_unit_string(unit: String) -> CommandResult<bool> {
    let converter = UNIT_CONVERTER
        .lock()
        .map_err(|e| internal_error(format!("Failed to lock converter: {e}")))?;

    match converter.parse_unit(&unit) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[command]
pub async fn get_supported_categories() -> CommandResult<Vec<String>> {
    const SUPPORTED_CATEGORIES: [&str; 12] = [
        "length",
        "mass",
        "time",
        "velocity",
        "force",
        "energy",
        "power",
        "pressure",
        "frequency",
        "current",
        "temperature",
        "other",
    ];
    Ok(SUPPORTED_CATEGORIES.iter().map(|c| c.to_string()).collect())
}

// ===== HELPER FUNCTIONS =====

fn format_dimension(dim: &Dimension) -> String {
    let mut parts = Vec::new();

    if dim.mass != 0 {
        parts.push(if dim.mass == 1 {
            "M".to_string()
        } else {
            format!("M^{}", dim.mass)
        });
    }
    if dim.length != 0 {
        parts.push(if dim.length == 1 {
            "L".to_string()
        } else {
            format!("L^{}", dim.length)
        });
    }
    if dim.time != 0 {
        parts.push(if dim.time == 1 {
            "T".to_string()
        } else {
            format!("T^{}", dim.time)
        });
    }
    if dim.current != 0 {
        parts.push(if dim.current == 1 {
            "I".to_string()
        } else {
            format!("I^{}", dim.current)
        });
    }
    if dim.temperature != 0 {
        parts.push(if dim.temperature == 1 {
            "Θ".to_string()
        } else {
            format!("Θ^{}", dim.temperature)
        });
    }
    if dim.amount != 0 {
        parts.push(if dim.amount == 1 {
            "N".to_string()
        } else {
            format!("N^{}", dim.amount)
        });
    }
    if dim.luminosity != 0 {
        parts.push(if dim.luminosity == 1 {
            "J".to_string()
        } else {
            format!("J^{}", dim.luminosity)
        });
    }

    if parts.is_empty() {
        "[dimensionless]".to_string()
    } else {
        format!("[{}]", parts.join("·"))
    }
}
