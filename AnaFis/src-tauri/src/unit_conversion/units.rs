// src-tauri/src/unit_conversion/units.rs

use super::core::{BaseUnit, Dimension};
use std::collections::HashMap;
use std::f64::consts::PI;

/// Register all base units organized by category
#[allow(
    clippy::too_many_lines,
    reason = "Registration function calls many sub-registrations"
)]
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
#[allow(
    clippy::too_many_lines,
    reason = "Comprehensive list of all units by category"
)]
pub fn get_unit_categories() -> HashMap<String, Vec<String>> {
    let mut categories = HashMap::new();

    categories.insert(
        "length".to_owned(),
        vec![
            "m".to_owned(),
            "mm".to_owned(),
            "cm".to_owned(),
            "km".to_owned(),
            "in".to_owned(),
            "ft".to_owned(),
            "yd".to_owned(),
            "mi".to_owned(),
            "mil".to_owned(),
            "\u{3bc}m".to_owned(),
            "nm".to_owned(),
            "\u{c5}".to_owned(),
            "nmi".to_owned(),
            "fathom".to_owned(),
            "au".to_owned(),
            "ly".to_owned(),
            "pc".to_owned(),
            "cubit".to_owned(),
            "furlong".to_owned(),
            "league".to_owned(),
            "chain".to_owned(),
            "rod".to_owned(),
            "link".to_owned(),
            "point".to_owned(),
            "pica".to_owned(),
        ],
    );

    categories.insert(
        "mass".to_owned(),
        vec![
            "kg".to_owned(),
            "g".to_owned(),
            "mg".to_owned(),
            "\u{3bc}g".to_owned(),
            "lb".to_owned(),
            "oz".to_owned(),
            "u".to_owned(),
            "stone".to_owned(),
            "ton".to_owned(),
            "metric_ton".to_owned(),
            "grain".to_owned(),
        ],
    );

    categories.insert(
        "time".to_owned(),
        vec![
            "s".to_owned(),
            "min".to_owned(),
            "h".to_owned(),
            "day".to_owned(),
            "week".to_owned(),
            "year".to_owned(),
            "ns".to_owned(),
            "\u{3bc}s".to_owned(),
            "ms".to_owned(),
        ],
    );

    categories.insert(
        "temperature".to_owned(),
        vec![
            "K".to_owned(),
            "\u{b0}C".to_owned(),
            "\u{b0}F".to_owned(),
            "\u{b0}R".to_owned(),
            "\u{b0}R\u{e9}".to_owned(),
        ],
    );

    categories.insert(
        "current".to_owned(),
        vec![
            "A".to_owned(),
            "mA".to_owned(),
            "\u{3bc}A".to_owned(),
            "kA".to_owned(),
        ],
    );

    categories.insert(
        "amount".to_owned(),
        vec![
            "mol".to_owned(),
            "mmol".to_owned(),
            "\u{3bc}mol".to_owned(),
            "kmol".to_owned(),
        ],
    );

    categories.insert(
        "luminous_intensity".to_owned(),
        vec![
            "cd".to_owned(),
            "mcd".to_owned(),
            "kcd".to_owned(),
            "Mcd".to_owned(),
            "cp".to_owned(),
            "hk".to_owned(),
        ],
    );

    categories.insert(
        "angle".to_owned(),
        vec![
            "rad".to_owned(),
            "deg".to_owned(),
            "\u{b0}".to_owned(),
            "arcmin".to_owned(),
            "arcsec".to_owned(),
            "grad".to_owned(),
            "turn".to_owned(),
        ],
    );

    categories.insert(
        "area".to_owned(),
        vec![
            "m\u{b2}".to_owned(),
            "km\u{b2}".to_owned(),
            "cm\u{b2}".to_owned(),
            "mm\u{b2}".to_owned(),
            "in\u{b2}".to_owned(),
            "ft\u{b2}".to_owned(),
            "yd\u{b2}".to_owned(),
            "mi\u{b2}".to_owned(),
            "ha".to_owned(),
            "acre".to_owned(),
            "perch".to_owned(),
        ],
    );

    categories.insert(
        "volume".to_owned(),
        vec![
            "m\u{b3}".to_owned(),
            "L".to_owned(),
            "mL".to_owned(),
            "cm\u{b3}".to_owned(),
            "in\u{b3}".to_owned(),
            "ft\u{b3}".to_owned(),
            "yd\u{b3}".to_owned(),
            "gal".to_owned(),
            "qt".to_owned(),
            "pt".to_owned(),
            "cup".to_owned(),
            "fl_oz".to_owned(),
            "tbsp".to_owned(),
            "tsp".to_owned(),
            "bbl".to_owned(),
            "bushel".to_owned(),
            "imp_gal".to_owned(),
            "imp_qt".to_owned(),
            "imp_pt".to_owned(),
            "imp_fl_oz".to_owned(),
        ],
    );

    categories.insert(
        "velocity".to_owned(),
        vec![
            "m/s".to_owned(),
            "km/h".to_owned(),
            "mph".to_owned(),
            "fps".to_owned(),
            "c".to_owned(),
            "kn".to_owned(),
        ],
    );

    categories.insert(
        "acceleration".to_owned(),
        vec![
            "m/s\u{b2}".to_owned(),
            "ft/s\u{b2}".to_owned(),
            "g".to_owned(),
        ],
    );

    categories.insert(
        "force".to_owned(),
        vec![
            "N".to_owned(),
            "lbf".to_owned(),
            "kgf".to_owned(),
            "dyn".to_owned(),
        ],
    );

    categories.insert(
        "pressure".to_owned(),
        vec![
            "Pa".to_owned(),
            "kPa".to_owned(),
            "MPa".to_owned(),
            "GPa".to_owned(),
            "bar".to_owned(),
            "atm".to_owned(),
            "psi".to_owned(),
            "mmHg".to_owned(),
            "torr".to_owned(),
            "hPa".to_owned(),
            "mbar".to_owned(),
            "inHg".to_owned(),
            "bar_abs".to_owned(),
        ],
    );

    categories.insert(
        "energy".to_owned(),
        vec![
            "J".to_owned(),
            "kJ".to_owned(),
            "MJ".to_owned(),
            "GJ".to_owned(),
            "cal".to_owned(),
            "kcal".to_owned(),
            "BTU".to_owned(),
            "kWh".to_owned(),
            "Wh".to_owned(),
            "eV".to_owned(),
            "keV".to_owned(),
            "MeV".to_owned(),
            "GeV".to_owned(),
            "erg".to_owned(),
            "dyne_cm".to_owned(),
        ],
    );

    categories.insert(
        "power".to_owned(),
        vec![
            "W".to_owned(),
            "kW".to_owned(),
            "MW".to_owned(),
            "GW".to_owned(),
            "mW".to_owned(),
            "hp".to_owned(),
            "PS".to_owned(),
            "erg/s".to_owned(),
            "ft.lbf/min".to_owned(),
            "BTU/h".to_owned(),
        ],
    );

    categories.insert(
        "frequency".to_owned(),
        vec![
            "Hz".to_owned(),
            "kHz".to_owned(),
            "MHz".to_owned(),
            "GHz".to_owned(),
        ],
    );

    categories.insert(
        "voltage".to_owned(),
        vec![
            "V".to_owned(),
            "mV".to_owned(),
            "kV".to_owned(),
            "MV".to_owned(),
        ],
    );

    categories.insert(
        "resistance".to_owned(),
        vec![
            "\u{3a9}".to_owned(),
            "m\u{3a9}".to_owned(),
            "k\u{3a9}".to_owned(),
            "M\u{3a9}".to_owned(),
        ],
    );

    categories.insert(
        "capacitance".to_owned(),
        vec![
            "F".to_owned(),
            "\u{3bc}F".to_owned(),
            "nF".to_owned(),
            "pF".to_owned(),
        ],
    );

    categories.insert(
        "inductance".to_owned(),
        vec![
            "H".to_owned(),
            "mH".to_owned(),
            "\u{3bc}H".to_owned(),
            "nH".to_owned(),
        ],
    );

    categories.insert(
        "conductance".to_owned(),
        vec!["S".to_owned(), "mS".to_owned(), "\u{3bc}S".to_owned()],
    );

    categories.insert(
        "magnetic_flux_density".to_owned(),
        vec![
            "T".to_owned(),
            "mT".to_owned(),
            "\u{3bc}T".to_owned(),
            "G".to_owned(),
            "gamma".to_owned(),
        ],
    );

    categories.insert(
        "magnetic_flux".to_owned(),
        vec![
            "Wb".to_owned(),
            "mWb".to_owned(),
            "Mx".to_owned(),
            "unit_pole".to_owned(),
            "statWb".to_owned(),
        ],
    );

    categories.insert(
        "electric_charge".to_owned(),
        vec![
            "C".to_owned(),
            "mC".to_owned(),
            "\u{3bc}C".to_owned(),
            "nC".to_owned(),
            "pC".to_owned(),
            "kC".to_owned(),
            "MC".to_owned(),
        ],
    );

    categories.insert(
        "radiation_activity".to_owned(),
        vec!["Bq".to_owned(), "Ci".to_owned()],
    );

    categories.insert(
        "radiation_dose".to_owned(),
        vec![
            "Gy".to_owned(),
            "Sv".to_owned(),
            "rem".to_owned(),
            "rd".to_owned(),
        ],
    );

    categories.insert(
        "illuminance".to_owned(),
        vec!["lx".to_owned(), "fc".to_owned(), "ph".to_owned()],
    );

    categories.insert(
        "data_storage".to_owned(),
        vec![
            "byte".to_owned(),
            "kB".to_owned(),
            "MB".to_owned(),
            "GB".to_owned(),
            "TB".to_owned(),
            "PB".to_owned(),
            "bit".to_owned(),
            "Kibit".to_owned(),
            "Mibit".to_owned(),
            "Gibit".to_owned(),
            "KiB".to_owned(),
            "MiB".to_owned(),
            "GiB".to_owned(),
        ],
    );

    categories.insert(
        "computing".to_owned(),
        vec!["FLOPS".to_owned(), "MIPS".to_owned()],
    );

    categories.insert(
        "textile".to_owned(),
        vec!["tex".to_owned(), "denier".to_owned()],
    );

    categories
}

// === SI BASE UNITS ===
fn register_si_base_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "m".to_owned(),
        BaseUnit {
            symbol: "m".to_owned(),
            name: "meter".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "kg".to_owned(),
        BaseUnit {
            symbol: "kg".to_owned(),
            name: "kilogram".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "s".to_owned(),
        BaseUnit {
            symbol: "s".to_owned(),
            name: "second".to_owned(),
            dimension: Dimension {
                time: 1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "A".to_owned(),
        BaseUnit {
            symbol: "A".to_owned(),
            name: "ampere".to_owned(),
            dimension: Dimension {
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "K".to_owned(),
        BaseUnit {
            symbol: "K".to_owned(),
            name: "kelvin".to_owned(),
            dimension: Dimension {
                temperature: 1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "mol".to_owned(),
        BaseUnit {
            symbol: "mol".to_owned(),
            name: "mole".to_owned(),
            dimension: Dimension {
                amount: 1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );
}

// === LENGTH UNITS ===
#[allow(
    clippy::too_many_lines,
    reason = "Extensive registration of length units"
)]
fn register_length_units(units: &mut HashMap<String, BaseUnit>) {
    // Metric length units
    units.insert(
        "mm".to_owned(),
        BaseUnit {
            symbol: "mm".to_owned(),
            name: "millimeter".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "cm".to_owned(),
        BaseUnit {
            symbol: "cm".to_owned(),
            name: "centimeter".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1e-2,
        },
    );

    units.insert(
        "km".to_owned(),
        BaseUnit {
            symbol: "km".to_owned(),
            name: "kilometer".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1e3,
        },
    );

    units.insert(
        "\u{3bc}m".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}m".to_owned(),
            name: "micrometer".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "nm".to_owned(),
        BaseUnit {
            symbol: "nm".to_owned(),
            name: "nanometer".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1e-9,
        },
    );

    units.insert(
        "\u{c5}".to_owned(),
        BaseUnit {
            symbol: "\u{c5}".to_owned(),
            name: "angstrom".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1e-10,
        },
    );

    // Imperial/US length units
    units.insert(
        "in".to_owned(),
        BaseUnit {
            symbol: "in".to_owned(),
            name: "inch".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 0.0254,
        },
    );

    units.insert(
        "ft".to_owned(),
        BaseUnit {
            symbol: "ft".to_owned(),
            name: "foot".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 0.3048,
        },
    );

    units.insert(
        "yd".to_owned(),
        BaseUnit {
            symbol: "yd".to_owned(),
            name: "yard".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 0.9144,
        },
    );

    units.insert(
        "mi".to_owned(),
        BaseUnit {
            symbol: "mi".to_owned(),
            name: "mile".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1609.34,
        },
    );

    units.insert(
        "mil".to_owned(),
        BaseUnit {
            symbol: "mil".to_owned(),
            name: "mil".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 2.54e-5,
        },
    );

    // Maritime units
    units.insert(
        "nmi".to_owned(),
        BaseUnit {
            symbol: "nmi".to_owned(),
            name: "nautical mile".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1852.0,
        },
    );

    units.insert(
        "fathom".to_owned(),
        BaseUnit {
            symbol: "fathom".to_owned(),
            name: "fathom".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1.8288,
        },
    );

    // Astronomical units
    units.insert(
        "au".to_owned(),
        BaseUnit {
            symbol: "au".to_owned(),
            name: "astronomical unit".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 1.495_978_707e11,
        },
    );

    units.insert(
        "ly".to_owned(),
        BaseUnit {
            symbol: "ly".to_owned(),
            name: "light year".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 9.460_730_472_580_8e15,
        },
    );

    units.insert(
        "pc".to_owned(),
        BaseUnit {
            symbol: "pc".to_owned(),
            name: "parsec".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 3.0857e16,
        },
    );

    // Historical units
    units.insert(
        "cubit".to_owned(),
        BaseUnit {
            symbol: "cubit".to_owned(),
            name: "cubit".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 0.4572,
        },
    );

    units.insert(
        "furlong".to_owned(),
        BaseUnit {
            symbol: "furlong".to_owned(),
            name: "furlong".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 201.168,
        },
    );

    units.insert(
        "league".to_owned(),
        BaseUnit {
            symbol: "league".to_owned(),
            name: "league".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 4828.032,
        },
    );

    units.insert(
        "chain".to_owned(),
        BaseUnit {
            symbol: "chain".to_owned(),
            name: "chain".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 20.1168,
        },
    );

    units.insert(
        "rod".to_owned(),
        BaseUnit {
            symbol: "rod".to_owned(),
            name: "rod".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 5.0292,
        },
    );

    units.insert(
        "link".to_owned(),
        BaseUnit {
            symbol: "link".to_owned(),
            name: "surveyor's link".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 0.201_168,
        },
    );

    // Printing units
    units.insert(
        "point".to_owned(),
        BaseUnit {
            symbol: "point".to_owned(),
            name: "point".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 0.000_352_778,
        },
    );

    units.insert(
        "pica".to_owned(),
        BaseUnit {
            symbol: "pica".to_owned(),
            name: "pica".to_owned(),
            dimension: Dimension {
                length: 1,
                ..Dimension::new()
            },
            si_factor: 0.004_233_33,
        },
    );
}

// === MASS UNITS ===
#[allow(
    clippy::too_many_lines,
    reason = "Extensive registration of mass units"
)]
fn register_mass_units(units: &mut HashMap<String, BaseUnit>) {
    // Metric mass units
    units.insert(
        "g".to_owned(),
        BaseUnit {
            symbol: "g".to_owned(),
            name: "gram".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "mg".to_owned(),
        BaseUnit {
            symbol: "mg".to_owned(),
            name: "milligram".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "\u{3bc}g".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}g".to_owned(),
            name: "microgram".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 1e-9,
        },
    );

    // Imperial/US mass units
    units.insert(
        "lb".to_owned(),
        BaseUnit {
            symbol: "lb".to_owned(),
            name: "pound".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 0.453_592,
        },
    );

    units.insert(
        "oz".to_owned(),
        BaseUnit {
            symbol: "oz".to_owned(),
            name: "ounce".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 0.028_349_5,
        },
    );

    units.insert(
        "stone".to_owned(),
        BaseUnit {
            symbol: "stone".to_owned(),
            name: "stone".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 6.35029,
        },
    );

    units.insert(
        "ton".to_owned(),
        BaseUnit {
            symbol: "ton".to_owned(),
            name: "short ton".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 907.185,
        },
    );

    units.insert(
        "metric_ton".to_owned(),
        BaseUnit {
            symbol: "metric_ton".to_owned(),
            name: "metric ton".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 1000.0,
        },
    );

    units.insert(
        "grain".to_owned(),
        BaseUnit {
            symbol: "grain".to_owned(),
            name: "grain".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 6.4799e-5,
        },
    );

    // Atomic mass unit
    units.insert(
        "u".to_owned(),
        BaseUnit {
            symbol: "u".to_owned(),
            name: "atomic mass unit".to_owned(),
            dimension: Dimension {
                mass: 1,
                ..Dimension::new()
            },
            si_factor: 1.66054e-27,
        },
    );
}

// === TIME UNITS ===
fn register_time_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "ns".to_owned(),
        BaseUnit {
            symbol: "ns".to_owned(),
            name: "nanosecond".to_owned(),
            dimension: Dimension {
                time: 1,
                ..Dimension::new()
            },
            si_factor: 1e-9,
        },
    );

    units.insert(
        "\u{3bc}s".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}s".to_owned(),
            name: "microsecond".to_owned(),
            dimension: Dimension {
                time: 1,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "ms".to_owned(),
        BaseUnit {
            symbol: "ms".to_owned(),
            name: "millisecond".to_owned(),
            dimension: Dimension {
                time: 1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "min".to_owned(),
        BaseUnit {
            symbol: "min".to_owned(),
            name: "minute".to_owned(),
            dimension: Dimension {
                time: 1,
                ..Dimension::new()
            },
            si_factor: 60.0,
        },
    );

    units.insert(
        "h".to_owned(),
        BaseUnit {
            symbol: "h".to_owned(),
            name: "hour".to_owned(),
            dimension: Dimension {
                time: 1,
                ..Dimension::new()
            },
            si_factor: 3600.0,
        },
    );

    units.insert(
        "day".to_owned(),
        BaseUnit {
            symbol: "day".to_owned(),
            name: "day".to_owned(),
            dimension: Dimension {
                time: 1,
                ..Dimension::new()
            },
            si_factor: 86400.0,
        },
    );

    units.insert(
        "week".to_owned(),
        BaseUnit {
            symbol: "week".to_owned(),
            name: "week".to_owned(),
            dimension: Dimension {
                time: 1,
                ..Dimension::new()
            },
            si_factor: 604_800.0,
        },
    );

    units.insert(
        "year".to_owned(),
        BaseUnit {
            symbol: "year".to_owned(),
            name: "year".to_owned(),
            dimension: Dimension {
                time: 1,
                ..Dimension::new()
            },
            si_factor: 31_557_600.0, // Julian year
        },
    );
}

// === TEMPERATURE UNITS ===
fn register_temperature_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "\u{b0}C".to_owned(),
        BaseUnit {
            symbol: "\u{b0}C".to_owned(),
            name: "celsius".to_owned(),
            dimension: Dimension {
                temperature: 1,
                ..Dimension::new()
            },
            si_factor: 1.0, // Special handling in conversion logic
        },
    );

    units.insert(
        "\u{b0}F".to_owned(),
        BaseUnit {
            symbol: "\u{b0}F".to_owned(),
            name: "fahrenheit".to_owned(),
            dimension: Dimension {
                temperature: 1,
                ..Dimension::new()
            },
            si_factor: 5.0 / 9.0, // Special handling in conversion logic
        },
    );

    units.insert(
        "\u{b0}R".to_owned(),
        BaseUnit {
            symbol: "\u{b0}R".to_owned(),
            name: "rankine".to_owned(),
            dimension: Dimension {
                temperature: 1,
                ..Dimension::new()
            },
            si_factor: 5.0 / 9.0,
        },
    );

    units.insert(
        "\u{b0}R\u{e9}".to_owned(),
        BaseUnit {
            symbol: "\u{b0}R\u{e9}".to_owned(),
            name: "r\u{e9}aumur".to_owned(),
            dimension: Dimension {
                temperature: 1,
                ..Dimension::new()
            },
            si_factor: 1.25,
        },
    );
}

// === ANGLE UNITS ===
fn register_angle_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "rad".to_owned(),
        BaseUnit {
            symbol: "rad".to_owned(),
            name: "radian".to_owned(),
            dimension: Dimension::new(), // Dimensionless
            si_factor: 1.0,
        },
    );

    units.insert(
        "deg".to_owned(),
        BaseUnit {
            symbol: "deg".to_owned(),
            name: "degree".to_owned(),
            dimension: Dimension::new(), // Dimensionless
            si_factor: PI / 180.0,
        },
    );

    units.insert(
        "\u{b0}".to_owned(),
        BaseUnit {
            symbol: "\u{b0}".to_owned(),
            name: "degree".to_owned(),
            dimension: Dimension::new(), // Dimensionless
            si_factor: PI / 180.0,
        },
    );

    units.insert(
        "arcmin".to_owned(),
        BaseUnit {
            symbol: "arcmin".to_owned(),
            name: "arcminute".to_owned(),
            dimension: Dimension::new(), // Dimensionless
            si_factor: PI / 10800.0,
        },
    );

    units.insert(
        "arcsec".to_owned(),
        BaseUnit {
            symbol: "arcsec".to_owned(),
            name: "arcsecond".to_owned(),
            dimension: Dimension::new(), // Dimensionless
            si_factor: PI / 648_000.0,
        },
    );

    units.insert(
        "grad".to_owned(),
        BaseUnit {
            symbol: "grad".to_owned(),
            name: "gradian".to_owned(),
            dimension: Dimension::new(), // Dimensionless
            si_factor: PI / 200.0,
        },
    );

    units.insert(
        "turn".to_owned(),
        BaseUnit {
            symbol: "turn".to_owned(),
            name: "full turn".to_owned(),
            dimension: Dimension::new(), // Dimensionless
            si_factor: 2.0 * PI,
        },
    );
}

// === AREA UNITS ===
#[allow(
    clippy::too_many_lines,
    reason = "Extensive registration of area units"
)]
fn register_area_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "m\u{b2}".to_owned(),
        BaseUnit {
            symbol: "m\u{b2}".to_owned(),
            name: "square meter".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "km\u{b2}".to_owned(),
        BaseUnit {
            symbol: "km\u{b2}".to_owned(),
            name: "square kilometer".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 1e6,
        },
    );

    units.insert(
        "cm\u{b2}".to_owned(),
        BaseUnit {
            symbol: "cm\u{b2}".to_owned(),
            name: "square centimeter".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 1e-4,
        },
    );

    units.insert(
        "mm\u{b2}".to_owned(),
        BaseUnit {
            symbol: "mm\u{b2}".to_owned(),
            name: "square millimeter".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "in\u{b2}".to_owned(),
        BaseUnit {
            symbol: "in\u{b2}".to_owned(),
            name: "square inch".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 6.4516e-4,
        },
    );

    units.insert(
        "ft\u{b2}".to_owned(),
        BaseUnit {
            symbol: "ft\u{b2}".to_owned(),
            name: "square foot".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 0.092_903,
        },
    );

    units.insert(
        "yd\u{b2}".to_owned(),
        BaseUnit {
            symbol: "yd\u{b2}".to_owned(),
            name: "square yard".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 0.836_127,
        },
    );

    units.insert(
        "mi\u{b2}".to_owned(),
        BaseUnit {
            symbol: "mi\u{b2}".to_owned(),
            name: "square mile".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 2.59e6,
        },
    );

    units.insert(
        "ha".to_owned(),
        BaseUnit {
            symbol: "ha".to_owned(),
            name: "hectare".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 10000.0,
        },
    );

    units.insert(
        "acre".to_owned(),
        BaseUnit {
            symbol: "acre".to_owned(),
            name: "acre".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 4046.86,
        },
    );

    units.insert(
        "perch".to_owned(),
        BaseUnit {
            symbol: "perch".to_owned(),
            name: "perch".to_owned(),
            dimension: Dimension {
                length: 2,
                ..Dimension::new()
            },
            si_factor: 25.2929, // 1 perch = 30.25 square yards
        },
    );
}

// === VOLUME UNITS ===
#[allow(
    clippy::too_many_lines,
    reason = "Extensive registration of volume units"
)]
fn register_volume_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "m\u{b3}".to_owned(),
        BaseUnit {
            symbol: "m\u{b3}".to_owned(),
            name: "cubic meter".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "L".to_owned(),
        BaseUnit {
            symbol: "L".to_owned(),
            name: "liter".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "mL".to_owned(),
        BaseUnit {
            symbol: "mL".to_owned(),
            name: "milliliter".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "cm\u{b3}".to_owned(),
        BaseUnit {
            symbol: "cm\u{b3}".to_owned(),
            name: "cubic centimeter".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "in\u{b3}".to_owned(),
        BaseUnit {
            symbol: "in\u{b3}".to_owned(),
            name: "cubic inch".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 1.6387e-5,
        },
    );

    units.insert(
        "ft\u{b3}".to_owned(),
        BaseUnit {
            symbol: "ft\u{b3}".to_owned(),
            name: "cubic foot".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.028_316_8,
        },
    );

    units.insert(
        "yd\u{b3}".to_owned(),
        BaseUnit {
            symbol: "yd\u{b3}".to_owned(),
            name: "cubic yard".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.764_555,
        },
    );

    // US Liquid volume
    units.insert(
        "gal".to_owned(),
        BaseUnit {
            symbol: "gal".to_owned(),
            name: "gallon".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.003_785_41,
        },
    );

    units.insert(
        "qt".to_owned(),
        BaseUnit {
            symbol: "qt".to_owned(),
            name: "quart".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.000_946_353,
        },
    );

    units.insert(
        "pt".to_owned(),
        BaseUnit {
            symbol: "pt".to_owned(),
            name: "pint".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.000_473_176,
        },
    );

    units.insert(
        "cup".to_owned(),
        BaseUnit {
            symbol: "cup".to_owned(),
            name: "cup".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.000_236_588,
        },
    );

    units.insert(
        "fl_oz".to_owned(),
        BaseUnit {
            symbol: "fl_oz".to_owned(),
            name: "fluid ounce".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 2.9574e-5,
        },
    );

    units.insert(
        "tbsp".to_owned(),
        BaseUnit {
            symbol: "tbsp".to_owned(),
            name: "tablespoon".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 1.4787e-5,
        },
    );

    units.insert(
        "tsp".to_owned(),
        BaseUnit {
            symbol: "tsp".to_owned(),
            name: "teaspoon".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 4.9289e-6,
        },
    );

    units.insert(
        "bbl".to_owned(),
        BaseUnit {
            symbol: "bbl".to_owned(),
            name: "barrel".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.158_987,
        },
    );

    units.insert(
        "bushel".to_owned(),
        BaseUnit {
            symbol: "bushel".to_owned(),
            name: "bushel".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.035_239_1,
        },
    );

    // Imperial volume
    units.insert(
        "imp_gal".to_owned(),
        BaseUnit {
            symbol: "imp_gal".to_owned(),
            name: "imperial gallon".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.004_546_09,
        },
    );

    units.insert(
        "imp_qt".to_owned(),
        BaseUnit {
            symbol: "imp_qt".to_owned(),
            name: "imperial quart".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.001_136_52,
        },
    );

    units.insert(
        "imp_pt".to_owned(),
        BaseUnit {
            symbol: "imp_pt".to_owned(),
            name: "imperial pint".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 0.000_568_261,
        },
    );

    units.insert(
        "imp_fl_oz".to_owned(),
        BaseUnit {
            symbol: "imp_fl_oz".to_owned(),
            name: "imperial fluid ounce".to_owned(),
            dimension: Dimension {
                length: 3,
                ..Dimension::new()
            },
            si_factor: 2.84131e-5,
        },
    );
}

// === VELOCITY UNITS ===
fn register_velocity_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "m/s".to_owned(),
        BaseUnit {
            symbol: "m/s".to_owned(),
            name: "meter per second".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "km/h".to_owned(),
        BaseUnit {
            symbol: "km/h".to_owned(),
            name: "kilometer per hour".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -1,
                ..Dimension::new()
            },
            si_factor: 1.0 / 3.6,
        },
    );

    units.insert(
        "mph".to_owned(),
        BaseUnit {
            symbol: "mph".to_owned(),
            name: "mile per hour".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -1,
                ..Dimension::new()
            },
            si_factor: 0.44704,
        },
    );

    units.insert(
        "fps".to_owned(),
        BaseUnit {
            symbol: "fps".to_owned(),
            name: "foot per second".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -1,
                ..Dimension::new()
            },
            si_factor: 0.3048,
        },
    );

    units.insert(
        "c".to_owned(),
        BaseUnit {
            symbol: "c".to_owned(),
            name: "speed of light".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -1,
                ..Dimension::new()
            },
            si_factor: 299_792_458.0,
        },
    );

    units.insert(
        "kn".to_owned(),
        BaseUnit {
            symbol: "kn".to_owned(),
            name: "knot".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -1,
                ..Dimension::new()
            },
            si_factor: 0.514_444,
        },
    );
}

// === ACCELERATION UNITS ===
fn register_acceleration_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "m/s\u{b2}".to_owned(),
        BaseUnit {
            symbol: "m/s\u{b2}".to_owned(),
            name: "meter per second squared".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "ft/s\u{b2}".to_owned(),
        BaseUnit {
            symbol: "ft/s\u{b2}".to_owned(),
            name: "foot per second squared".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 0.3048,
        },
    );

    // Note: 'g₀' for standard gravity acceleration (to avoid conflict with gram 'g')
    units.insert(
        "g\u{2080}".to_owned(),
        BaseUnit {
            symbol: "g\u{2080}".to_owned(),
            name: "standard gravity".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 9.80665,
        },
    );

    // Alternative notation for standard gravity
    units.insert(
        "g_0".to_owned(),
        BaseUnit {
            symbol: "g_0".to_owned(),
            name: "standard gravity".to_owned(),
            dimension: Dimension {
                length: 1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 9.80665,
        },
    );
}

// === FORCE UNITS ===
fn register_force_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "N".to_owned(),
        BaseUnit {
            symbol: "N".to_owned(),
            name: "newton".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "lbf".to_owned(),
        BaseUnit {
            symbol: "lbf".to_owned(),
            name: "pound-force".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 4.44822,
        },
    );

    units.insert(
        "kgf".to_owned(),
        BaseUnit {
            symbol: "kgf".to_owned(),
            name: "kilogram-force".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 9.80665,
        },
    );

    units.insert(
        "dyn".to_owned(),
        BaseUnit {
            symbol: "dyn".to_owned(),
            name: "dyne".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1e-5,
        },
    );
}

// === PRESSURE UNITS ===
#[allow(
    clippy::too_many_lines,
    reason = "Extensive registration of pressure units"
)]
fn register_pressure_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "Pa".to_owned(),
        BaseUnit {
            symbol: "Pa".to_owned(),
            name: "pascal".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "kPa".to_owned(),
        BaseUnit {
            symbol: "kPa".to_owned(),
            name: "kilopascal".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1000.0,
        },
    );

    units.insert(
        "MPa".to_owned(),
        BaseUnit {
            symbol: "MPa".to_owned(),
            name: "megapascal".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1_000_000.0,
        },
    );

    units.insert(
        "GPa".to_owned(),
        BaseUnit {
            symbol: "GPa".to_owned(),
            name: "gigapascal".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1_000_000_000.0,
        },
    );

    units.insert(
        "bar".to_owned(),
        BaseUnit {
            symbol: "bar".to_owned(),
            name: "bar".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1e5,
        },
    );

    units.insert(
        "atm".to_owned(),
        BaseUnit {
            symbol: "atm".to_owned(),
            name: "atmosphere".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 101_325.0,
        },
    );

    units.insert(
        "psi".to_owned(),
        BaseUnit {
            symbol: "psi".to_owned(),
            name: "pound per square inch".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 6894.76,
        },
    );

    units.insert(
        "mmHg".to_owned(),
        BaseUnit {
            symbol: "mmHg".to_owned(),
            name: "millimeter of mercury".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 133.322,
        },
    );

    units.insert(
        "torr".to_owned(),
        BaseUnit {
            symbol: "torr".to_owned(),
            name: "torr".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 133.322,
        },
    );

    units.insert(
        "hPa".to_owned(),
        BaseUnit {
            symbol: "hPa".to_owned(),
            name: "hectopascal".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 100.0,
        },
    );

    units.insert(
        "mbar".to_owned(),
        BaseUnit {
            symbol: "mbar".to_owned(),
            name: "millibar".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 100.0,
        },
    );

    units.insert(
        "inHg".to_owned(),
        BaseUnit {
            symbol: "inHg".to_owned(),
            name: "inch of mercury".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 3386.39,
        },
    );

    units.insert(
        "bar_abs".to_owned(),
        BaseUnit {
            symbol: "bar_abs".to_owned(),
            name: "bar absolute".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 100_000.0,
        },
    );
}

// === ENERGY UNITS ===
#[allow(
    clippy::too_many_lines,
    reason = "Extensive registration of energy units"
)]
fn register_energy_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "J".to_owned(),
        BaseUnit {
            symbol: "J".to_owned(),
            name: "joule".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "kJ".to_owned(),
        BaseUnit {
            symbol: "kJ".to_owned(),
            name: "kilojoule".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1000.0,
        },
    );

    units.insert(
        "MJ".to_owned(),
        BaseUnit {
            symbol: "MJ".to_owned(),
            name: "megajoule".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1_000_000.0,
        },
    );

    units.insert(
        "GJ".to_owned(),
        BaseUnit {
            symbol: "GJ".to_owned(),
            name: "gigajoule".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1_000_000_000.0,
        },
    );

    units.insert(
        "cal".to_owned(),
        BaseUnit {
            symbol: "cal".to_owned(),
            name: "calorie".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 4.184,
        },
    );

    units.insert(
        "kcal".to_owned(),
        BaseUnit {
            symbol: "kcal".to_owned(),
            name: "kilocalorie".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 4184.0,
        },
    );

    units.insert(
        "BTU".to_owned(),
        BaseUnit {
            symbol: "BTU".to_owned(),
            name: "british thermal unit".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1055.06,
        },
    );

    units.insert(
        "kWh".to_owned(),
        BaseUnit {
            symbol: "kWh".to_owned(),
            name: "kilowatt-hour".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 3.6e6,
        },
    );

    units.insert(
        "Wh".to_owned(),
        BaseUnit {
            symbol: "Wh".to_owned(),
            name: "watt-hour".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 3600.0,
        },
    );

    units.insert(
        "eV".to_owned(),
        BaseUnit {
            symbol: "eV".to_owned(),
            name: "electron volt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.602_176_634e-19,
        },
    );

    units.insert(
        "keV".to_owned(),
        BaseUnit {
            symbol: "keV".to_owned(),
            name: "kiloelectron volt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.602_176_634e-16,
        },
    );

    units.insert(
        "MeV".to_owned(),
        BaseUnit {
            symbol: "MeV".to_owned(),
            name: "megaelectron volt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.602_176_634e-13,
        },
    );

    units.insert(
        "GeV".to_owned(),
        BaseUnit {
            symbol: "GeV".to_owned(),
            name: "gigaelectron volt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.602_176_634e-10,
        },
    );

    units.insert(
        "erg".to_owned(),
        BaseUnit {
            symbol: "erg".to_owned(),
            name: "erg".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1e-7,
        },
    );

    units.insert(
        "dyne_cm".to_owned(),
        BaseUnit {
            symbol: "dyne_cm".to_owned(),
            name: "dyne centimeter".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1e-7,
        },
    );
}

// === POWER UNITS ===
#[allow(
    clippy::too_many_lines,
    reason = "Extensive registration of power units"
)]
fn register_power_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "W".to_owned(),
        BaseUnit {
            symbol: "W".to_owned(),
            name: "watt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "kW".to_owned(),
        BaseUnit {
            symbol: "kW".to_owned(),
            name: "kilowatt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 1e3,
        },
    );

    units.insert(
        "MW".to_owned(),
        BaseUnit {
            symbol: "MW".to_owned(),
            name: "megawatt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 1e6,
        },
    );

    units.insert(
        "GW".to_owned(),
        BaseUnit {
            symbol: "GW".to_owned(),
            name: "gigawatt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 1e9,
        },
    );

    units.insert(
        "mW".to_owned(),
        BaseUnit {
            symbol: "mW".to_owned(),
            name: "milliwatt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "hp".to_owned(),
        BaseUnit {
            symbol: "hp".to_owned(),
            name: "horsepower".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 745.7,
        },
    );

    units.insert(
        "PS".to_owned(),
        BaseUnit {
            symbol: "PS".to_owned(),
            name: "metric horsepower".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 735.5,
        },
    );

    units.insert(
        "erg/s".to_owned(),
        BaseUnit {
            symbol: "erg/s".to_owned(),
            name: "erg per second".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 1e-7,
        },
    );

    units.insert(
        "ft.lbf/min".to_owned(),
        BaseUnit {
            symbol: "ft.lbf/min".to_owned(),
            name: "foot-pound per minute".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 0.022_596_966_7,
        },
    );

    units.insert(
        "BTU/h".to_owned(),
        BaseUnit {
            symbol: "BTU/h".to_owned(),
            name: "BTU per hour".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimension::new()
            },
            si_factor: 0.293_071_07,
        },
    );
}

// === CURRENT UNITS ===
fn register_current_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "mA".to_owned(),
        BaseUnit {
            symbol: "mA".to_owned(),
            name: "milliampere".to_owned(),
            dimension: Dimension {
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "\u{3bc}A".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}A".to_owned(),
            name: "microampere".to_owned(),
            dimension: Dimension {
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "kA".to_owned(),
        BaseUnit {
            symbol: "kA".to_owned(),
            name: "kiloampere".to_owned(),
            dimension: Dimension {
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1e3,
        },
    );
}

// === AMOUNT UNITS ===
fn register_amount_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "mmol".to_owned(),
        BaseUnit {
            symbol: "mmol".to_owned(),
            name: "millimole".to_owned(),
            dimension: Dimension {
                amount: 1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "\u{3bc}mol".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}mol".to_owned(),
            name: "micromole".to_owned(),
            dimension: Dimension {
                amount: 1,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "kmol".to_owned(),
        BaseUnit {
            symbol: "kmol".to_owned(),
            name: "kilomole".to_owned(),
            dimension: Dimension {
                amount: 1,
                ..Dimension::new()
            },
            si_factor: 1e3,
        },
    );
}

// === LUMINOUS INTENSITY UNITS ===
fn register_luminous_intensity_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "cd".to_owned(),
        BaseUnit {
            symbol: "cd".to_owned(),
            name: "candela".to_owned(),
            dimension: Dimension {
                luminosity: 1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "mcd".to_owned(),
        BaseUnit {
            symbol: "mcd".to_owned(),
            name: "millicandela".to_owned(),
            dimension: Dimension {
                luminosity: 1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "kcd".to_owned(),
        BaseUnit {
            symbol: "kcd".to_owned(),
            name: "kilocandela".to_owned(),
            dimension: Dimension {
                luminosity: 1,
                ..Dimension::new()
            },
            si_factor: 1e3,
        },
    );

    units.insert(
        "Mcd".to_owned(),
        BaseUnit {
            symbol: "Mcd".to_owned(),
            name: "megacandela".to_owned(),
            dimension: Dimension {
                luminosity: 1,
                ..Dimension::new()
            },
            si_factor: 1e6,
        },
    );

    units.insert(
        "cp".to_owned(),
        BaseUnit {
            symbol: "cp".to_owned(),
            name: "candlepower".to_owned(),
            dimension: Dimension {
                luminosity: 1,
                ..Dimension::new()
            },
            si_factor: 0.981,
        },
    );

    units.insert(
        "hk".to_owned(),
        BaseUnit {
            symbol: "hk".to_owned(),
            name: "Hefnerkerze".to_owned(),
            dimension: Dimension {
                luminosity: 1,
                ..Dimension::new()
            },
            si_factor: 0.903,
        },
    );
}

// === FREQUENCY UNITS ===
fn register_frequency_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "Hz".to_owned(),
        BaseUnit {
            symbol: "Hz".to_owned(),
            name: "hertz".to_owned(),
            dimension: Dimension {
                time: -1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "kHz".to_owned(),
        BaseUnit {
            symbol: "kHz".to_owned(),
            name: "kilohertz".to_owned(),
            dimension: Dimension {
                time: -1,
                ..Dimension::new()
            },
            si_factor: 1e3,
        },
    );

    units.insert(
        "MHz".to_owned(),
        BaseUnit {
            symbol: "MHz".to_owned(),
            name: "megahertz".to_owned(),
            dimension: Dimension {
                time: -1,
                ..Dimension::new()
            },
            si_factor: 1e6,
        },
    );

    units.insert(
        "GHz".to_owned(),
        BaseUnit {
            symbol: "GHz".to_owned(),
            name: "gigahertz".to_owned(),
            dimension: Dimension {
                time: -1,
                ..Dimension::new()
            },
            si_factor: 1e9,
        },
    );
}

// === VOLTAGE UNITS ===
fn register_voltage_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "V".to_owned(),
        BaseUnit {
            symbol: "V".to_owned(),
            name: "volt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "mV".to_owned(),
        BaseUnit {
            symbol: "mV".to_owned(),
            name: "millivolt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "kV".to_owned(),
        BaseUnit {
            symbol: "kV".to_owned(),
            name: "kilovolt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e3,
        },
    );

    units.insert(
        "MV".to_owned(),
        BaseUnit {
            symbol: "MV".to_owned(),
            name: "megavolt".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e6,
        },
    );
}

// === RESISTANCE UNITS ===
fn register_resistance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "\u{3a9}".to_owned(),
        BaseUnit {
            symbol: "\u{3a9}".to_owned(),
            name: "ohm".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                current: -2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "m\u{3a9}".to_owned(),
        BaseUnit {
            symbol: "m\u{3a9}".to_owned(),
            name: "milliohm".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                current: -2,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "k\u{3a9}".to_owned(),
        BaseUnit {
            symbol: "k\u{3a9}".to_owned(),
            name: "kiloohm".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                current: -2,
                ..Dimension::new()
            },
            si_factor: 1e3,
        },
    );

    units.insert(
        "M\u{3a9}".to_owned(),
        BaseUnit {
            symbol: "M\u{3a9}".to_owned(),
            name: "megaohm".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -3,
                current: -2,
                ..Dimension::new()
            },
            si_factor: 1e6,
        },
    );
}

// === CAPACITANCE UNITS ===
fn register_capacitance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "F".to_owned(),
        BaseUnit {
            symbol: "F".to_owned(),
            name: "farad".to_owned(),
            dimension: Dimension {
                mass: -1,
                length: -2,
                time: 4,
                current: 2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "\u{3bc}F".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}F".to_owned(),
            name: "microfarad".to_owned(),
            dimension: Dimension {
                mass: -1,
                length: -2,
                time: 4,
                current: 2,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "nF".to_owned(),
        BaseUnit {
            symbol: "nF".to_owned(),
            name: "nanofarad".to_owned(),
            dimension: Dimension {
                mass: -1,
                length: -2,
                time: 4,
                current: 2,
                ..Dimension::new()
            },
            si_factor: 1e-9,
        },
    );

    units.insert(
        "pF".to_owned(),
        BaseUnit {
            symbol: "pF".to_owned(),
            name: "picofarad".to_owned(),
            dimension: Dimension {
                mass: -1,
                length: -2,
                time: 4,
                current: 2,
                ..Dimension::new()
            },
            si_factor: 1e-12,
        },
    );
}

// === INDUCTANCE UNITS ===
fn register_inductance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "H".to_owned(),
        BaseUnit {
            symbol: "H".to_owned(),
            name: "henry".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                current: -2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "mH".to_owned(),
        BaseUnit {
            symbol: "mH".to_owned(),
            name: "millihenry".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                current: -2,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "\u{3bc}H".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}H".to_owned(),
            name: "microhenry".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                current: -2,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "nH".to_owned(),
        BaseUnit {
            symbol: "nH".to_owned(),
            name: "nanohenry".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                current: -2,
                ..Dimension::new()
            },
            si_factor: 1e-9,
        },
    );
}

// === CONDUCTANCE UNITS ===
fn register_conductance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "S".to_owned(),
        BaseUnit {
            symbol: "S".to_owned(),
            name: "siemens".to_owned(),
            dimension: Dimension {
                mass: -1,
                length: -2,
                time: 3,
                current: 2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "mS".to_owned(),
        BaseUnit {
            symbol: "mS".to_owned(),
            name: "millisiemens".to_owned(),
            dimension: Dimension {
                mass: -1,
                length: -2,
                time: 3,
                current: 2,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "\u{3bc}S".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}S".to_owned(),
            name: "microsiemens".to_owned(),
            dimension: Dimension {
                mass: -1,
                length: -2,
                time: 3,
                current: 2,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );
}

// === MAGNETIC FLUX DENSITY UNITS ===
fn register_magnetic_flux_density_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "T".to_owned(),
        BaseUnit {
            symbol: "T".to_owned(),
            name: "tesla".to_owned(),
            dimension: Dimension {
                mass: 1,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "mT".to_owned(),
        BaseUnit {
            symbol: "mT".to_owned(),
            name: "millitesla".to_owned(),
            dimension: Dimension {
                mass: 1,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "\u{3bc}T".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}T".to_owned(),
            name: "microtesla".to_owned(),
            dimension: Dimension {
                mass: 1,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "G".to_owned(),
        BaseUnit {
            symbol: "G".to_owned(),
            name: "gauss".to_owned(),
            dimension: Dimension {
                mass: 1,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e-4,
        },
    );

    units.insert(
        "gamma".to_owned(),
        BaseUnit {
            symbol: "gamma".to_owned(),
            name: "gamma".to_owned(),
            dimension: Dimension {
                mass: 1,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e-9,
        },
    );
}

// === MAGNETIC FLUX UNITS ===
fn register_magnetic_flux_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "Wb".to_owned(),
        BaseUnit {
            symbol: "Wb".to_owned(),
            name: "weber".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "mWb".to_owned(),
        BaseUnit {
            symbol: "mWb".to_owned(),
            name: "milliweber".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "Mx".to_owned(),
        BaseUnit {
            symbol: "Mx".to_owned(),
            name: "maxwell".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e-8,
        },
    );

    units.insert(
        "unit_pole".to_owned(),
        BaseUnit {
            symbol: "unit_pole".to_owned(),
            name: "unit pole".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 1e-8,
        },
    );

    units.insert(
        "statWb".to_owned(),
        BaseUnit {
            symbol: "statWb".to_owned(),
            name: "statweber".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: 2,
                time: -2,
                current: -1,
                ..Dimension::new()
            },
            si_factor: 2.997_924_58e10,
        },
    );
}

// === ELECTRIC CHARGE UNITS ===
fn register_electric_charge_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "C".to_owned(),
        BaseUnit {
            symbol: "C".to_owned(),
            name: "coulomb".to_owned(),
            dimension: Dimension {
                time: 1,
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "mC".to_owned(),
        BaseUnit {
            symbol: "mC".to_owned(),
            name: "millicoulomb".to_owned(),
            dimension: Dimension {
                time: 1,
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1e-3,
        },
    );

    units.insert(
        "\u{3bc}C".to_owned(),
        BaseUnit {
            symbol: "\u{3bc}C".to_owned(),
            name: "microcoulomb".to_owned(),
            dimension: Dimension {
                time: 1,
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "nC".to_owned(),
        BaseUnit {
            symbol: "nC".to_owned(),
            name: "nanocoulomb".to_owned(),
            dimension: Dimension {
                time: 1,
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1e-9,
        },
    );

    units.insert(
        "pC".to_owned(),
        BaseUnit {
            symbol: "pC".to_owned(),
            name: "picocoulomb".to_owned(),
            dimension: Dimension {
                time: 1,
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1e-12,
        },
    );

    units.insert(
        "kC".to_owned(),
        BaseUnit {
            symbol: "kC".to_owned(),
            name: "kilocoulomb".to_owned(),
            dimension: Dimension {
                time: 1,
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1e3,
        },
    );

    units.insert(
        "MC".to_owned(),
        BaseUnit {
            symbol: "MC".to_owned(),
            name: "megacoulomb".to_owned(),
            dimension: Dimension {
                time: 1,
                current: 1,
                ..Dimension::new()
            },
            si_factor: 1e6,
        },
    );
}

// === RADIATION ACTIVITY UNITS ===
fn register_radiation_activity_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "Bq".to_owned(),
        BaseUnit {
            symbol: "Bq".to_owned(),
            name: "becquerel".to_owned(),
            dimension: Dimension {
                time: -1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "Ci".to_owned(),
        BaseUnit {
            symbol: "Ci".to_owned(),
            name: "curie".to_owned(),
            dimension: Dimension {
                time: -1,
                ..Dimension::new()
            },
            si_factor: 3.7e10,
        },
    );
}

// === RADIATION DOSE UNITS ===
fn register_radiation_dose_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "Gy".to_owned(),
        BaseUnit {
            symbol: "Gy".to_owned(),
            name: "gray".to_owned(),
            dimension: Dimension {
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "Sv".to_owned(),
        BaseUnit {
            symbol: "Sv".to_owned(),
            name: "sievert".to_owned(),
            dimension: Dimension {
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "rem".to_owned(),
        BaseUnit {
            symbol: "rem".to_owned(),
            name: "roentgen equivalent man".to_owned(),
            dimension: Dimension {
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 0.01,
        },
    );

    units.insert(
        "rd".to_owned(),
        BaseUnit {
            symbol: "rd".to_owned(),
            name: "radiation absorbed dose".to_owned(),
            dimension: Dimension {
                length: 2,
                time: -2,
                ..Dimension::new()
            },
            si_factor: 0.01,
        },
    );
}

// === ILLUMINANCE UNITS ===
fn register_illuminance_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "lx".to_owned(),
        BaseUnit {
            symbol: "lx".to_owned(),
            name: "lux".to_owned(),
            dimension: Dimension {
                luminosity: 1,
                length: -2,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "fc".to_owned(),
        BaseUnit {
            symbol: "fc".to_owned(),
            name: "foot-candle".to_owned(),
            dimension: Dimension {
                luminosity: 1,
                length: -2,
                ..Dimension::new()
            },
            si_factor: 10.764,
        },
    );

    units.insert(
        "ph".to_owned(),
        BaseUnit {
            symbol: "ph".to_owned(),
            name: "phot".to_owned(),
            dimension: Dimension {
                luminosity: 1,
                length: -2,
                ..Dimension::new()
            },
            si_factor: 10000.0,
        },
    );
}

// === DATA STORAGE UNITS ===
#[allow(
    clippy::too_many_lines,
    reason = "Extensive registration of data storage units"
)]
fn register_data_storage_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "byte".to_owned(),
        BaseUnit {
            symbol: "byte".to_owned(),
            name: "byte".to_owned(),
            dimension: Dimension::new(), // Dimensionless information
            si_factor: 8.0,              // 1 byte = 8 bits
        },
    );

    units.insert(
        "kB".to_owned(),
        BaseUnit {
            symbol: "kB".to_owned(),
            name: "kilobyte".to_owned(),
            dimension: Dimension::new(),
            si_factor: 8000.0,
        },
    );

    units.insert(
        "MB".to_owned(),
        BaseUnit {
            symbol: "MB".to_owned(),
            name: "megabyte".to_owned(),
            dimension: Dimension::new(),
            si_factor: 8e6,
        },
    );

    units.insert(
        "GB".to_owned(),
        BaseUnit {
            symbol: "GB".to_owned(),
            name: "gigabyte".to_owned(),
            dimension: Dimension::new(),
            si_factor: 8e9,
        },
    );

    units.insert(
        "TB".to_owned(),
        BaseUnit {
            symbol: "TB".to_owned(),
            name: "terabyte".to_owned(),
            dimension: Dimension::new(),
            si_factor: 8e12,
        },
    );

    units.insert(
        "PB".to_owned(),
        BaseUnit {
            symbol: "PB".to_owned(),
            name: "petabyte".to_owned(),
            dimension: Dimension::new(),
            si_factor: 8e15,
        },
    );

    units.insert(
        "bit".to_owned(),
        BaseUnit {
            symbol: "bit".to_owned(),
            name: "bit".to_owned(),
            dimension: Dimension::new(),
            si_factor: 1.0,
        },
    );

    units.insert(
        "Kibit".to_owned(),
        BaseUnit {
            symbol: "Kibit".to_owned(),
            name: "kibibit".to_owned(),
            dimension: Dimension::new(),
            si_factor: 1024.0,
        },
    );

    units.insert(
        "Mibit".to_owned(),
        BaseUnit {
            symbol: "Mibit".to_owned(),
            name: "mebibit".to_owned(),
            dimension: Dimension::new(),
            si_factor: 1_048_576.0,
        },
    );

    units.insert(
        "Gibit".to_owned(),
        BaseUnit {
            symbol: "Gibit".to_owned(),
            name: "gibibit".to_owned(),
            dimension: Dimension::new(),
            si_factor: 1_073_741_824.0,
        },
    );

    units.insert(
        "KiB".to_owned(),
        BaseUnit {
            symbol: "KiB".to_owned(),
            name: "kibibyte".to_owned(),
            dimension: Dimension::new(),
            si_factor: 8192.0,
        },
    );

    units.insert(
        "MiB".to_owned(),
        BaseUnit {
            symbol: "MiB".to_owned(),
            name: "mebibyte".to_owned(),
            dimension: Dimension::new(),
            si_factor: 8_388_608.0,
        },
    );

    units.insert(
        "GiB".to_owned(),
        BaseUnit {
            symbol: "GiB".to_owned(),
            name: "gibibyte".to_owned(),
            dimension: Dimension::new(),
            si_factor: 8_589_934_592.0,
        },
    );
}

// === COMPUTING UNITS ===
fn register_computing_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "FLOPS".to_owned(),
        BaseUnit {
            symbol: "FLOPS".to_owned(),
            name: "floating point operations per second".to_owned(),
            dimension: Dimension {
                time: -1,
                ..Dimension::new()
            },
            si_factor: 1.0,
        },
    );

    units.insert(
        "MIPS".to_owned(),
        BaseUnit {
            symbol: "MIPS".to_owned(),
            name: "million instructions per second".to_owned(),
            dimension: Dimension {
                time: -1,
                ..Dimension::new()
            },
            si_factor: 1e6,
        },
    );
}

// === TEXTILE UNITS ===
fn register_textile_units(units: &mut HashMap<String, BaseUnit>) {
    units.insert(
        "tex".to_owned(),
        BaseUnit {
            symbol: "tex".to_owned(),
            name: "tex".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                ..Dimension::new()
            },
            si_factor: 1e-6,
        },
    );

    units.insert(
        "denier".to_owned(),
        BaseUnit {
            symbol: "denier".to_owned(),
            name: "denier".to_owned(),
            dimension: Dimension {
                mass: 1,
                length: -1,
                ..Dimension::new()
            },
            si_factor: 1.111e-7,
        },
    );
}
