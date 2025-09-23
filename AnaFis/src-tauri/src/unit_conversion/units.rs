// src-tauri/src/unit_conversion/units.rs

use std::collections::HashMap;
use super::core::{BaseUnit, Dimension};

/// Register all base units organized by category
pub fn register_all_units() -> HashMap<String, BaseUnit> {
    let mut units = HashMap::new();

    // Register units by category
    register_si_base_units(&mut units);
    register_length_units(&mut units);
    register_mass_units(&mut units);
    register_time_units(&mut units);
    register_temperature_units(&mut units);
    register_current_units(&mut units);
    register_amount_units(&mut units);
    register_luminous_intensity_units(&mut units);
    register_angle_units(&mut units);
    register_area_units(&mut units);
    register_volume_units(&mut units);
    register_velocity_units(&mut units);
    register_acceleration_units(&mut units);
    register_force_units(&mut units);
    register_pressure_units(&mut units);
    register_energy_units(&mut units);
    register_power_units(&mut units);
    register_frequency_units(&mut units);
    register_voltage_units(&mut units);
    register_resistance_units(&mut units);
    register_capacitance_units(&mut units);
    register_inductance_units(&mut units);
    register_conductance_units(&mut units);
    register_electrical_units(&mut units);
    register_magnetic_flux_density_units(&mut units);
    register_magnetic_flux_units(&mut units);
    register_electric_charge_units(&mut units);
    register_radiation_activity_units(&mut units);
    register_radiation_dose_units(&mut units);
    register_illuminance_units(&mut units);
    register_data_storage_units(&mut units);
    register_computing_units(&mut units);
    register_textile_units(&mut units);

    units
}

/// Get all unit categories with their respective unit symbols
pub fn get_unit_categories() -> HashMap<String, Vec<String>> {
    let mut categories = HashMap::new();

    categories.insert("length".to_string(), vec![
        "m".to_string(), "mm".to_string(), "cm".to_string(), "km".to_string(),
        "in".to_string(), "ft".to_string(), "yd".to_string(), "mi".to_string(),
        "mil".to_string(), "μm".to_string(), "nm".to_string(), "Å".to_string(),
        "nmi".to_string(), "fathom".to_string(), "au".to_string(), "ly".to_string(),
        "pc".to_string(), "cubit".to_string(), "furlong".to_string(), "league".to_string(),
        "chain".to_string(), "rod".to_string(), "link".to_string(),
        "point".to_string(), "pica".to_string()
    ]);

    categories.insert("mass".to_string(), vec![
        "kg".to_string(), "g".to_string(), "mg".to_string(), "μg".to_string(),
        "lb".to_string(), "oz".to_string(), "u".to_string(), "stone".to_string(),
        "ton".to_string(), "metric_ton".to_string(), "grain".to_string()
    ]);

    categories.insert("time".to_string(), vec![
        "s".to_string(), "min".to_string(), "h".to_string(), "day".to_string(),
        "week".to_string(), "year".to_string(), "ns".to_string(), "μs".to_string(), "ms".to_string()
    ]);

    categories.insert("temperature".to_string(), vec![
        "K".to_string(), "°C".to_string(), "°F".to_string(), "°R".to_string(), "°Ré".to_string()
    ]);

    categories.insert("current".to_string(), vec![
        "A".to_string(), "mA".to_string(), "μA".to_string(), "kA".to_string()
    ]);

    categories.insert("amount".to_string(), vec![
        "mol".to_string(), "mmol".to_string(), "μmol".to_string(), "kmol".to_string()
    ]);

    categories.insert("luminous_intensity".to_string(), vec![
        "cd".to_string(), "mcd".to_string(), "kcd".to_string(), "Mcd".to_string(), "cp".to_string(), "hk".to_string()
    ]);

    categories.insert("angle".to_string(), vec![
        "rad".to_string(), "deg".to_string(), "°".to_string(), "arcmin".to_string(),
        "arcsec".to_string(), "grad".to_string(), "turn".to_string()
    ]);

    categories.insert("area".to_string(), vec![
        "m²".to_string(), "km²".to_string(), "cm²".to_string(), "mm²".to_string(),
        "in²".to_string(), "ft²".to_string(), "yd²".to_string(), "mi²".to_string(),
        "ha".to_string(), "acre".to_string(), "perch".to_string()
    ]);

    categories.insert("volume".to_string(), vec![
        "m³".to_string(), "L".to_string(), "mL".to_string(), "cm³".to_string(),
        "in³".to_string(), "ft³".to_string(), "yd³".to_string(), "gal".to_string(),
        "qt".to_string(), "pt".to_string(), "cup".to_string(), "fl_oz".to_string(),
        "tbsp".to_string(), "tsp".to_string(), "bbl".to_string(), "bushel".to_string(),
        "imp_gal".to_string(), "imp_qt".to_string(), "imp_pt".to_string(), "imp_fl_oz".to_string()
    ]);

    categories.insert("velocity".to_string(), vec![
        "m/s".to_string(), "km/h".to_string(), "mph".to_string(), "fps".to_string(),
        "c".to_string(), "kn".to_string()
    ]);

    categories.insert("acceleration".to_string(), vec![
        "m/s²".to_string(), "ft/s²".to_string(), "g".to_string()
    ]);

    categories.insert("force".to_string(), vec![
        "N".to_string(), "lbf".to_string(), "kgf".to_string(), "dyn".to_string()
    ]);

    categories.insert("pressure".to_string(), vec![
        "Pa".to_string(), "kPa".to_string(), "MPa".to_string(), "GPa".to_string(),
        "bar".to_string(), "atm".to_string(), "psi".to_string(), "mmHg".to_string(),
        "torr".to_string(), "hPa".to_string(), "mbar".to_string(), "inHg".to_string(),
        "bar_abs".to_string()
    ]);

    categories.insert("energy".to_string(), vec![
        "J".to_string(), "kJ".to_string(), "MJ".to_string(), "GJ".to_string(),
        "cal".to_string(), "kcal".to_string(), "BTU".to_string(), "kWh".to_string(),
        "Wh".to_string(), "eV".to_string(), "keV".to_string(), "MeV".to_string(),
        "GeV".to_string(), "erg".to_string(), "dyne_cm".to_string()
    ]);

    categories.insert("power".to_string(), vec![
        "W".to_string(), "kW".to_string(), "MW".to_string(), "GW".to_string(),
        "mW".to_string(), "hp".to_string(), "PS".to_string(), "erg/s".to_string(), "ft.lbf/min".to_string(), "BTU/h".to_string()
    ]);

    categories.insert("frequency".to_string(), vec![
        "Hz".to_string(), "kHz".to_string(), "MHz".to_string(), "GHz".to_string()
    ]);

    categories.insert("voltage".to_string(), vec![
        "V".to_string(), "mV".to_string(), "kV".to_string(), "MV".to_string()
    ]);

    categories.insert("resistance".to_string(), vec![
        "Ω".to_string(), "mΩ".to_string(), "kΩ".to_string(), "MΩ".to_string()
    ]);

    categories.insert("capacitance".to_string(), vec![
        "F".to_string(), "μF".to_string(), "nF".to_string(), "pF".to_string()
    ]);

    categories.insert("inductance".to_string(), vec![
        "H".to_string(), "mH".to_string(), "μH".to_string(), "nH".to_string()
    ]);

    categories.insert("conductance".to_string(), vec![
        "S".to_string(), "mS".to_string(), "μS".to_string()
    ]);

    categories.insert("magnetic_flux_density".to_string(), vec![
        "T".to_string(), "mT".to_string(), "μT".to_string(), "G".to_string(), "gamma".to_string()
    ]);

    categories.insert("magnetic_flux".to_string(), vec![
        "Wb".to_string(), "mWb".to_string(), "Mx".to_string(), "unit_pole".to_string(), "statWb".to_string()
    ]);

    categories.insert("electric_charge".to_string(), vec![
        "C".to_string(), "mC".to_string(), "μC".to_string(), "nC".to_string(),
        "pC".to_string(), "kC".to_string(), "MC".to_string()
    ]);

    categories.insert("radiation_activity".to_string(), vec![
        "Bq".to_string(), "Ci".to_string()
    ]);

    categories.insert("radiation_dose".to_string(), vec![
        "Gy".to_string(), "Sv".to_string(), "rem".to_string(), "rd".to_string()
    ]);

    categories.insert("illuminance".to_string(), vec![
        "lx".to_string(), "fc".to_string(), "ph".to_string()
    ]);

    categories.insert("data_storage".to_string(), vec![
        "byte".to_string(), "kB".to_string(), "MB".to_string(), "GB".to_string(),
        "TB".to_string(), "PB".to_string(), "bit".to_string(), "Kibit".to_string(),
        "Mibit".to_string(), "Gibit".to_string(), "KiB".to_string(), "MiB".to_string(),
        "GiB".to_string()
    ]);

    categories.insert("computing".to_string(), vec![
        "FLOPS".to_string(), "MIPS".to_string()
    ]);

    categories.insert("textile".to_string(), vec![
        "tex".to_string(), "denier".to_string()
    ]);

    categories
}

// === SI BASE UNITS ===
fn register_si_base_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("m".to_string(), BaseUnit {
        symbol: "m".to_string(),
        name: "meter".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("kg".to_string(), BaseUnit {
        symbol: "kg".to_string(),
        name: "kilogram".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("s".to_string(), BaseUnit {
        symbol: "s".to_string(),
        name: "second".to_string(),
        dimension: Dimension { time: 1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("A".to_string(), BaseUnit {
        symbol: "A".to_string(),
        name: "ampere".to_string(),
        dimension: Dimension { current: 1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("K".to_string(), BaseUnit {
        symbol: "K".to_string(),
        name: "kelvin".to_string(),
        dimension: Dimension { temperature: 1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("mol".to_string(), BaseUnit {
        symbol: "mol".to_string(),
        name: "mole".to_string(),
        dimension: Dimension { amount: 1, ..Dimension::new() },
        si_factor: 1.0,
    });
}


// === LENGTH UNITS ===
fn register_length_units(units: &mut HashMap<String, BaseUnit>) {
    // Metric length units
    units.insert("mm".to_string(), BaseUnit {
        symbol: "mm".to_string(),
        name: "millimeter".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("cm".to_string(), BaseUnit {
        symbol: "cm".to_string(),
        name: "centimeter".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1e-2,
    });

    units.insert("km".to_string(), BaseUnit {
        symbol: "km".to_string(),
        name: "kilometer".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1e3,
    });

    units.insert("μm".to_string(), BaseUnit {
        symbol: "μm".to_string(),
        name: "micrometer".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("nm".to_string(), BaseUnit {
        symbol: "nm".to_string(),
        name: "nanometer".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1e-9,
    });

    units.insert("Å".to_string(), BaseUnit {
        symbol: "Å".to_string(),
        name: "angstrom".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1e-10,
    });

    // Imperial/US length units
    units.insert("in".to_string(), BaseUnit {
        symbol: "in".to_string(),
        name: "inch".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 0.0254,
    });

    units.insert("ft".to_string(), BaseUnit {
        symbol: "ft".to_string(),
        name: "foot".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 0.3048,
    });

    units.insert("yd".to_string(), BaseUnit {
        symbol: "yd".to_string(),
        name: "yard".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 0.9144,
    });

    units.insert("mi".to_string(), BaseUnit {
        symbol: "mi".to_string(),
        name: "mile".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1609.34,
    });

    units.insert("mil".to_string(), BaseUnit {
        symbol: "mil".to_string(),
        name: "mil".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 2.54e-5,
    });

    // Maritime units
    units.insert("nmi".to_string(), BaseUnit {
        symbol: "nmi".to_string(),
        name: "nautical mile".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1852.0,
    });

    units.insert("fathom".to_string(), BaseUnit {
        symbol: "fathom".to_string(),
        name: "fathom".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1.8288,
    });

    // Astronomical units
    units.insert("au".to_string(), BaseUnit {
        symbol: "au".to_string(),
        name: "astronomical unit".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 1.495978707e11,
    });

    units.insert("ly".to_string(), BaseUnit {
        symbol: "ly".to_string(),
        name: "light year".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 9.4607304725808e15,
    });

    units.insert("pc".to_string(), BaseUnit {
        symbol: "pc".to_string(),
        name: "parsec".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 3.0857e16,
    });

    // Historical units
    units.insert("cubit".to_string(), BaseUnit {
        symbol: "cubit".to_string(),
        name: "cubit".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 0.4572,
    });

    units.insert("furlong".to_string(), BaseUnit {
        symbol: "furlong".to_string(),
        name: "furlong".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 201.168,
    });

    units.insert("league".to_string(), BaseUnit {
        symbol: "league".to_string(),
        name: "league".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 4828.032,
    });

    units.insert("chain".to_string(), BaseUnit {
        symbol: "chain".to_string(),
        name: "chain".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 20.1168,
    });

    units.insert("rod".to_string(), BaseUnit {
        symbol: "rod".to_string(),
        name: "rod".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 5.0292,
    });

    units.insert("link".to_string(), BaseUnit {
        symbol: "link".to_string(),
        name: "surveyor's link".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 0.201168,
    });

    // Printing units
    units.insert("point".to_string(), BaseUnit {
        symbol: "point".to_string(),
        name: "point".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 0.000352778,
    });

    units.insert("pica".to_string(), BaseUnit {
        symbol: "pica".to_string(),
        name: "pica".to_string(),
        dimension: Dimension { length: 1, ..Dimension::new() },
        si_factor: 0.00423333,
    });
}

// === MASS UNITS ===
fn register_mass_units(units: &mut HashMap<String, BaseUnit>) {
    // Metric mass units
    units.insert("g".to_string(), BaseUnit {
        symbol: "g".to_string(),
        name: "gram".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("mg".to_string(), BaseUnit {
        symbol: "mg".to_string(),
        name: "milligram".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("μg".to_string(), BaseUnit {
        symbol: "μg".to_string(),
        name: "microgram".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 1e-9,
    });

    // Imperial/US mass units
    units.insert("lb".to_string(), BaseUnit {
        symbol: "lb".to_string(),
        name: "pound".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 0.453592,
    });

    units.insert("oz".to_string(), BaseUnit {
        symbol: "oz".to_string(),
        name: "ounce".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 0.0283495,
    });

    units.insert("stone".to_string(), BaseUnit {
        symbol: "stone".to_string(),
        name: "stone".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 6.35029,
    });

    units.insert("ton".to_string(), BaseUnit {
        symbol: "ton".to_string(),
        name: "short ton".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 907.185,
    });

    units.insert("metric_ton".to_string(), BaseUnit {
        symbol: "metric_ton".to_string(),
        name: "metric ton".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 1000.0,
    });

    units.insert("grain".to_string(), BaseUnit {
        symbol: "grain".to_string(),
        name: "grain".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 6.4799e-5,
    });

    // Atomic mass unit
    units.insert("u".to_string(), BaseUnit {
        symbol: "u".to_string(),
        name: "atomic mass unit".to_string(),
        dimension: Dimension { mass: 1, ..Dimension::new() },
        si_factor: 1.66054e-27,
    });
}

// === TIME UNITS ===
fn register_time_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("ns".to_string(), BaseUnit {
        symbol: "ns".to_string(),
        name: "nanosecond".to_string(),
        dimension: Dimension { time: 1, ..Dimension::new() },
        si_factor: 1e-9,
    });

    units.insert("μs".to_string(), BaseUnit {
        symbol: "μs".to_string(),
        name: "microsecond".to_string(),
        dimension: Dimension { time: 1, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("ms".to_string(), BaseUnit {
        symbol: "ms".to_string(),
        name: "millisecond".to_string(),
        dimension: Dimension { time: 1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("min".to_string(), BaseUnit {
        symbol: "min".to_string(),
        name: "minute".to_string(),
        dimension: Dimension { time: 1, ..Dimension::new() },
        si_factor: 60.0,
    });

    units.insert("h".to_string(), BaseUnit {
        symbol: "h".to_string(),
        name: "hour".to_string(),
        dimension: Dimension { time: 1, ..Dimension::new() },
        si_factor: 3600.0,
    });

    units.insert("day".to_string(), BaseUnit {
        symbol: "day".to_string(),
        name: "day".to_string(),
        dimension: Dimension { time: 1, ..Dimension::new() },
        si_factor: 86400.0,
    });

    units.insert("week".to_string(), BaseUnit {
        symbol: "week".to_string(),
        name: "week".to_string(),
        dimension: Dimension { time: 1, ..Dimension::new() },
        si_factor: 604800.0,
    });

    units.insert("year".to_string(), BaseUnit {
        symbol: "year".to_string(),
        name: "year".to_string(),
        dimension: Dimension { time: 1, ..Dimension::new() },
        si_factor: 31557600.0, // Julian year
    });
}

// === TEMPERATURE UNITS ===
fn register_temperature_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("°C".to_string(), BaseUnit {
        symbol: "°C".to_string(),
        name: "celsius".to_string(),
        dimension: Dimension { temperature: 1, ..Dimension::new() },
        si_factor: 1.0, // Special handling in conversion logic
    });

    units.insert("°F".to_string(), BaseUnit {
        symbol: "°F".to_string(),
        name: "fahrenheit".to_string(),
        dimension: Dimension { temperature: 1, ..Dimension::new() },
        si_factor: 5.0/9.0, // Special handling in conversion logic
    });

    units.insert("°R".to_string(), BaseUnit {
        symbol: "°R".to_string(),
        name: "rankine".to_string(),
        dimension: Dimension { temperature: 1, ..Dimension::new() },
        si_factor: 5.0/9.0,
    });

    units.insert("°Ré".to_string(), BaseUnit {
        symbol: "°Ré".to_string(),
        name: "réaumur".to_string(),
        dimension: Dimension { temperature: 1, ..Dimension::new() },
        si_factor: 1.25,
    });
}

// === ANGLE UNITS ===
fn register_angle_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("rad".to_string(), BaseUnit {
        symbol: "rad".to_string(),
        name: "radian".to_string(),
        dimension: Dimension::new(), // Dimensionless
        si_factor: 1.0,
    });

    units.insert("deg".to_string(), BaseUnit {
        symbol: "deg".to_string(),
        name: "degree".to_string(),
        dimension: Dimension::new(), // Dimensionless
        si_factor: std::f64::consts::PI / 180.0,
    });

    units.insert("°".to_string(), BaseUnit {
        symbol: "°".to_string(),
        name: "degree".to_string(),
        dimension: Dimension::new(), // Dimensionless
        si_factor: std::f64::consts::PI / 180.0,
    });

    units.insert("arcmin".to_string(), BaseUnit {
        symbol: "arcmin".to_string(),
        name: "arcminute".to_string(),
        dimension: Dimension::new(), // Dimensionless
        si_factor: std::f64::consts::PI / 10800.0,
    });

    units.insert("arcsec".to_string(), BaseUnit {
        symbol: "arcsec".to_string(),
        name: "arcsecond".to_string(),
        dimension: Dimension::new(), // Dimensionless
        si_factor: std::f64::consts::PI / 648000.0,
    });

    units.insert("grad".to_string(), BaseUnit {
        symbol: "grad".to_string(),
        name: "gradian".to_string(),
        dimension: Dimension::new(), // Dimensionless
        si_factor: std::f64::consts::PI / 200.0,
    });

    units.insert("turn".to_string(), BaseUnit {
        symbol: "turn".to_string(),
        name: "full turn".to_string(),
        dimension: Dimension::new(), // Dimensionless
        si_factor: 2.0 * std::f64::consts::PI,
    });
}

// === AREA UNITS ===
fn register_area_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("m²".to_string(), BaseUnit {
        symbol: "m²".to_string(),
        name: "square meter".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("km²".to_string(), BaseUnit {
        symbol: "km²".to_string(),
        name: "square kilometer".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 1e6,
    });

    units.insert("cm²".to_string(), BaseUnit {
        symbol: "cm²".to_string(),
        name: "square centimeter".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 1e-4,
    });

    units.insert("mm²".to_string(), BaseUnit {
        symbol: "mm²".to_string(),
        name: "square millimeter".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("in²".to_string(), BaseUnit {
        symbol: "in²".to_string(),
        name: "square inch".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 6.4516e-4,
    });

    units.insert("ft²".to_string(), BaseUnit {
        symbol: "ft²".to_string(),
        name: "square foot".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 0.092903,
    });

    units.insert("yd²".to_string(), BaseUnit {
        symbol: "yd²".to_string(),
        name: "square yard".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 0.836127,
    });

    units.insert("mi²".to_string(), BaseUnit {
        symbol: "mi²".to_string(),
        name: "square mile".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 2.59e6,
    });

    units.insert("ha".to_string(), BaseUnit {
        symbol: "ha".to_string(),
        name: "hectare".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 10000.0,
    });

    units.insert("acre".to_string(), BaseUnit {
        symbol: "acre".to_string(),
        name: "acre".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 4046.86,
    });

    units.insert("perch".to_string(), BaseUnit {
        symbol: "perch".to_string(),
        name: "perch".to_string(),
        dimension: Dimension { length: 2, ..Dimension::new() },
        si_factor: 25.2929, // 1 perch = 30.25 square yards
    });
}

// === VOLUME UNITS ===
fn register_volume_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("m³".to_string(), BaseUnit {
        symbol: "m³".to_string(),
        name: "cubic meter".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("L".to_string(), BaseUnit {
        symbol: "L".to_string(),
        name: "liter".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("mL".to_string(), BaseUnit {
        symbol: "mL".to_string(),
        name: "milliliter".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("cm³".to_string(), BaseUnit {
        symbol: "cm³".to_string(),
        name: "cubic centimeter".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("in³".to_string(), BaseUnit {
        symbol: "in³".to_string(),
        name: "cubic inch".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 1.6387e-5,
    });

    units.insert("ft³".to_string(), BaseUnit {
        symbol: "ft³".to_string(),
        name: "cubic foot".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.0283168,
    });

    units.insert("yd³".to_string(), BaseUnit {
        symbol: "yd³".to_string(),
        name: "cubic yard".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.764555,
    });

    // US Liquid volume
    units.insert("gal".to_string(), BaseUnit {
        symbol: "gal".to_string(),
        name: "gallon".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.00378541,
    });

    units.insert("qt".to_string(), BaseUnit {
        symbol: "qt".to_string(),
        name: "quart".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.000946353,
    });

    units.insert("pt".to_string(), BaseUnit {
        symbol: "pt".to_string(),
        name: "pint".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.000473176,
    });

    units.insert("cup".to_string(), BaseUnit {
        symbol: "cup".to_string(),
        name: "cup".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.000236588,
    });

    units.insert("fl_oz".to_string(), BaseUnit {
        symbol: "fl_oz".to_string(),
        name: "fluid ounce".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 2.9574e-5,
    });

    units.insert("tbsp".to_string(), BaseUnit {
        symbol: "tbsp".to_string(),
        name: "tablespoon".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 1.4787e-5,
    });

    units.insert("tsp".to_string(), BaseUnit {
        symbol: "tsp".to_string(),
        name: "teaspoon".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 4.9289e-6,
    });

    units.insert("bbl".to_string(), BaseUnit {
        symbol: "bbl".to_string(),
        name: "barrel".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.158987,
    });

    units.insert("bushel".to_string(), BaseUnit {
        symbol: "bushel".to_string(),
        name: "bushel".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.0352391,
    });

    // Imperial volume
    units.insert("imp_gal".to_string(), BaseUnit {
        symbol: "imp_gal".to_string(),
        name: "imperial gallon".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.00454609,
    });

    units.insert("imp_qt".to_string(), BaseUnit {
        symbol: "imp_qt".to_string(),
        name: "imperial quart".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.00113652,
    });

    units.insert("imp_pt".to_string(), BaseUnit {
        symbol: "imp_pt".to_string(),
        name: "imperial pint".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 0.000568261,
    });

    units.insert("imp_fl_oz".to_string(), BaseUnit {
        symbol: "imp_fl_oz".to_string(),
        name: "imperial fluid ounce".to_string(),
        dimension: Dimension { length: 3, ..Dimension::new() },
        si_factor: 2.84131e-5,
    });
}

// === VELOCITY UNITS ===
fn register_velocity_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("m/s".to_string(), BaseUnit {
        symbol: "m/s".to_string(),
        name: "meter per second".to_string(),
        dimension: Dimension { length: 1, time: -1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("km/h".to_string(), BaseUnit {
        symbol: "km/h".to_string(),
        name: "kilometer per hour".to_string(),
        dimension: Dimension { length: 1, time: -1, ..Dimension::new() },
        si_factor: 1.0 / 3.6,
    });

    units.insert("mph".to_string(), BaseUnit {
        symbol: "mph".to_string(),
        name: "mile per hour".to_string(),
        dimension: Dimension { length: 1, time: -1, ..Dimension::new() },
        si_factor: 0.44704,
    });

    units.insert("fps".to_string(), BaseUnit {
        symbol: "fps".to_string(),
        name: "foot per second".to_string(),
        dimension: Dimension { length: 1, time: -1, ..Dimension::new() },
        si_factor: 0.3048,
    });

    units.insert("c".to_string(), BaseUnit {
        symbol: "c".to_string(),
        name: "speed of light".to_string(),
        dimension: Dimension { length: 1, time: -1, ..Dimension::new() },
        si_factor: 299792458.0,
    });

    units.insert("kn".to_string(), BaseUnit {
        symbol: "kn".to_string(),
        name: "knot".to_string(),
        dimension: Dimension { length: 1, time: -1, ..Dimension::new() },
        si_factor: 0.514444,
    });
}

// === ACCELERATION UNITS ===
fn register_acceleration_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("m/s²".to_string(), BaseUnit {
        symbol: "m/s²".to_string(),
        name: "meter per second squared".to_string(),
        dimension: Dimension { length: 1, time: -2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("ft/s²".to_string(), BaseUnit {
        symbol: "ft/s²".to_string(),
        name: "foot per second squared".to_string(),
        dimension: Dimension { length: 1, time: -2, ..Dimension::new() },
        si_factor: 0.3048,
    });

    // Note: 'g' for gravity acceleration, different from gram
    units.insert("g".to_string(), BaseUnit {
        symbol: "g".to_string(),
        name: "standard gravity".to_string(),
        dimension: Dimension { length: 1, time: -2, ..Dimension::new() },
        si_factor: 9.80665,
    });
}

// === FORCE UNITS ===
fn register_force_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("N".to_string(), BaseUnit {
        symbol: "N".to_string(),
        name: "newton".to_string(),
        dimension: Dimension { mass: 1, length: 1, time: -2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("lbf".to_string(), BaseUnit {
        symbol: "lbf".to_string(),
        name: "pound-force".to_string(),
        dimension: Dimension { mass: 1, length: 1, time: -2, ..Dimension::new() },
        si_factor: 4.44822,
    });

    units.insert("kgf".to_string(), BaseUnit {
        symbol: "kgf".to_string(),
        name: "kilogram-force".to_string(),
        dimension: Dimension { mass: 1, length: 1, time: -2, ..Dimension::new() },
        si_factor: 9.80665,
    });

    units.insert("dyn".to_string(), BaseUnit {
        symbol: "dyn".to_string(),
        name: "dyne".to_string(),
        dimension: Dimension { mass: 1, length: 1, time: -2, ..Dimension::new() },
        si_factor: 1e-5,
    });
}

// === PRESSURE UNITS ===
fn register_pressure_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("Pa".to_string(), BaseUnit {
        symbol: "Pa".to_string(),
        name: "pascal".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("kPa".to_string(), BaseUnit {
        symbol: "kPa".to_string(),
        name: "kilopascal".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 1000.0,
    });

    units.insert("MPa".to_string(), BaseUnit {
        symbol: "MPa".to_string(),
        name: "megapascal".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 1000000.0,
    });

    units.insert("GPa".to_string(), BaseUnit {
        symbol: "GPa".to_string(),
        name: "gigapascal".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 1000000000.0,
    });

    units.insert("bar".to_string(), BaseUnit {
        symbol: "bar".to_string(),
        name: "bar".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 1e5,
    });

    units.insert("atm".to_string(), BaseUnit {
        symbol: "atm".to_string(),
        name: "atmosphere".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 101325.0,
    });

    units.insert("psi".to_string(), BaseUnit {
        symbol: "psi".to_string(),
        name: "pound per square inch".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 6894.76,
    });

    units.insert("mmHg".to_string(), BaseUnit {
        symbol: "mmHg".to_string(),
        name: "millimeter of mercury".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 133.322,
    });

    units.insert("torr".to_string(), BaseUnit {
        symbol: "torr".to_string(),
        name: "torr".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 133.322,
    });

    units.insert("hPa".to_string(), BaseUnit {
        symbol: "hPa".to_string(),
        name: "hectopascal".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 100.0,
    });

    units.insert("mbar".to_string(), BaseUnit {
        symbol: "mbar".to_string(),
        name: "millibar".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 100.0,
    });

    units.insert("inHg".to_string(), BaseUnit {
        symbol: "inHg".to_string(),
        name: "inch of mercury".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 3386.39,
    });

    units.insert("bar_abs".to_string(), BaseUnit {
        symbol: "bar_abs".to_string(),
        name: "bar absolute".to_string(),
        dimension: Dimension { mass: 1, length: -1, time: -2, ..Dimension::new() },
        si_factor: 100000.0,
    });
}

// === ENERGY UNITS ===
fn register_energy_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("J".to_string(), BaseUnit {
        symbol: "J".to_string(),
        name: "joule".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("kJ".to_string(), BaseUnit {
        symbol: "kJ".to_string(),
        name: "kilojoule".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1000.0,
    });

    units.insert("MJ".to_string(), BaseUnit {
        symbol: "MJ".to_string(),
        name: "megajoule".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1000000.0,
    });

    units.insert("GJ".to_string(), BaseUnit {
        symbol: "GJ".to_string(),
        name: "gigajoule".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1000000000.0,
    });

    units.insert("cal".to_string(), BaseUnit {
        symbol: "cal".to_string(),
        name: "calorie".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 4.184,
    });

    units.insert("kcal".to_string(), BaseUnit {
        symbol: "kcal".to_string(),
        name: "kilocalorie".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 4184.0,
    });

    units.insert("BTU".to_string(), BaseUnit {
        symbol: "BTU".to_string(),
        name: "british thermal unit".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1055.06,
    });

    units.insert("kWh".to_string(), BaseUnit {
        symbol: "kWh".to_string(),
        name: "kilowatt-hour".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 3.6e6,
    });

    units.insert("Wh".to_string(), BaseUnit {
        symbol: "Wh".to_string(),
        name: "watt-hour".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 3600.0,
    });

    units.insert("eV".to_string(), BaseUnit {
        symbol: "eV".to_string(),
        name: "electron volt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1.602176634e-19,
    });

    units.insert("keV".to_string(), BaseUnit {
        symbol: "keV".to_string(),
        name: "kiloelectron volt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1.602176634e-16,
    });

    units.insert("MeV".to_string(), BaseUnit {
        symbol: "MeV".to_string(),
        name: "megaelectron volt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1.602176634e-13,
    });

    units.insert("GeV".to_string(), BaseUnit {
        symbol: "GeV".to_string(),
        name: "gigaelectron volt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1.602176634e-10,
    });

    units.insert("erg".to_string(), BaseUnit {
        symbol: "erg".to_string(),
        name: "erg".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1e-7,
    });

    units.insert("dyne_cm".to_string(), BaseUnit {
        symbol: "dyne_cm".to_string(),
        name: "dyne centimeter".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, ..Dimension::new() },
        si_factor: 1e-7,
    });
}

// === POWER UNITS ===
fn register_power_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("W".to_string(), BaseUnit {
        symbol: "W".to_string(),
        name: "watt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("kW".to_string(), BaseUnit {
        symbol: "kW".to_string(),
        name: "kilowatt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 1e3,
    });

    units.insert("MW".to_string(), BaseUnit {
        symbol: "MW".to_string(),
        name: "megawatt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 1e6,
    });

    units.insert("GW".to_string(), BaseUnit {
        symbol: "GW".to_string(),
        name: "gigawatt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 1e9,
    });

    units.insert("mW".to_string(), BaseUnit {
        symbol: "mW".to_string(),
        name: "milliwatt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("hp".to_string(), BaseUnit {
        symbol: "hp".to_string(),
        name: "horsepower".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 745.7,
    });

    units.insert("PS".to_string(), BaseUnit {
        symbol: "PS".to_string(),
        name: "metric horsepower".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 735.5,
    });

    units.insert("erg/s".to_string(), BaseUnit {
        symbol: "erg/s".to_string(),
        name: "erg per second".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 1e-7,
    });

    units.insert("ft.lbf/min".to_string(), BaseUnit {
        symbol: "ft.lbf/min".to_string(),
        name: "foot-pound per minute".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 0.0225969667,
    });

    units.insert("BTU/h".to_string(), BaseUnit {
        symbol: "BTU/h".to_string(),
        name: "BTU per hour".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, ..Dimension::new() },
        si_factor: 0.29307107,
    });
}

// === CURRENT UNITS ===
fn register_current_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("mA".to_string(), BaseUnit {
        symbol: "mA".to_string(),
        name: "milliampere".to_string(),
        dimension: Dimension { current: 1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("μA".to_string(), BaseUnit {
        symbol: "μA".to_string(),
        name: "microampere".to_string(),
        dimension: Dimension { current: 1, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("kA".to_string(), BaseUnit {
        symbol: "kA".to_string(),
        name: "kiloampere".to_string(),
        dimension: Dimension { current: 1, ..Dimension::new() },
        si_factor: 1e3,
    });
}

// === AMOUNT UNITS ===
fn register_amount_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("mmol".to_string(), BaseUnit {
        symbol: "mmol".to_string(),
        name: "millimole".to_string(),
        dimension: Dimension { amount: 1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("μmol".to_string(), BaseUnit {
        symbol: "μmol".to_string(),
        name: "micromole".to_string(),
        dimension: Dimension { amount: 1, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("kmol".to_string(), BaseUnit {
        symbol: "kmol".to_string(),
        name: "kilomole".to_string(),
        dimension: Dimension { amount: 1, ..Dimension::new() },
        si_factor: 1e3,
    });
}

// === LUMINOUS INTENSITY UNITS ===
fn register_luminous_intensity_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("cd".to_string(), BaseUnit {
        symbol: "cd".to_string(),
        name: "candela".to_string(),
        dimension: Dimension { luminosity: 1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("mcd".to_string(), BaseUnit {
        symbol: "mcd".to_string(),
        name: "millicandela".to_string(),
        dimension: Dimension { luminosity: 1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("kcd".to_string(), BaseUnit {
        symbol: "kcd".to_string(),
        name: "kilocandela".to_string(),
        dimension: Dimension { luminosity: 1, ..Dimension::new() },
        si_factor: 1e3,
    });

    units.insert("Mcd".to_string(), BaseUnit {
        symbol: "Mcd".to_string(),
        name: "megacandela".to_string(),
        dimension: Dimension { luminosity: 1, ..Dimension::new() },
        si_factor: 1e6,
    });

    units.insert("cp".to_string(), BaseUnit {
        symbol: "cp".to_string(),
        name: "candlepower".to_string(),
        dimension: Dimension { luminosity: 1, ..Dimension::new() },
        si_factor: 0.981,
    });

    units.insert("hk".to_string(), BaseUnit {
        symbol: "hk".to_string(),
        name: "Hefnerkerze".to_string(),
        dimension: Dimension { luminosity: 1, ..Dimension::new() },
        si_factor: 0.903,
    });
}

// === FREQUENCY UNITS ===
fn register_frequency_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("Hz".to_string(), BaseUnit {
        symbol: "Hz".to_string(),
        name: "hertz".to_string(),
        dimension: Dimension { time: -1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("kHz".to_string(), BaseUnit {
        symbol: "kHz".to_string(),
        name: "kilohertz".to_string(),
        dimension: Dimension { time: -1, ..Dimension::new() },
        si_factor: 1e3,
    });

    units.insert("MHz".to_string(), BaseUnit {
        symbol: "MHz".to_string(),
        name: "megahertz".to_string(),
        dimension: Dimension { time: -1, ..Dimension::new() },
        si_factor: 1e6,
    });

    units.insert("GHz".to_string(), BaseUnit {
        symbol: "GHz".to_string(),
        name: "gigahertz".to_string(),
        dimension: Dimension { time: -1, ..Dimension::new() },
        si_factor: 1e9,
    });
}

// === VOLTAGE UNITS ===
fn register_voltage_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("V".to_string(), BaseUnit {
        symbol: "V".to_string(),
        name: "volt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, current: -1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("mV".to_string(), BaseUnit {
        symbol: "mV".to_string(),
        name: "millivolt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, current: -1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("kV".to_string(), BaseUnit {
        symbol: "kV".to_string(),
        name: "kilovolt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, current: -1, ..Dimension::new() },
        si_factor: 1e3,
    });

    units.insert("MV".to_string(), BaseUnit {
        symbol: "MV".to_string(),
        name: "megavolt".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, current: -1, ..Dimension::new() },
        si_factor: 1e6,
    });
}

// === RESISTANCE UNITS ===
fn register_resistance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("Ω".to_string(), BaseUnit {
        symbol: "Ω".to_string(),
        name: "ohm".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, current: -2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("mΩ".to_string(), BaseUnit {
        symbol: "mΩ".to_string(),
        name: "milliohm".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, current: -2, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("kΩ".to_string(), BaseUnit {
        symbol: "kΩ".to_string(),
        name: "kiloohm".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, current: -2, ..Dimension::new() },
        si_factor: 1e3,
    });

    units.insert("MΩ".to_string(), BaseUnit {
        symbol: "MΩ".to_string(),
        name: "megaohm".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -3, current: -2, ..Dimension::new() },
        si_factor: 1e6,
    });
}

// === CAPACITANCE UNITS ===
fn register_capacitance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("F".to_string(), BaseUnit {
        symbol: "F".to_string(),
        name: "farad".to_string(),
        dimension: Dimension { mass: -1, length: -2, time: 4, current: 2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("μF".to_string(), BaseUnit {
        symbol: "μF".to_string(),
        name: "microfarad".to_string(),
        dimension: Dimension { mass: -1, length: -2, time: 4, current: 2, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("nF".to_string(), BaseUnit {
        symbol: "nF".to_string(),
        name: "nanofarad".to_string(),
        dimension: Dimension { mass: -1, length: -2, time: 4, current: 2, ..Dimension::new() },
        si_factor: 1e-9,
    });

    units.insert("pF".to_string(), BaseUnit {
        symbol: "pF".to_string(),
        name: "picofarad".to_string(),
        dimension: Dimension { mass: -1, length: -2, time: 4, current: 2, ..Dimension::new() },
        si_factor: 1e-12,
    });
}

// === INDUCTANCE UNITS ===
fn register_inductance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("H".to_string(), BaseUnit {
        symbol: "H".to_string(),
        name: "henry".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, current: -2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("mH".to_string(), BaseUnit {
        symbol: "mH".to_string(),
        name: "millihenry".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, current: -2, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("μH".to_string(), BaseUnit {
        symbol: "μH".to_string(),
        name: "microhenry".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, current: -2, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("nH".to_string(), BaseUnit {
        symbol: "nH".to_string(),
        name: "nanohenry".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, current: -2, ..Dimension::new() },
        si_factor: 1e-9,
    });
}

// === CONDUCTANCE UNITS ===
fn register_conductance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("S".to_string(), BaseUnit {
        symbol: "S".to_string(),
        name: "siemens".to_string(),
        dimension: Dimension { mass: -1, length: -2, time: 3, current: 2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("mS".to_string(), BaseUnit {
        symbol: "mS".to_string(),
        name: "millisiemens".to_string(),
        dimension: Dimension { mass: -1, length: -2, time: 3, current: 2, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("μS".to_string(), BaseUnit {
        symbol: "μS".to_string(),
        name: "microsiemens".to_string(),
        dimension: Dimension { mass: -1, length: -2, time: 3, current: 2, ..Dimension::new() },
        si_factor: 1e-6,
    });
}

// === ELECTRICAL UNITS ===
#[allow(unused_variables)]
fn register_electrical_units(units: &mut HashMap<String, BaseUnit>) {
    // This function is kept for any remaining electrical units not covered by specific categories
    // Currently empty as all units have been moved to specific categories (voltage, resistance, etc.)
}

// === MAGNETIC FLUX DENSITY UNITS ===
fn register_magnetic_flux_density_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("T".to_string(), BaseUnit {
        symbol: "T".to_string(),
        name: "tesla".to_string(),
        dimension: Dimension { mass: 1, time: -2, current: -1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("mT".to_string(), BaseUnit {
        symbol: "mT".to_string(),
        name: "millitesla".to_string(),
        dimension: Dimension { mass: 1, time: -2, current: -1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("μT".to_string(), BaseUnit {
        symbol: "μT".to_string(),
        name: "microtesla".to_string(),
        dimension: Dimension { mass: 1, time: -2, current: -1, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("G".to_string(), BaseUnit {
        symbol: "G".to_string(),
        name: "gauss".to_string(),
        dimension: Dimension { mass: 1, time: -2, current: -1, ..Dimension::new() },
        si_factor: 1e-4,
    });

    units.insert("gamma".to_string(), BaseUnit {
        symbol: "gamma".to_string(),
        name: "gamma".to_string(),
        dimension: Dimension { mass: 1, time: -2, current: -1, ..Dimension::new() },
        si_factor: 1e-9,
    });
}

// === MAGNETIC FLUX UNITS ===
fn register_magnetic_flux_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("Wb".to_string(), BaseUnit {
        symbol: "Wb".to_string(),
        name: "weber".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, current: -1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("mWb".to_string(), BaseUnit {
        symbol: "mWb".to_string(),
        name: "milliweber".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, current: -1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("Mx".to_string(), BaseUnit {
        symbol: "Mx".to_string(),
        name: "maxwell".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, current: -1, ..Dimension::new() },
        si_factor: 1e-8,
    });

    units.insert("unit_pole".to_string(), BaseUnit {
        symbol: "unit_pole".to_string(),
        name: "unit pole".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, current: -1, ..Dimension::new() },
        si_factor: 1e-8,
    });

    units.insert("statWb".to_string(), BaseUnit {
        symbol: "statWb".to_string(),
        name: "statweber".to_string(),
        dimension: Dimension { mass: 1, length: 2, time: -2, current: -1, ..Dimension::new() },
        si_factor: 2.99792458e10,
    });
}

// === ELECTRIC CHARGE UNITS ===
fn register_electric_charge_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("C".to_string(), BaseUnit {
        symbol: "C".to_string(),
        name: "coulomb".to_string(),
        dimension: Dimension { time: 1, current: 1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("mC".to_string(), BaseUnit {
        symbol: "mC".to_string(),
        name: "millicoulomb".to_string(),
        dimension: Dimension { time: 1, current: 1, ..Dimension::new() },
        si_factor: 1e-3,
    });

    units.insert("μC".to_string(), BaseUnit {
        symbol: "μC".to_string(),
        name: "microcoulomb".to_string(),
        dimension: Dimension { time: 1, current: 1, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("nC".to_string(), BaseUnit {
        symbol: "nC".to_string(),
        name: "nanocoulomb".to_string(),
        dimension: Dimension { time: 1, current: 1, ..Dimension::new() },
        si_factor: 1e-9,
    });

    units.insert("pC".to_string(), BaseUnit {
        symbol: "pC".to_string(),
        name: "picocoulomb".to_string(),
        dimension: Dimension { time: 1, current: 1, ..Dimension::new() },
        si_factor: 1e-12,
    });

    units.insert("kC".to_string(), BaseUnit {
        symbol: "kC".to_string(),
        name: "kilocoulomb".to_string(),
        dimension: Dimension { time: 1, current: 1, ..Dimension::new() },
        si_factor: 1e3,
    });

    units.insert("MC".to_string(), BaseUnit {
        symbol: "MC".to_string(),
        name: "megacoulomb".to_string(),
        dimension: Dimension { time: 1, current: 1, ..Dimension::new() },
        si_factor: 1e6,
    });
}

// === RADIATION ACTIVITY UNITS ===
fn register_radiation_activity_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("Bq".to_string(), BaseUnit {
        symbol: "Bq".to_string(),
        name: "becquerel".to_string(),
        dimension: Dimension { time: -1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("Ci".to_string(), BaseUnit {
        symbol: "Ci".to_string(),
        name: "curie".to_string(),
        dimension: Dimension { time: -1, ..Dimension::new() },
        si_factor: 3.7e10,
    });
}

// === RADIATION DOSE UNITS ===
fn register_radiation_dose_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("Gy".to_string(), BaseUnit {
        symbol: "Gy".to_string(),
        name: "gray".to_string(),
        dimension: Dimension { length: 2, time: -2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("Sv".to_string(), BaseUnit {
        symbol: "Sv".to_string(),
        name: "sievert".to_string(),
        dimension: Dimension { length: 2, time: -2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("rem".to_string(), BaseUnit {
        symbol: "rem".to_string(),
        name: "roentgen equivalent man".to_string(),
        dimension: Dimension { length: 2, time: -2, ..Dimension::new() },
        si_factor: 0.01,
    });

    units.insert("rd".to_string(), BaseUnit {
        symbol: "rd".to_string(),
        name: "radiation absorbed dose".to_string(),
        dimension: Dimension { length: 2, time: -2, ..Dimension::new() },
        si_factor: 0.01,
    });
}

// === ILLUMINANCE UNITS ===
fn register_illuminance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("lx".to_string(), BaseUnit {
        symbol: "lx".to_string(),
        name: "lux".to_string(),
        dimension: Dimension { luminosity: 1, length: -2, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("fc".to_string(), BaseUnit {
        symbol: "fc".to_string(),
        name: "foot-candle".to_string(),
        dimension: Dimension { luminosity: 1, length: -2, ..Dimension::new() },
        si_factor: 10.764,
    });

    units.insert("ph".to_string(), BaseUnit {
        symbol: "ph".to_string(),
        name: "phot".to_string(),
        dimension: Dimension { luminosity: 1, length: -2, ..Dimension::new() },
        si_factor: 10000.0,
    });
}

// === DATA STORAGE UNITS ===
fn register_data_storage_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("byte".to_string(), BaseUnit {
        symbol: "byte".to_string(),
        name: "byte".to_string(),
        dimension: Dimension::new(), // Dimensionless information
        si_factor: 8.0, // 1 byte = 8 bits
    });

    units.insert("kB".to_string(), BaseUnit {
        symbol: "kB".to_string(),
        name: "kilobyte".to_string(),
        dimension: Dimension::new(),
        si_factor: 8000.0,
    });

    units.insert("MB".to_string(), BaseUnit {
        symbol: "MB".to_string(),
        name: "megabyte".to_string(),
        dimension: Dimension::new(),
        si_factor: 8e6,
    });

    units.insert("GB".to_string(), BaseUnit {
        symbol: "GB".to_string(),
        name: "gigabyte".to_string(),
        dimension: Dimension::new(),
        si_factor: 8e9,
    });

    units.insert("TB".to_string(), BaseUnit {
        symbol: "TB".to_string(),
        name: "terabyte".to_string(),
        dimension: Dimension::new(),
        si_factor: 8e12,
    });

    units.insert("PB".to_string(), BaseUnit {
        symbol: "PB".to_string(),
        name: "petabyte".to_string(),
        dimension: Dimension::new(),
        si_factor: 8e15,
    });

    units.insert("bit".to_string(), BaseUnit {
        symbol: "bit".to_string(),
        name: "bit".to_string(),
        dimension: Dimension::new(),
        si_factor: 1.0,
    });

    units.insert("Kibit".to_string(), BaseUnit {
        symbol: "Kibit".to_string(),
        name: "kibibit".to_string(),
        dimension: Dimension::new(),
        si_factor: 1024.0,
    });

    units.insert("Mibit".to_string(), BaseUnit {
        symbol: "Mibit".to_string(),
        name: "mebibit".to_string(),
        dimension: Dimension::new(),
        si_factor: 1048576.0,
    });

    units.insert("Gibit".to_string(), BaseUnit {
        symbol: "Gibit".to_string(),
        name: "gibibit".to_string(),
        dimension: Dimension::new(),
        si_factor: 1073741824.0,
    });

    units.insert("KiB".to_string(), BaseUnit {
        symbol: "KiB".to_string(),
        name: "kibibyte".to_string(),
        dimension: Dimension::new(),
        si_factor: 8192.0,
    });

    units.insert("MiB".to_string(), BaseUnit {
        symbol: "MiB".to_string(),
        name: "mebibyte".to_string(),
        dimension: Dimension::new(),
        si_factor: 8388608.0,
    });

    units.insert("GiB".to_string(), BaseUnit {
        symbol: "GiB".to_string(),
        name: "gibibyte".to_string(),
        dimension: Dimension::new(),
        si_factor: 8589934592.0,
    });
}

// === COMPUTING UNITS ===
fn register_computing_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("FLOPS".to_string(), BaseUnit {
        symbol: "FLOPS".to_string(),
        name: "floating point operations per second".to_string(),
        dimension: Dimension { time: -1, ..Dimension::new() },
        si_factor: 1.0,
    });

    units.insert("MIPS".to_string(), BaseUnit {
        symbol: "MIPS".to_string(),
        name: "million instructions per second".to_string(),
        dimension: Dimension { time: -1, ..Dimension::new() },
        si_factor: 1e6,
    });
}


// === TEXTILE UNITS ===
fn register_textile_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert("tex".to_string(), BaseUnit {
        symbol: "tex".to_string(),
        name: "tex".to_string(),
        dimension: Dimension { mass: 1, length: -1, ..Dimension::new() },
        si_factor: 1e-6,
    });

    units.insert("denier".to_string(), BaseUnit {
        symbol: "denier".to_string(),
        name: "denier".to_string(),
        dimension: Dimension { mass: 1, length: -1, ..Dimension::new() },
        si_factor: 1.111e-7,
    });
}