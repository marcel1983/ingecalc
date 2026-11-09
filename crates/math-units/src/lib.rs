// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;
use thiserror::Error;

/// A dimension vector over the seven SI base dimensions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimension {
    pub length: i8,
    pub mass: i8,
    pub time: i8,
    pub current: i8,
    pub temperature: i8,
    pub amount: i8,
    pub luminosity: i8,
}

impl Dimension {
    pub const fn new(
        length: i8,
        mass: i8,
        time: i8,
        current: i8,
        temperature: i8,
        amount: i8,
        luminosity: i8,
    ) -> Self {
        Self {
            length,
            mass,
            time,
            current,
            temperature,
            amount,
            luminosity,
        }
    }

    pub fn is_dimensionless(&self) -> bool {
        *self == Self::default()
    }

    pub fn mul(&self, other: &Self) -> Self {
        Self {
            length: self.length.saturating_add(other.length),
            mass: self.mass.saturating_add(other.mass),
            time: self.time.saturating_add(other.time),
            current: self.current.saturating_add(other.current),
            temperature: self.temperature.saturating_add(other.temperature),
            amount: self.amount.saturating_add(other.amount),
            luminosity: self.luminosity.saturating_add(other.luminosity),
        }
    }

    pub fn div(&self, other: &Self) -> Self {
        Self {
            length: self.length.saturating_sub(other.length),
            mass: self.mass.saturating_sub(other.mass),
            time: self.time.saturating_sub(other.time),
            current: self.current.saturating_sub(other.current),
            temperature: self.temperature.saturating_sub(other.temperature),
            amount: self.amount.saturating_sub(other.amount),
            luminosity: self.luminosity.saturating_sub(other.luminosity),
        }
    }

    pub fn powi(&self, exponent: i32) -> Self {
        fn scale(value: i8, exponent: i32) -> i8 {
            let value = i32::from(value) * exponent;
            value.clamp(i32::from(i8::MIN), i32::from(i8::MAX)) as i8
        }

        Self {
            length: scale(self.length, exponent),
            mass: scale(self.mass, exponent),
            time: scale(self.time, exponent),
            current: scale(self.current, exponent),
            temperature: scale(self.temperature, exponent),
            amount: scale(self.amount, exponent),
            luminosity: scale(self.luminosity, exponent),
        }
    }
}

/// A unit definition loaded from the units catalogue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub symbol: String,
    pub factor: f64,
    pub offset: f64,
    pub dimension: Dimension,
}

impl Unit {
    pub fn new(symbol: impl Into<String>, factor: f64, offset: f64, dimension: Dimension) -> Self {
        Self {
            symbol: symbol.into(),
            factor,
            offset,
            dimension,
        }
    }

    pub fn is_linear(&self) -> bool {
        self.offset == 0.0
    }
}

/// A quantity whose `value` is stored in SI base units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: Unit,
}

#[derive(Debug, Error, PartialEq)]
pub enum UnitError {
    #[error("incompatible dimensions")]
    IncompatibleDimensions,
    #[error("offset units are not supported in this operation")]
    OffsetUnit,
    #[error("division by zero")]
    DivisionByZero,
    #[error("unit exponent must be an integer")]
    InvalidExponent,
    #[error("unit already defined: {0}")]
    DuplicateUnit(String),
}

impl Quantity {
    /// Build a quantity from a value expressed in `unit`.
    pub fn new(value: f64, unit: Unit) -> Self {
        Self {
            value: value * unit.factor + unit.offset,
            unit,
        }
    }

    /// Value expressed in the quantity's display unit.
    pub fn display_value(&self) -> f64 {
        (self.value - self.unit.offset) / self.unit.factor
    }

    pub fn display(&self) -> String {
        self.display_with_format(&NumberFormat::default())
    }

    pub fn display_with_format(&self, format: &NumberFormat) -> String {
        let number = format_number_with(self.display_value(), format);
        if self.unit.symbol.is_empty() {
            number
        } else if self.unit.symbol.starts_with('%') || self.unit.symbol.starts_with('°') {
            format!("{number}{}", self.unit.symbol)
        } else {
            format!("{number} {}", self.unit.symbol)
        }
    }

    pub fn is_dimensionless(&self) -> bool {
        self.unit.dimension.is_dimensionless()
    }

    /// Convert to a plain scalar when the quantity is dimensionless.
    pub fn scalar(&self) -> Option<f64> {
        self.is_dimensionless().then_some(self.value)
    }

    pub fn negate(&self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit.clone(),
        }
    }

    pub fn abs(&self) -> Self {
        Self {
            value: self.value.abs(),
            unit: self.unit.clone(),
        }
    }

    pub fn scale(&self, factor: f64) -> Self {
        let display = (self.value - self.unit.offset) * factor + self.unit.offset;
        Self {
            value: display,
            unit: self.unit.clone(),
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self, UnitError> {
        if self.unit.dimension != other.unit.dimension {
            return Err(UnitError::IncompatibleDimensions);
        }
        Ok(Self {
            value: self.value + other.value,
            unit: self.unit.clone(),
        })
    }

    pub fn sub(&self, other: &Self) -> Result<Self, UnitError> {
        if self.unit.dimension != other.unit.dimension {
            return Err(UnitError::IncompatibleDimensions);
        }
        Ok(Self {
            value: self.value - other.value,
            unit: self.unit.clone(),
        })
    }

    pub fn mul(&self, other: &Self) -> Result<Self, UnitError> {
        if !self.unit.is_linear() || !other.unit.is_linear() {
            return Err(UnitError::OffsetUnit);
        }

        let dimension = self.unit.dimension.mul(&other.unit.dimension);
        let (symbol, factor) = if dimension.is_dimensionless() {
            (String::new(), 1.0)
        } else {
            (
                multiply_symbols(&self.unit.symbol, &other.unit.symbol),
                self.unit.factor * other.unit.factor,
            )
        };

        Ok(Self {
            value: self.value * other.value,
            unit: Unit::new(symbol, factor, 0.0, dimension),
        })
    }

    pub fn div(&self, other: &Self) -> Result<Self, UnitError> {
        if other.value == 0.0 {
            return Err(UnitError::DivisionByZero);
        }
        if !self.unit.is_linear() || !other.unit.is_linear() {
            return Err(UnitError::OffsetUnit);
        }

        let dimension = self.unit.dimension.div(&other.unit.dimension);
        let (symbol, factor) = if dimension.is_dimensionless() {
            (String::new(), 1.0)
        } else {
            (
                divide_symbols(&self.unit.symbol, &other.unit.symbol),
                self.unit.factor / other.unit.factor,
            )
        };

        Ok(Self {
            value: self.value / other.value,
            unit: Unit::new(symbol, factor, 0.0, dimension),
        })
    }

    pub fn powi(&self, exponent: i32) -> Result<Self, UnitError> {
        if exponent == 0 {
            return Ok(Self {
                value: 1.0,
                unit: Unit::new("", 1.0, 0.0, Dimension::default()),
            });
        }
        if !self.unit.is_linear() {
            return Err(UnitError::OffsetUnit);
        }

        let symbol = pow_symbol(&self.unit.symbol, exponent);

        Ok(Self {
            value: self.value.powi(exponent),
            unit: Unit::new(
                symbol,
                self.unit.factor.powi(exponent),
                0.0,
                self.unit.dimension.powi(exponent),
            ),
        })
    }
}

/// Catalogue of SI and common units. Lookup is exact, with a few aliases.
#[derive(Debug, Clone, Default)]
pub struct UnitRegistry {
    units: HashMap<String, Unit>,
}

impl UnitRegistry {
    pub fn si() -> Self {
        let mut registry = Self::default();

        let length = Dimension::new(1, 0, 0, 0, 0, 0, 0);
        let mass = Dimension::new(0, 1, 0, 0, 0, 0, 0);
        let time = Dimension::new(0, 0, 1, 0, 0, 0, 0);
        let current = Dimension::new(0, 0, 0, 1, 0, 0, 0);
        let temperature = Dimension::new(0, 0, 0, 0, 1, 0, 0);
        let amount = Dimension::new(0, 0, 0, 0, 0, 1, 0);
        let luminosity = Dimension::new(0, 0, 0, 0, 0, 0, 1);

        registry.add("m", 1.0, 0.0, length);
        registry.add("kg", 1.0, 0.0, mass);
        registry.add("s", 1.0, 0.0, time);
        registry.add("A", 1.0, 0.0, current);
        registry.add("K", 1.0, 0.0, temperature);
        registry.add("mol", 1.0, 0.0, amount);
        registry.add("cd", 1.0, 0.0, luminosity);

        registry.add("Hz", 1.0, 0.0, Dimension::new(0, 0, -1, 0, 0, 0, 0));
        registry.add("N", 1.0, 0.0, Dimension::new(1, 1, -2, 0, 0, 0, 0));
        registry.add("Pa", 1.0, 0.0, Dimension::new(-1, 1, -2, 0, 0, 0, 0));
        registry.add("J", 1.0, 0.0, Dimension::new(2, 1, -2, 0, 0, 0, 0));
        registry.add("W", 1.0, 0.0, Dimension::new(2, 1, -3, 0, 0, 0, 0));
        registry.add("C", 1.0, 0.0, Dimension::new(0, 0, 1, 1, 0, 0, 0));
        registry.add("V", 1.0, 0.0, Dimension::new(2, 1, -3, -1, 0, 0, 0));
        registry.add("F", 1.0, 0.0, Dimension::new(-2, -1, 4, 2, 0, 0, 0));
        registry.add("Ω", 1.0, 0.0, Dimension::new(2, 1, -3, -2, 0, 0, 0));
        registry.add("S", 1.0, 0.0, Dimension::new(-2, -1, 3, 2, 0, 0, 0));
        registry.add("Wb", 1.0, 0.0, Dimension::new(2, 1, -2, -1, 0, 0, 0));
        registry.add("T", 1.0, 0.0, Dimension::new(0, 1, -2, -1, 0, 0, 0));
        registry.add("H", 1.0, 0.0, Dimension::new(2, 1, -2, -2, 0, 0, 0));
        registry.add("°C", 1.0, 273.15, temperature);
        registry.add("lm", 1.0, 0.0, luminosity);
        registry.add("lx", 1.0, 0.0, Dimension::new(-2, 0, 0, 0, 0, 0, 1));
        registry.add("Bq", 1.0, 0.0, Dimension::new(0, 0, -1, 0, 0, 0, 0));
        registry.add("Gy", 1.0, 0.0, Dimension::new(2, 0, -2, 0, 0, 0, 0));
        registry.add("Sv", 1.0, 0.0, Dimension::new(2, 0, -2, 0, 0, 0, 0));
        registry.add("kat", 1.0, 0.0, Dimension::new(0, 0, -1, 0, 0, 1, 0));

        registry.add("g", 0.001, 0.0, mass);
        registry.add("mg", 0.000_001, 0.0, mass);
        registry.add("µg", 0.000_000_001, 0.0, mass);
        registry.add("cm", 0.01, 0.0, length);
        registry.add("mm", 0.001, 0.0, length);
        registry.add("km", 1_000.0, 0.0, length);
        registry.add("µm", 0.000_001, 0.0, length);
        registry.add("L", 0.001, 0.0, Dimension::new(3, 0, 0, 0, 0, 0, 0));
        registry.add("mL", 0.000_001, 0.0, Dimension::new(3, 0, 0, 0, 0, 0, 0));
        registry.add("min", 60.0, 0.0, time);
        registry.add("h", 3_600.0, 0.0, time);
        registry.add("d", 86_400.0, 0.0, time);
        registry.add("bar", 100_000.0, 0.0, Dimension::new(-1, 1, -2, 0, 0, 0, 0));
        registry.add("atm", 101_325.0, 0.0, Dimension::new(-1, 1, -2, 0, 0, 0, 0));
        registry.add("psi", 6_894.757, 0.0, Dimension::new(-1, 1, -2, 0, 0, 0, 0));
        registry.add("kgf", 9.806_65, 0.0, Dimension::new(1, 1, -2, 0, 0, 0, 0));
        registry.add(
            "ksi",
            6_894_757.293_168,
            0.0,
            Dimension::new(-1, 1, -2, 0, 0, 0, 0),
        );
        registry.add("rad", 1.0, 0.0, Dimension::default());
        registry.add("sr", 1.0, 0.0, Dimension::default());
        registry.add("°", std::f64::consts::PI / 180.0, 0.0, Dimension::default());
        registry.add("%", 0.01, 0.0, Dimension::default());
        registry.add(
            "kWh",
            3_600_000.0,
            0.0,
            Dimension::new(2, 1, -2, 0, 0, 0, 0),
        );
        registry.add("in", 0.0254, 0.0, length);
        registry.add("ft", 0.3048, 0.0, length);
        registry.add("yd", 0.9144, 0.0, length);
        registry.add("mi", 1_609.344, 0.0, length);
        registry.add("lb", 0.453_592_37, 0.0, mass);
        registry.add("oz", 0.028_349_523_125, 0.0, mass);
        registry.add("°F", 5.0 / 9.0, 255.372_222_222_222_2, temperature);
        registry.add(
            "lbf",
            4.448_221_615_260_5,
            0.0,
            Dimension::new(1, 1, -2, 0, 0, 0, 0),
        );
        registry.add(
            "ft·lb",
            1.355_817_948_331_400_4,
            0.0,
            Dimension::new(2, 1, -2, 0, 0, 0, 0),
        );
        registry.add(
            "hp",
            745.699_871_582_270_2,
            0.0,
            Dimension::new(2, 1, -3, 0, 0, 0, 0),
        );
        registry.add("m²", 1.0, 0.0, Dimension::new(2, 0, 0, 0, 0, 0, 0));
        registry.add("m³", 1.0, 0.0, Dimension::new(3, 0, 0, 0, 0, 0, 0));
        registry.add(
            "ft²",
            0.092_903_04,
            0.0,
            Dimension::new(2, 0, 0, 0, 0, 0, 0),
        );
        registry.add(
            "ft³",
            0.028_316_846_592,
            0.0,
            Dimension::new(3, 0, 0, 0, 0, 0, 0),
        );
        registry.add(
            "in²",
            0.000_645_16,
            0.0,
            Dimension::new(2, 0, 0, 0, 0, 0, 0),
        );
        registry.add(
            "gal",
            0.003_785_411_784,
            0.0,
            Dimension::new(3, 0, 0, 0, 0, 0, 0),
        );
        registry.add(
            "qt",
            0.000_946_352_946,
            0.0,
            Dimension::new(3, 0, 0, 0, 0, 0, 0),
        );
        registry.add(
            "pt",
            0.000_473_176_473,
            0.0,
            Dimension::new(3, 0, 0, 0, 0, 0, 0),
        );
        registry.add(
            "acre",
            4_046.856_422_4,
            0.0,
            Dimension::new(2, 0, 0, 0, 0, 0, 0),
        );
        registry.add("m/s", 1.0, 0.0, Dimension::new(1, 0, -1, 0, 0, 0, 0));
        registry.add("m/s^2", 1.0, 0.0, Dimension::new(1, 0, -2, 0, 0, 0, 0));
        registry.add("m/s²", 1.0, 0.0, Dimension::new(1, 0, -2, 0, 0, 0, 0));
        registry.add(
            "km/h",
            1_000.0 / 3_600.0,
            0.0,
            Dimension::new(1, 0, -1, 0, 0, 0, 0),
        );
        registry.add("kg/m^3", 1.0, 0.0, Dimension::new(-3, 1, 0, 0, 0, 0, 0));
        registry.add("kg/m³", 1.0, 0.0, Dimension::new(-3, 1, 0, 0, 0, 0, 0));
        registry.add("N·m", 1.0, 0.0, Dimension::new(2, 1, -2, 0, 0, 0, 0));
        registry.add("N*m", 1.0, 0.0, Dimension::new(2, 1, -2, 0, 0, 0, 0));

        registry.alias("ohm", "Ω");
        registry.alias("Omega", "Ω");
        registry.alias("degC", "°C");
        registry.alias("um", "µm");
        registry.alias("μm", "µm");

        for alias in ["metro", "metros", "meter", "meters"] {
            registry.alias(alias, "m");
        }
        for alias in ["segundo", "segundos", "second", "seconds"] {
            registry.alias(alias, "s");
        }
        for alias in ["kilogramo", "kilogramos", "kilogram", "kilograms"] {
            registry.alias(alias, "kg");
        }
        for alias in ["gramo", "gramos", "gram", "grams"] {
            registry.alias(alias, "g");
        }
        for alias in ["litro", "litros", "liter", "liters"] {
            registry.alias(alias, "L");
        }
        for alias in ["pulgada", "pulgadas", "inch", "inches"] {
            registry.alias(alias, "in");
        }
        for alias in ["pie", "pies", "foot", "feet"] {
            registry.alias(alias, "ft");
        }
        for alias in ["yarda", "yardas", "yard", "yards"] {
            registry.alias(alias, "yd");
        }
        for alias in ["milla", "millas", "mile", "miles"] {
            registry.alias(alias, "mi");
        }
        for alias in ["libra", "libras", "pound", "pounds"] {
            registry.alias(alias, "lb");
        }
        for alias in ["onza", "onzas", "ounce", "ounces"] {
            registry.alias(alias, "oz");
        }
        for alias in ["galón", "galon", "galones", "gallon", "gallons"] {
            registry.alias(alias, "gal");
        }
        for alias in ["grado Celsius", "grados Celsius", "Celsius"] {
            registry.alias(alias, "°C");
        }
        for alias in ["grado Fahrenheit", "grados Fahrenheit", "Fahrenheit"] {
            registry.alias(alias, "°F");
        }
        for alias in ["libra fuerza", "libras fuerza", "pound force"] {
            registry.alias(alias, "lbf");
        }
        for alias in ["pie-libra", "pies-libra", "foot-pound"] {
            registry.alias(alias, "ft·lb");
        }
        for alias in ["caballo de fuerza", "caballos de fuerza", "horsepower"] {
            registry.alias(alias, "hp");
        }
        for alias in ["kelvin", "kelvins"] {
            registry.alias(alias, "K");
        }
        for alias in ["minuto", "minutos", "minute", "minutes"] {
            registry.alias(alias, "min");
        }
        for alias in ["hora", "horas", "hour", "hours"] {
            registry.alias(alias, "h");
        }

        registry
    }

    pub fn global() -> &'static Self {
        static REGISTRY: OnceLock<UnitRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::si)
    }

    pub fn lookup(&self, symbol: &str) -> Option<Unit> {
        self.units.get(symbol).cloned()
    }

    pub fn define(&mut self, unit: Unit) -> Result<(), UnitError> {
        if self.units.contains_key(&unit.symbol) {
            return Err(UnitError::DuplicateUnit(unit.symbol));
        }
        self.units.insert(unit.symbol.clone(), unit);
        Ok(())
    }

    pub fn contains(&self, symbol: &str) -> bool {
        self.units.contains_key(symbol)
    }

    fn add(&mut self, symbol: &str, factor: f64, offset: f64, dimension: Dimension) {
        self.units.insert(
            symbol.to_string(),
            Unit::new(symbol, factor, offset, dimension),
        );
    }

    fn alias(&mut self, alias: &str, canonical: &str) {
        if let Some(unit) = self.units.get(canonical).cloned() {
            self.units.insert(alias.to_string(), unit);
        }
    }
}

fn multiply_symbols(left: &str, right: &str) -> String {
    combine_symbols(left, right, false)
}

fn divide_symbols(left: &str, right: &str) -> String {
    combine_symbols(left, right, true)
}

fn pow_symbol(symbol: &str, exponent: i32) -> String {
    let mut factors = Vec::new();
    add_symbol_factors(symbol, 1, &mut factors);
    for (_, power) in &mut factors {
        *power *= exponent;
    }
    format_factors(&factors)
}

fn combine_symbols(left: &str, right: &str, divide: bool) -> String {
    let mut factors = Vec::new();
    add_symbol_factors(left, 1, &mut factors);
    add_symbol_factors(right, if divide { -1 } else { 1 }, &mut factors);
    format_factors(&factors)
}

fn add_symbol_factors(symbol: &str, sign: i32, factors: &mut Vec<(String, i32)>) {
    for term in symbol.split('·') {
        let mut parts = term.split('/');
        if let Some(numerator) = parts.next() {
            add_symbol_factor(numerator, sign, factors);
        }
        for denominator in parts {
            add_symbol_factor(denominator, -sign, factors);
        }
    }
}

fn add_symbol_factor(term: &str, sign: i32, factors: &mut Vec<(String, i32)>) {
    let term = term.trim();
    if term.is_empty() {
        return;
    }

    let (base, exponent) = match term.rsplit_once('^') {
        Some((base, exponent)) if !base.is_empty() => match exponent.parse::<i32>() {
            Ok(exponent) => (base, exponent),
            Err(_) => (term, 1),
        },
        _ => (term, 1),
    };
    let exponent = exponent * sign;

    if let Some((_, current)) = factors.iter_mut().find(|(name, _)| name == base) {
        *current += exponent;
    } else {
        factors.push((base.to_string(), exponent));
    }
}

fn format_factors(factors: &[(String, i32)]) -> String {
    let mut numerator = Vec::new();
    let mut denominator = Vec::new();
    for (base, exponent) in factors {
        if *exponent > 0 {
            numerator.push(format_factor(base, *exponent));
        } else if *exponent < 0 {
            denominator.push(format_factor(base, -*exponent));
        }
    }

    match (numerator.is_empty(), denominator.is_empty()) {
        (true, true) => String::new(),
        (false, true) => numerator.join("·"),
        (true, false) => format!("1/{}", denominator.join("·")),
        (false, false) => format!("{}/{}", numerator.join("·"), denominator.join("·")),
    }
}

fn format_factor(base: &str, exponent: i32) -> String {
    if exponent == 1 {
        base.to_string()
    } else {
        format!("{base}^{exponent}")
    }
}

/// Preferences for converting result quantities to a display unit system.
#[derive(Debug, Clone, Default)]
pub struct UnitPreferences {
    /// `metric`, `imperial`, `custom` or `original`.
    pub system: String,
    /// Per-dimension overrides used by the `custom` system.
    pub overrides: BTreeMap<String, String>,
}

impl UnitPreferences {
    pub fn new(system: impl Into<String>, overrides: BTreeMap<String, String>) -> Self {
        Self {
            system: system.into(),
            overrides,
        }
    }

    pub fn metric() -> Self {
        Self::new("metric", BTreeMap::new())
    }

    pub fn original() -> Self {
        Self::new("original", BTreeMap::new())
    }

    pub fn preferred_unit(&self, dimension: &Dimension) -> Option<Unit> {
        if dimension.is_dimensionless() || self.system == "original" {
            return None;
        }

        let registry = UnitRegistry::global();

        if let Some(key) = dimension_key(dimension) {
            if let Some(symbol) = self.overrides.get(key) {
                if let Some(unit) = registry.lookup(symbol) {
                    if unit.dimension == *dimension {
                        return Some(unit);
                    }
                }
            }
        }

        if let Some(symbol) = named_result_unit(&self.system, dimension) {
            if let Some(unit) = registry.lookup(symbol) {
                if unit.dimension == *dimension {
                    return Some(unit);
                }
            }
        }

        self.compound_unit(dimension)
    }

    fn compound_unit(&self, dimension: &Dimension) -> Option<Unit> {
        let imperial = self.system == "imperial";
        let registry = UnitRegistry::global();
        let base = |key: &str, metric: &str, imperial_symbol: &str| {
            if self.system == "custom" {
                if let Some(symbol) = self.overrides.get(key) {
                    if let Some(unit) = registry.lookup(symbol) {
                        if unit.is_linear() {
                            return symbol.clone();
                        }
                    }
                }
            }
            if imperial {
                imperial_symbol.to_string()
            } else {
                metric.to_string()
            }
        };

        let entries = [
            (base("length", "m", "ft"), dimension.length),
            (base("mass", "kg", "lb"), dimension.mass),
            (base("time", "s", "s"), dimension.time),
            (base("current", "A", "A"), dimension.current),
            ("K".to_string(), dimension.temperature),
            (base("amount", "mol", "mol"), dimension.amount),
            (base("luminosity", "cd", "cd"), dimension.luminosity),
        ];

        let mut factors = Vec::new();
        let mut factor = 1.0;
        for (symbol, exponent) in entries {
            if exponent == 0 {
                continue;
            }
            let unit = registry.lookup(&symbol)?;
            factor *= unit.factor.powi(i32::from(exponent));
            factors.push((symbol, i32::from(exponent)));
        }

        Some(Unit::new(format_factors(&factors), factor, 0.0, *dimension))
    }
}

/// Convert a quantity to the display unit selected by `preferences`.
pub fn convert_quantity(quantity: &Quantity, preferences: &UnitPreferences) -> Quantity {
    match preferences.preferred_unit(&quantity.unit.dimension) {
        Some(unit) => Quantity {
            value: quantity.value,
            unit,
        },
        None => quantity.clone(),
    }
}

fn dimension_key(dimension: &Dimension) -> Option<&'static str> {
    let candidates = [
        ("length", Dimension::new(1, 0, 0, 0, 0, 0, 0)),
        ("mass", Dimension::new(0, 1, 0, 0, 0, 0, 0)),
        ("time", Dimension::new(0, 0, 1, 0, 0, 0, 0)),
        ("current", Dimension::new(0, 0, 0, 1, 0, 0, 0)),
        ("temperature", Dimension::new(0, 0, 0, 0, 1, 0, 0)),
        ("amount", Dimension::new(0, 0, 0, 0, 0, 1, 0)),
        ("luminosity", Dimension::new(0, 0, 0, 0, 0, 0, 1)),
        ("area", Dimension::new(2, 0, 0, 0, 0, 0, 0)),
        ("volume", Dimension::new(3, 0, 0, 0, 0, 0, 0)),
        ("speed", Dimension::new(1, 0, -1, 0, 0, 0, 0)),
        ("acceleration", Dimension::new(1, 0, -2, 0, 0, 0, 0)),
        ("frequency", Dimension::new(0, 0, -1, 0, 0, 0, 0)),
        ("force", Dimension::new(1, 1, -2, 0, 0, 0, 0)),
        ("pressure", Dimension::new(-1, 1, -2, 0, 0, 0, 0)),
        ("energy", Dimension::new(2, 1, -2, 0, 0, 0, 0)),
        ("power", Dimension::new(2, 1, -3, 0, 0, 0, 0)),
        ("density", Dimension::new(-3, 1, 0, 0, 0, 0, 0)),
        ("charge", Dimension::new(0, 0, 1, 1, 0, 0, 0)),
        ("voltage", Dimension::new(2, 1, -3, -1, 0, 0, 0)),
        ("resistance", Dimension::new(2, 1, -3, -2, 0, 0, 0)),
        ("capacitance", Dimension::new(-2, -1, 4, 2, 0, 0, 0)),
        ("conductance", Dimension::new(-2, -1, 3, 2, 0, 0, 0)),
        ("magnetic_flux", Dimension::new(2, 1, -2, -1, 0, 0, 0)),
        (
            "magnetic_flux_density",
            Dimension::new(0, 1, -2, -1, 0, 0, 0),
        ),
        ("inductance", Dimension::new(2, 1, -2, -2, 0, 0, 0)),
        ("illuminance", Dimension::new(-2, 0, 0, 0, 0, 0, 1)),
        ("radioactivity", Dimension::new(0, 0, -1, 0, 0, 0, 0)),
        ("absorbed_dose", Dimension::new(2, 0, -2, 0, 0, 0, 0)),
        ("catalytic_activity", Dimension::new(0, 0, -1, 0, 0, 1, 0)),
    ];

    candidates
        .into_iter()
        .find_map(|(key, candidate)| (candidate == *dimension).then_some(key))
}

fn named_result_unit(system: &str, dimension: &Dimension) -> Option<&'static str> {
    let imperial = system == "imperial";
    let key = dimension_key(dimension)?;
    Some(match key {
        "temperature" => {
            if imperial {
                "°F"
            } else {
                "°C"
            }
        }
        "area" => {
            if imperial {
                "ft²"
            } else {
                "m²"
            }
        }
        "volume" => {
            if imperial {
                "ft³"
            } else {
                "m³"
            }
        }
        "frequency" => "Hz",
        "force" => {
            if imperial {
                "lbf"
            } else {
                "N"
            }
        }
        "pressure" => {
            if imperial {
                "psi"
            } else {
                "Pa"
            }
        }
        "energy" => {
            if imperial {
                "ft·lb"
            } else {
                "J"
            }
        }
        "power" => {
            if imperial {
                "hp"
            } else {
                "W"
            }
        }
        "charge" => "C",
        "voltage" => "V",
        "resistance" => "Ω",
        "capacitance" => "F",
        "conductance" => "S",
        "magnetic_flux" => "Wb",
        "magnetic_flux_density" => "T",
        "inductance" => "H",
        "illuminance" => "lx",
        "radioactivity" => "Bq",
        "absorbed_dose" => "Gy",
        "catalytic_activity" => "kat",
        _ => return None,
    })
}

/// Formatting options for numeric output.
#[derive(Debug, Clone)]
pub struct NumberFormat {
    /// `auto`, `fixed`, `scientific` or `engineering`.
    pub notation: String,
    /// Number of digits after the decimal separator.
    pub precision: usize,
    pub trim_zeros: bool,
    /// `auto` keeps exact fractions; `decimal` forces decimal output.
    pub fractions: String,
}

impl Default for NumberFormat {
    fn default() -> Self {
        Self {
            notation: "auto".into(),
            precision: 10,
            trim_zeros: true,
            fractions: "auto".into(),
        }
    }
}

impl NumberFormat {
    pub fn from_settings(
        notation: &str,
        precision: u32,
        trim_zeros: bool,
        fractions: &str,
    ) -> Self {
        Self {
            notation: notation.into(),
            precision: (precision as usize).min(12),
            trim_zeros,
            fractions: fractions.into(),
        }
    }

    fn notation_kind(&self) -> &str {
        match self.notation.as_str() {
            "fixed" | "decimal" => "fixed",
            "scientific" => "scientific",
            "engineering" => "engineering",
            _ => "auto",
        }
    }
}

pub fn format_number(value: f64) -> String {
    format_number_with(value, &NumberFormat::default())
}

pub fn format_number_with(value: f64, format: &NumberFormat) -> String {
    if !value.is_finite() {
        return value.to_string();
    }

    let decimals = format.precision.min(12);
    if value == 0.0 {
        return if format.trim_zeros {
            "0".into()
        } else {
            format!("{value:.decimals$}")
        };
    }

    match format.notation_kind() {
        "fixed" => format_fixed(value, decimals, format.trim_zeros),
        "scientific" => format_scientific(value, decimals, format.trim_zeros),
        "engineering" => format_engineering(value, decimals, format.trim_zeros),
        _ => {
            let threshold = 10f64.powi(-(decimals as i32)).max(1e-6);
            if value.abs() < threshold || value.abs() >= 1e12 {
                format_scientific(value, decimals, format.trim_zeros)
            } else {
                format_fixed(value, decimals, format.trim_zeros)
            }
        }
    }
}

fn format_fixed(value: f64, decimals: usize, trim_zeros: bool) -> String {
    let text = format!("{value:.decimals$}");
    if trim_zeros {
        trim_fixed(text)
    } else {
        text
    }
}

fn trim_fixed(text: String) -> String {
    if !text.contains('.') {
        return if text == "-0" { "0".into() } else { text };
    }

    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        "0".into()
    } else {
        trimmed.to_string()
    }
}

fn format_scientific(value: f64, decimals: usize, trim_zeros: bool) -> String {
    let exponent = value.abs().log10().floor() as i32;
    let mantissa = value / 10f64.powi(exponent);
    format!(
        "{}e{exponent}",
        format_fixed(mantissa, decimals, trim_zeros)
    )
}

fn format_engineering(value: f64, decimals: usize, trim_zeros: bool) -> String {
    if value == 0.0 {
        return format_fixed(value, decimals, trim_zeros);
    }

    let exponent = (value.abs().log10().floor() as i32).div_euclid(3) * 3;
    let mantissa = value / 10f64.powi(exponent);
    let text = format_fixed(mantissa, decimals, trim_zeros);
    if exponent == 0 {
        text
    } else {
        format!("{text}e{exponent}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_centimeters_to_meters() {
        let meter = UnitRegistry::global().lookup("m").expect("meter");
        let centimeter = UnitRegistry::global().lookup("cm").expect("centimeter");
        let left = Quantity::new(2.0, meter);
        let right = Quantity::new(30.0, centimeter);
        let sum = left.add(&right).expect("same dimension");
        assert_eq!(sum.display(), "2.3 m");
    }

    #[test]
    fn multiplies_and_divides_units() {
        let meter = UnitRegistry::global().lookup("m").expect("meter");
        let second = UnitRegistry::global().lookup("s").expect("second");
        let speed = Quantity::new(10.0, meter)
            .div(&Quantity::new(2.0, second))
            .expect("speed");
        assert_eq!(speed.display(), "5 m/s");
        assert_eq!(speed.unit.dimension, Dimension::new(1, 0, -1, 0, 0, 0, 0));
    }

    #[test]
    fn cancels_dimensions_in_ratios() {
        let meter = UnitRegistry::global().lookup("m").expect("meter");
        let centimeter = UnitRegistry::global().lookup("cm").expect("centimeter");
        let ratio = Quantity::new(1.0, meter)
            .div(&Quantity::new(1.0, centimeter))
            .expect("ratio");
        assert_eq!(ratio.display(), "100");
        assert!(ratio.is_dimensionless());
    }

    #[test]
    fn simplifies_compound_unit_symbols() {
        let meter = UnitRegistry::global().lookup("m").expect("meter");
        let second = UnitRegistry::global().lookup("s").expect("second");
        let speed = Quantity::new(3.0, meter.clone())
            .div(&Quantity::new(2.0, second.clone()))
            .expect("speed");
        assert_eq!(speed.display(), "1.5 m/s");

        let distance = speed.mul(&Quantity::new(2.0, second)).expect("distance");
        assert_eq!(distance.display(), "3 m");

        let area = Quantity::new(2.0, meter.clone())
            .mul(&Quantity::new(3.0, meter.clone()))
            .expect("area");
        assert_eq!(area.display(), "6 m^2");
        assert_eq!(
            area.div(&Quantity::new(2.0, meter))
                .expect("length")
                .display(),
            "3 m"
        );
    }

    #[test]
    fn powers_compound_unit_symbols() {
        let meter = UnitRegistry::global().lookup("m").expect("meter");
        let second = UnitRegistry::global().lookup("s").expect("second");
        let speed = Quantity::new(2.0, meter)
            .div(&Quantity::new(1.0, second))
            .expect("speed");

        assert_eq!(speed.powi(2).expect("squared").display(), "4 m^2/s^2");
        assert_eq!(speed.powi(-1).expect("inverse").display(), "0.5 s/m");
    }

    #[test]
    fn supports_celsius_offsets() {
        let celsius = UnitRegistry::global().lookup("°C").expect("celsius");
        let kelvin = UnitRegistry::global().lookup("K").expect("kelvin");
        let value = Quantity::new(20.0, celsius);
        assert_eq!(value.value, 293.15);
        let sum = value
            .add(&Quantity::new(5.0, kelvin))
            .expect("same dimension");
        assert_eq!(sum.display(), "25°C");
    }

    #[test]
    fn rejects_incompatible_dimensions() {
        let meter = UnitRegistry::global().lookup("m").expect("meter");
        let second = UnitRegistry::global().lookup("s").expect("second");
        assert_eq!(
            Quantity::new(1.0, meter).add(&Quantity::new(1.0, second)),
            Err(UnitError::IncompatibleDimensions)
        );
    }

    #[test]
    fn resolves_localized_aliases() {
        assert_eq!(
            UnitRegistry::global()
                .lookup("pulgada")
                .expect("localized inch")
                .factor,
            0.0254
        );
        assert_eq!(
            UnitRegistry::global()
                .lookup("metro")
                .expect("localized meter")
                .symbol,
            "m"
        );
        assert_eq!(
            UnitRegistry::global()
                .lookup("grado Celsius")
                .expect("localized Celsius")
                .symbol,
            "°C"
        );
        assert_eq!(
            UnitRegistry::global()
                .lookup("grado Fahrenheit")
                .expect("localized Fahrenheit")
                .symbol,
            "°F"
        );
    }

    #[test]
    fn converts_results_to_metric_and_imperial() {
        let inch = UnitRegistry::global().lookup("in").expect("inch");
        let metric = convert_quantity(
            &Quantity::new(6627.6535433071, inch),
            &UnitPreferences::metric(),
        );
        assert_eq!(metric.display(), "168.3424 m");

        let meter = UnitRegistry::global().lookup("m").expect("meter");
        let imperial = convert_quantity(
            &Quantity::new(2.3, meter),
            &UnitPreferences::new("imperial", std::collections::BTreeMap::new()),
        );
        assert_eq!(imperial.display(), "7.5459317585 ft");
    }

    #[test]
    fn converts_temperature_and_custom_speed() {
        let celsius = UnitRegistry::global().lookup("°C").expect("celsius");
        let imperial = convert_quantity(
            &Quantity::new(25.0, celsius),
            &UnitPreferences::new("imperial", std::collections::BTreeMap::new()),
        );
        assert_eq!(imperial.display(), "77°F");

        let meter = UnitRegistry::global().lookup("m").expect("meter");
        let second = UnitRegistry::global().lookup("s").expect("second");
        let speed = Quantity::new(5.0, meter)
            .div(&Quantity::new(1.0, second))
            .expect("speed");

        let mut overrides = std::collections::BTreeMap::new();
        overrides.insert("length".into(), "cm".into());
        overrides.insert("time".into(), "min".into());
        let custom = UnitPreferences::new("custom", overrides);
        assert_eq!(convert_quantity(&speed, &custom).display(), "30000 cm/min");
    }

    #[test]
    fn can_keep_original_result_units() {
        let inch = UnitRegistry::global().lookup("in").expect("inch");
        let original = convert_quantity(&Quantity::new(2.0, inch), &UnitPreferences::original());
        assert_eq!(original.display(), "2 in");
    }

    #[test]
    fn registers_imperial_energy_and_power_units() {
        let foot_pound = UnitRegistry::global().lookup("ft·lb").expect("foot pound");
        assert!((foot_pound.factor - 1.355_817_948_331_400_4).abs() < 1e-12);
        assert_eq!(foot_pound.dimension, Dimension::new(2, 1, -2, 0, 0, 0, 0));

        let horsepower = UnitRegistry::global().lookup("hp").expect("horsepower");
        assert!((horsepower.factor - 745.699_871_582_270_2).abs() < 1e-9);
    }

    #[test]
    fn formats_numbers_with_notation_and_decimals() {
        assert_eq!(format_number(0.000000009771908), "9.771908e-9");

        let auto = NumberFormat::from_settings("auto", 4, true, "auto");
        assert_eq!(format_number_with(6627.6535433071, &auto), "6627.6535");
        assert_eq!(format_number_with(0.000000009771908, &auto), "9.7719e-9");
        assert_eq!(format_number_with(1234567890.0, &auto), "1234567890");
        assert_eq!(format_number_with(1234567890123.0, &auto), "1.2346e12");
        assert_eq!(format_number_with(2.3, &auto), "2.3");

        let fixed = NumberFormat::from_settings("fixed", 2, false, "auto");
        assert_eq!(format_number_with(2.3, &fixed), "2.30");

        let scientific = NumberFormat::from_settings("scientific", 3, true, "auto");
        assert_eq!(format_number_with(12345.0, &scientific), "1.234e4");

        let engineering = NumberFormat::from_settings("engineering", 2, true, "auto");
        assert_eq!(format_number_with(0.0012, &engineering), "1.2e-3");
    }
}
