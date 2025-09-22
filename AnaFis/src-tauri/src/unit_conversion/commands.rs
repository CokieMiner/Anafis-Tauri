// src-tauri/src/unit_conversion/commands.rs

use tauri::command;
use serde::{Deserialize, Serialize};
use crate::unit_conversion::core::{
    UNIT_CONVERTER, ConversionRequest, ConversionResult,
    ConversionPreview, UnitInfo, Dimension
};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct RangeConversionRequest {
    pub range: String,
    pub from_unit: String,
    pub to_unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RangeConversionResult {
    pub range: String,
    pub converted_count: usize,
    pub preview: String,
}

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

// ===== BASIC CONVERSION COMMANDS =====

#[command]
pub async fn convert_value(request: ConversionRequest) -> Result<ConversionResult, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;
    converter.convert(&request)
}

#[command]
pub async fn get_conversion_preview(from_unit: String, to_unit: String) -> Result<ConversionPreview, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;
    Ok(converter.get_conversion_preview(&from_unit, &to_unit))
}

#[command]
pub async fn check_unit_compatibility(from_unit: String, to_unit: String) -> Result<bool, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;
    Ok(converter.check_unit_compatibility(&from_unit, &to_unit))
}

#[command]
pub async fn get_available_units() -> Result<HashMap<String, UnitInfo>, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;
    Ok(converter.get_available_units())
}

// ===== QUICK CONVERSION FOR MENU BUTTONS =====

#[command]
pub async fn quick_convert_value(value: f64, from_unit: String, to_unit: String) -> Result<Option<f64>, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;
    Ok(converter.quick_convert(value, &from_unit, &to_unit))
}

#[command]
pub async fn get_conversion_factor(from_unit: String, to_unit: String) -> Result<f64, String> {
    let dummy_request = ConversionRequest {
        value: 1.0,
        from_unit,
        to_unit,
    };

    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;
    match converter.convert(&dummy_request) {
        Ok(result) => Ok(result.conversion_factor),
        Err(e) => Err(e),
    }
}

// ===== ADVANCED DIMENSIONAL ANALYSIS =====

#[command]
pub async fn parse_unit_formula(unit_formula: String) -> Result<DimensionalAnalysisResult, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;

    match converter.parse_unit(&unit_formula) {
        Ok(parsed) => {
            Ok(DimensionalAnalysisResult {
                unit_formula: unit_formula.clone(),
                dimensional_formula: format_dimension(&parsed.dimension),
                si_factor: parsed.si_factor,
                is_valid: true,
                error_message: None,
            })
        }
        Err(e) => {
            Ok(DimensionalAnalysisResult {
                unit_formula: unit_formula.clone(),
                dimensional_formula: String::new(),
                si_factor: 0.0,
                is_valid: false,
                error_message: Some(e),
            })
        }
    }
}

#[command]
pub async fn analyze_dimensional_compatibility(unit1: String, unit2: String) -> Result<CompatibilityAnalysisResult, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;

    let parse1 = converter.parse_unit(&unit1);
    let parse2 = converter.parse_unit(&unit2);

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
        (Err(e1), _) => {
            Ok(CompatibilityAnalysisResult {
                unit1: unit1.clone(),
                unit2: unit2.clone(),
                are_compatible: false,
                unit1_formula: String::new(),
                unit2_formula: String::new(),
                conversion_factor: None,
                analysis_details: format!("Error parsing {unit1}: {e1}"),
            })
        }
        (_, Err(e2)) => {
            Ok(CompatibilityAnalysisResult {
                unit1: unit1.clone(),
                unit2: unit2.clone(),
                are_compatible: false,
                unit1_formula: String::new(),
                unit2_formula: String::new(),
                conversion_factor: None,
                analysis_details: format!("Error parsing {unit2}: {e2}"),
            })
        }
    }
}

#[command]
pub async fn get_unit_dimensional_formula(unit: String) -> Result<String, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;

    match converter.parse_unit(&unit) {
        Ok(parsed) => Ok(format_dimension(&parsed.dimension)),
        Err(e) => Err(e)
    }
}

// ===== SPREADSHEET INTEGRATION =====

#[command]
pub async fn convert_spreadsheet_range(request: RangeConversionRequest) -> Result<RangeConversionResult, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;

    let preview = converter.get_conversion_preview(&request.from_unit, &request.to_unit);

    if !preview.is_valid {
        return Err(format!("Invalid unit conversion: {} to {}", request.from_unit, request.to_unit));
    }

    Ok(RangeConversionResult {
        range: request.range.clone(),
        converted_count: 0, // Would be calculated based on actual range
        preview: format!("Would convert range {} from {} to {} (factor: {:.6})",
                        request.range, request.from_unit, request.to_unit, preview.conversion_factor),
    })
}

// ===== UTILITY COMMANDS =====

#[command]
pub async fn validate_unit_string(unit: String) -> Result<bool, String> {
    let converter = UNIT_CONVERTER.lock().map_err(|e| format!("Failed to lock converter: {e}"))?;

    match converter.parse_unit(&unit) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[command]
pub async fn get_supported_categories() -> Result<Vec<String>, String> {
    Ok(vec![
        "length".to_string(),
        "mass".to_string(),
        "time".to_string(),
        "velocity".to_string(),
        "force".to_string(),
        "energy".to_string(),
        "power".to_string(),
        "pressure".to_string(),
        "frequency".to_string(),
        "current".to_string(),
        "temperature".to_string(),
        "other".to_string(),
    ])
}

// ===== HELPER FUNCTIONS =====

fn format_dimension(dim: &Dimension) -> String {
    let mut parts = Vec::new();

    if dim.mass != 0 {
        parts.push(if dim.mass == 1 { "M".to_string() } else { format!("M^{}", dim.mass) });
    }
    if dim.length != 0 {
        parts.push(if dim.length == 1 { "L".to_string() } else { format!("L^{}", dim.length) });
    }
    if dim.time != 0 {
        parts.push(if dim.time == 1 { "T".to_string() } else { format!("T^{}", dim.time) });
    }
    if dim.current != 0 {
        parts.push(if dim.current == 1 { "I".to_string() } else { format!("I^{}", dim.current) });
    }
    if dim.temperature != 0 {
        parts.push(if dim.temperature == 1 { "Θ".to_string() } else { format!("Θ^{}", dim.temperature) });
    }
    if dim.amount != 0 {
        parts.push(if dim.amount == 1 { "N".to_string() } else { format!("N^{}", dim.amount) });
    }
    if dim.luminosity != 0 {
        parts.push(if dim.luminosity == 1 { "J".to_string() } else { format!("J^{}", dim.luminosity) });
    }

    if parts.is_empty() {
        "[dimensionless]".to_string()
    } else {
        format!("[{}]", parts.join("·"))
    }
}