// src-tauri/src/unit_conversion/core.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use regex::Regex;
use once_cell::sync::Lazy;
use super::units::{register_all_units, get_unit_categories};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversionRequest {
    pub value: f64,
    pub from_unit: String,
    pub to_unit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversionResult {
    pub value: f64,
    pub formatted_result: String,
    pub conversion_factor: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnitInfo {
    pub symbol: String,
    pub name: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversionPreview {
    pub preview_text: String,
    pub conversion_factor: f64,
    pub is_valid: bool,
}

/// Represents the dimensional formula of a unit in terms of SI base units
/// [M^a L^b T^c I^d Θ^e N^f J^g] where:
/// M = mass (kg), L = length (m), T = time (s), I = electric current (A)
/// Θ = temperature (K), N = amount of substance (mol), J = luminous intensity (cd)
#[derive(Debug, Clone, PartialEq)]
pub struct Dimension {
    pub mass: i32,      // M
    pub length: i32,    // L
    pub time: i32,      // T
    pub current: i32,   // I
    pub temperature: i32, // Θ
    pub amount: i32,    // N
    pub luminosity: i32, // J
}

impl Dimension {
    pub fn new() -> Self {
        Dimension {
            mass: 0, length: 0, time: 0, current: 0,
            temperature: 0, amount: 0, luminosity: 0
        }
    }

    pub fn is_compatible(&self, other: &Dimension) -> bool {
        self == other
    }

    pub fn multiply(&self, other: &Dimension) -> Dimension {
        Dimension {
            mass: self.mass + other.mass,
            length: self.length + other.length,
            time: self.time + other.time,
            current: self.current + other.current,
            temperature: self.temperature + other.temperature,
            amount: self.amount + other.amount,
            luminosity: self.luminosity + other.luminosity,
        }
    }

    pub fn power(&self, exponent: i32) -> Dimension {
        Dimension {
            mass: self.mass * exponent,
            length: self.length * exponent,
            time: self.time * exponent,
            current: self.current * exponent,
            temperature: self.temperature * exponent,
            amount: self.amount * exponent,
            luminosity: self.luminosity * exponent,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BaseUnit {
    pub symbol: String,
    pub name: String,
    pub dimension: Dimension,
    pub si_factor: f64, // Conversion factor to SI base unit
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedUnit {
    pub dimension: Dimension,
    pub si_factor: f64,
    pub original: String,
}

pub struct UnitConverter {
    base_units: HashMap<String, BaseUnit>,
    prefixes: HashMap<String, f64>,
    categories: HashMap<String, Vec<String>>,
    quick_conversions: HashMap<String, HashMap<String, f64>>, // For fast menu button conversions
}

impl UnitConverter {
    pub fn new() -> Self {
        let mut converter = UnitConverter {
            base_units: HashMap::new(),
            prefixes: HashMap::new(),
            categories: HashMap::new(),
            quick_conversions: HashMap::new(),
        };

        converter.initialize_base_units();
        converter.initialize_prefixes();
        converter.initialize_categories();
        converter.initialize_quick_conversions();
        converter
    }

    fn initialize_base_units(&mut self) {
        self.base_units = register_all_units();
    }

    fn initialize_categories(&mut self) {
        self.categories = get_unit_categories();
    }

    fn initialize_prefixes(&mut self) {
        self.prefixes.insert("Y".to_string(), 1e24);   // yotta
        self.prefixes.insert("Z".to_string(), 1e21);   // zetta
        self.prefixes.insert("E".to_string(), 1e18);   // exa
        self.prefixes.insert("P".to_string(), 1e15);   // peta
        self.prefixes.insert("T".to_string(), 1e12);   // tera
        self.prefixes.insert("G".to_string(), 1e9);    // giga
        self.prefixes.insert("M".to_string(), 1e6);    // mega
        self.prefixes.insert("k".to_string(), 1e3);    // kilo
        self.prefixes.insert("h".to_string(), 1e2);    // hecto
        self.prefixes.insert("da".to_string(), 1e1);   // deka
        self.prefixes.insert("d".to_string(), 1e-1);   // deci
        self.prefixes.insert("c".to_string(), 1e-2);   // centi
        self.prefixes.insert("m".to_string(), 1e-3);   // milli
        self.prefixes.insert("μ".to_string(), 1e-6);   // micro
        self.prefixes.insert("u".to_string(), 1e-6);   // micro (alternative)
        self.prefixes.insert("n".to_string(), 1e-9);   // nano
        self.prefixes.insert("p".to_string(), 1e-12);  // pico
        self.prefixes.insert("f".to_string(), 1e-15);  // femto
        self.prefixes.insert("a".to_string(), 1e-18);  // atto
        self.prefixes.insert("z".to_string(), 1e-21);  // zepto
        self.prefixes.insert("y".to_string(), 1e-24);  // yocto
    }

    fn initialize_quick_conversions(&mut self) {
        // For fast menu button conversions - pre-calculated common conversions
        let mut length_conversions = HashMap::new();
        length_conversions.insert("cm_to_in".to_string(), 0.393701);
        length_conversions.insert("in_to_cm".to_string(), 2.54);
        length_conversions.insert("ft_to_m".to_string(), 0.3048);
        length_conversions.insert("m_to_ft".to_string(), 3.28084);
        length_conversions.insert("mi_to_km".to_string(), 1.60934);
        length_conversions.insert("km_to_mi".to_string(), 0.621371);

        let mut mass_conversions = HashMap::new();
        mass_conversions.insert("lb_to_kg".to_string(), 0.453592);
        mass_conversions.insert("kg_to_lb".to_string(), 2.20462);
        mass_conversions.insert("oz_to_g".to_string(), 28.3495);
        mass_conversions.insert("g_to_oz".to_string(), 0.035274);

        self.quick_conversions.insert("length".to_string(), length_conversions);
        self.quick_conversions.insert("mass".to_string(), mass_conversions);
    }

    /// Parse a unit string into its dimensional components
    pub fn parse_unit(&self, unit_str: &str) -> Result<ParsedUnit, String> {
        // Updated regex to support: ^ (caret), ** (double asterisk), and Unicode superscript (⁻, ⁰-⁹)
        static UNIT_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"([a-zA-Zμ°]+)([\^\*]*[⁻⁰¹²³⁴⁵⁶⁷⁸⁹]+|[\^\*]*-?\d+)?").unwrap()
        });

        let mut total_dimension = Dimension::new();
        let mut total_factor = 1.0;

        // Handle simple cases first
        if let Some(base_unit) = self.base_units.get(unit_str) {
            return Ok(ParsedUnit {
                dimension: base_unit.dimension.clone(),
                si_factor: base_unit.si_factor,
                original: unit_str.to_string(),
            });
        }

        // Parse complex units like "m/s^2", "kg*m/s^2", "m·kg/s^2", "m**2", "m⁻²", "(kg·m⁻²)/(s⁴·A⁻¹)", etc.
        // First, remove parentheses and handle them as grouping only
        let normalized = unit_str
            .replace("(", "")
            .replace(")", "")
            .replace("*", " ")
            .replace("·", " ")
            .replace("/", " / ");
        let parts: Vec<&str> = normalized.split_whitespace().collect();

        let mut dividing = false;

        for part in parts {
            if part == "/" {
                dividing = true;
                continue;
            }

            if let Some(captures) = UNIT_REGEX.captures(part) {
                let unit_part = captures.get(1).unwrap().as_str();
                let power_part = captures.get(2).map(|m| m.as_str());

                let power = if let Some(pow_str) = power_part {
                    // Handle different exponent formats
                    let clean_pow = pow_str
                        .trim_start_matches('^')
                        .trim_start_matches('*')
                        .replace("**", ""); // Remove ** prefix

                    // Convert Unicode superscript to regular digits
                    let normalized_pow = clean_pow
                        .replace("⁰", "0")
                        .replace("¹", "1")
                        .replace("²", "2")
                        .replace("³", "3")
                        .replace("⁴", "4")
                        .replace("⁵", "5")
                        .replace("⁶", "6")
                        .replace("⁷", "7")
                        .replace("⁸", "8")
                        .replace("⁹", "9")
                        .replace("⁻", "-");

                    normalized_pow.parse::<i32>().unwrap_or(1)
                } else {
                    1
                };

                let actual_power = if dividing { -power } else { power };

                // Try to find the unit (with potential prefix)
                if let Some((unit, factor)) = self.parse_unit_with_prefix(unit_part) {
                    total_dimension = total_dimension.multiply(&unit.dimension.power(actual_power));
                    total_factor *= factor.powi(actual_power);
                } else {
                    return Err(format!("Unknown unit: {}", unit_part));
                }
            }
        }

        Ok(ParsedUnit {
            dimension: total_dimension,
            si_factor: total_factor,
            original: unit_str.to_string(),
        })
    }

    fn parse_unit_with_prefix(&self, unit_str: &str) -> Option<(&BaseUnit, f64)> {
        // First try exact match
        if let Some(unit) = self.base_units.get(unit_str) {
            return Some((unit, unit.si_factor));
        }

        // Try with prefixes
        for (prefix, prefix_factor) in &self.prefixes {
            if unit_str.starts_with(prefix) {
                let base_unit_str = &unit_str[prefix.len()..];
                if let Some(base_unit) = self.base_units.get(base_unit_str) {
                    return Some((base_unit, base_unit.si_factor * prefix_factor));
                }
            }
        }

        None
    }

    /// Fast conversion for menu buttons (pre-calculated common conversions)
    pub fn quick_convert(&self, value: f64, from: &str, to: &str) -> Option<f64> {
        // Special handling for temperature conversions
        match (from, to) {
            // Celsius conversions
            ("°C", "K") => return Some(value + 273.15),
            ("K", "°C") => return Some(value - 273.15),
            ("°C", "°F") => return Some(value * 9.0 / 5.0 + 32.0),
            ("°F", "°C") => return Some((value - 32.0) * 5.0 / 9.0),

            // Fahrenheit conversions
            ("°F", "K") => return Some((value - 32.0) * 5.0 / 9.0 + 273.15),
            ("K", "°F") => return Some((value - 273.15) * 9.0 / 5.0 + 32.0),

            // Rankine conversions
            ("°R", "K") => return Some(value * 5.0 / 9.0),
            ("K", "°R") => return Some(value * 9.0 / 5.0),
            ("°R", "°F") => return Some(value - 459.67),
            ("°F", "°R") => return Some(value + 459.67),
            ("°R", "°C") => return Some((value - 491.67) * 5.0 / 9.0),
            ("°C", "°R") => return Some((value + 273.15) * 9.0 / 5.0),

            // Réaumur conversions
            ("°Ré", "°C") => return Some(value * 1.25),
            ("°C", "°Ré") => return Some(value * 0.8),
            ("°Ré", "K") => return Some(value * 1.25 + 273.15),
            ("K", "°Ré") => return Some((value - 273.15) * 0.8),
            ("°Ré", "°F") => return Some(value * 2.25 + 32.0),
            ("°F", "°Ré") => return Some((value - 32.0) / 2.25),

            // Same temperature unit
            ("°C", "°C") | ("°F", "°F") | ("K", "K") | ("°R", "°R") | ("°Ré", "°Ré") => return Some(value),
            _ => {}
        }

        let conversion_key = format!("{}_to_{}", from, to);

        for category_conversions in self.quick_conversions.values() {
            if let Some(&factor) = category_conversions.get(&conversion_key) {
                return Some(value * factor);
            }
        }

        None
    }

    /// Advanced conversion using dimensional analysis
    pub fn convert(&self, request: &ConversionRequest) -> Result<ConversionResult, String> {
        // Try quick conversion first for common cases
        if let Some(quick_result) = self.quick_convert(request.value, &request.from_unit, &request.to_unit) {
            return Ok(ConversionResult {
                value: quick_result,
                formatted_result: self.format_result(request.value, &request.from_unit, quick_result, &request.to_unit),
                conversion_factor: quick_result / request.value,
            });
        }

        // Parse both units
        let from_unit = self.parse_unit(&request.from_unit)?;
        let to_unit = self.parse_unit(&request.to_unit)?;

        // Check dimensional compatibility
        if !from_unit.dimension.is_compatible(&to_unit.dimension) {
            return Err(format!(
                "Incompatible dimensions: {} and {} cannot be converted",
                request.from_unit, request.to_unit
            ));
        }

        // Calculate conversion factor
        let conversion_factor = from_unit.si_factor / to_unit.si_factor;
        let converted_value = request.value * conversion_factor;

        Ok(ConversionResult {
            value: converted_value,
            formatted_result: self.format_result(request.value, &request.from_unit, converted_value, &request.to_unit),
            conversion_factor,
        })
    }

    pub fn get_conversion_preview(&self, from_unit: &str, to_unit: &str) -> ConversionPreview {
        if from_unit == to_unit {
            return ConversionPreview {
                preview_text: format!("1 {} = 1 {}", from_unit, to_unit),
                conversion_factor: 1.0,
                is_valid: true,
            };
        }

        let dummy_request = ConversionRequest {
            value: 1.0,
            from_unit: from_unit.to_string(),
            to_unit: to_unit.to_string(),
        };

        match self.convert(&dummy_request) {
            Ok(result) => ConversionPreview {
                preview_text: result.formatted_result,
                conversion_factor: result.conversion_factor,
                is_valid: true,
            },
            Err(error) => ConversionPreview {
                preview_text: error,
                conversion_factor: 0.0,
                is_valid: false,
            },
        }
    }

    pub fn check_unit_compatibility(&self, from_unit: &str, to_unit: &str) -> bool {
        if from_unit == to_unit {
            return true;
        }

        match (self.parse_unit(from_unit), self.parse_unit(to_unit)) {
            (Ok(from), Ok(to)) => from.dimension.is_compatible(&to.dimension),
            _ => false,
        }
    }

    pub fn get_available_units(&self) -> HashMap<String, UnitInfo> {
        let categories = get_unit_categories();
        let mut category_map = HashMap::new();

        // Build reverse lookup: unit symbol -> category
        for (category, units) in categories.iter() {
            for unit_symbol in units {
                category_map.insert(unit_symbol.clone(), category.clone());
            }
        }

        self.base_units.iter().map(|(symbol, unit)| {
            let category = category_map.get(symbol).unwrap_or(&"other".to_string()).clone();
            (symbol.clone(), UnitInfo {
                symbol: symbol.clone(),
                name: unit.name.clone(),
                category,
                description: format!("SI factor: {}", unit.si_factor),
            })
        }).collect()
    }

    #[allow(dead_code)]
    pub fn get_categories(&self) -> &HashMap<String, Vec<String>> {
        &self.categories
    }

    #[allow(dead_code)]
    fn get_unit_category(&self, dimension: &Dimension) -> String {
        match dimension {
            d if d.length == 1 && d.mass == 0 && d.time == 0 => "length".to_string(),
            d if d.mass == 1 && d.length == 0 && d.time == 0 => "mass".to_string(),
            d if d.time == 1 && d.mass == 0 && d.length == 0 => "time".to_string(),
            d if d.length == 1 && d.time == -1 => "velocity".to_string(),
            d if d.mass == 1 && d.length == 1 && d.time == -2 => "force".to_string(),
            d if d.mass == 1 && d.length == 2 && d.time == -2 => "energy".to_string(),
            d if d.mass == 1 && d.length == 2 && d.time == -3 => "power".to_string(),
            d if d.mass == 1 && d.length == -1 && d.time == -2 => "pressure".to_string(),
            d if d.time == -1 => "frequency".to_string(),
            d if d.current == 1 => "current".to_string(),
            d if d.temperature == 1 => "temperature".to_string(),
            _ => "other".to_string(),
        }
    }

    fn format_result(&self, input_value: f64, from_unit: &str, output_value: f64, to_unit: &str) -> String {
        let formatted_output = if output_value.abs() >= 1000000.0 {
            format!("{:.6e}", output_value)
        } else if output_value.abs() >= 1.0 {
            format!("{:.6}", output_value).trim_end_matches('0').trim_end_matches('.').to_string()
        } else if output_value.abs() >= 0.000001 {
            format!("{:.8}", output_value).trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            format!("{:.6e}", output_value)
        };

        format!("{} {} = {} {}", input_value, from_unit, formatted_output, to_unit)
    }
}

// Global converter instance
use std::sync::Mutex;

pub static UNIT_CONVERTER: Lazy<Mutex<UnitConverter>> = Lazy::new(|| {
    Mutex::new(UnitConverter::new())
});