// SPDX-License-Identifier: GPL-3.0-or-later

use math_units::{format_number_with, NumberFormat, Quantity};
use serde::{Deserialize, Serialize};

/// Real scalar used by the evaluator.
pub type Real = f64;

/// Exact rational number reduced to lowest terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rational {
    pub numerator: i64,
    pub denominator: i64,
}

#[allow(clippy::should_implement_trait)]
impl Rational {
    pub fn new(numerator: i64, denominator: i64) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let sign = if denominator < 0 { -1 } else { 1 };
        let numerator = numerator.checked_mul(sign)?;
        let denominator = denominator.abs();
        let divisor = gcd(numerator.unsigned_abs(), denominator as u64) as i64;
        Some(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    pub fn integer(value: i64) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    pub fn from_integral_real(value: Real) -> Option<Self> {
        if !value.is_finite() || value.fract() != 0.0 {
            return None;
        }
        if value < i64::MIN as Real || value > i64::MAX as Real {
            return None;
        }
        Some(Self::integer(value as i64))
    }

    pub fn to_f64(self) -> Real {
        self.numerator as Real / self.denominator as Real
    }

    pub fn add(self, other: Self) -> Option<Self> {
        Self::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }

    pub fn sub(self, other: Self) -> Option<Self> {
        Self::new(
            self.numerator * other.denominator - other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }

    pub fn mul(self, other: Self) -> Option<Self> {
        Self::new(
            self.numerator * other.numerator,
            self.denominator * other.denominator,
        )
    }

    pub fn div(self, other: Self) -> Option<Self> {
        if other.numerator == 0 {
            return None;
        }
        Self::new(
            self.numerator * other.denominator,
            self.denominator * other.numerator,
        )
    }

    pub fn neg(self) -> Self {
        Self {
            numerator: -self.numerator,
            denominator: self.denominator,
        }
    }

    pub fn abs(self) -> Self {
        Self {
            numerator: self.numerator.abs(),
            denominator: self.denominator,
        }
    }

    pub fn powi(self, exponent: i32) -> Option<Self> {
        if exponent >= 0 {
            Self::new(
                self.numerator.pow(exponent as u32),
                self.denominator.pow(exponent as u32),
            )
        } else {
            if self.numerator == 0 {
                return None;
            }
            let power = (-exponent) as u32;
            Self::new(self.denominator.pow(power), self.numerator.pow(power))
        }
    }

    pub fn display(self) -> String {
        self.display_with_format(&NumberFormat::default())
    }

    pub fn display_with_format(self, format: &NumberFormat) -> String {
        if format.fractions == "decimal" {
            return format_number_with(self.to_f64(), format);
        }

        if self.denominator == 1 {
            self.numerator.to_string()
        } else {
            format!("{}/{}", self.numerator, self.denominator)
        }
    }
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
}

/// Complex number with real and imaginary `f64` components.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Complex {
    pub re: Real,
    pub im: Real,
}

#[allow(clippy::should_implement_trait)]
impl Complex {
    pub const fn new(re: Real, im: Real) -> Self {
        Self { re, im }
    }

    pub fn real(value: Real) -> Self {
        Self::new(value, 0.0)
    }

    pub fn is_real(self) -> bool {
        self.im.abs() < 1e-12
    }

    pub fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }

    pub fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }

    pub fn mul(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    pub fn div(self, other: Self) -> Option<Self> {
        let denominator = other.re * other.re + other.im * other.im;
        if denominator.abs() < 1e-12 {
            return None;
        }
        Some(Self::new(
            (self.re * other.re + self.im * other.im) / denominator,
            (self.im * other.re - self.re * other.im) / denominator,
        ))
    }

    pub fn neg(self) -> Self {
        Self::new(-self.re, -self.im)
    }

    pub fn abs(self) -> Real {
        self.re.hypot(self.im)
    }

    pub fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    pub fn powi(self, exponent: i32) -> Option<Self> {
        if exponent == 0 {
            return Some(Self::real(1.0));
        }
        let mut base = if exponent < 0 {
            Self::real(1.0).div(self)?
        } else {
            self
        };
        let mut power = exponent.unsigned_abs();
        let mut result = Self::real(1.0);
        while power > 0 {
            if power % 2 == 1 {
                result = result.mul(base);
            }
            power /= 2;
            if power > 0 {
                base = base.mul(base);
            }
        }
        Some(result)
    }

    pub fn sqrt(self) -> Self {
        if self.im.abs() < 1e-12 {
            if self.re < 0.0 {
                return Self::new(0.0, (-self.re).sqrt());
            }
            return Self::real(self.re.sqrt());
        }

        let magnitude = self.abs();
        let re = ((magnitude + self.re) / 2.0).sqrt();
        let im = ((magnitude - self.re) / 2.0).sqrt() * self.im.signum();
        Self::new(re, im)
    }

    pub fn display(self) -> String {
        self.display_with_format(&NumberFormat::default())
    }

    pub fn display_with_format(self, format: &NumberFormat) -> String {
        if self.is_real() {
            return format_number_with(self.re, format);
        }

        let imaginary = format_imaginary_with(self.im.abs(), format);
        if self.re.abs() < 1e-12 {
            if self.im < 0.0 {
                format!("-{imaginary}")
            } else {
                imaginary
            }
        } else {
            let sign = if self.im < 0.0 { "-" } else { "+" };
            format!("{} {sign} {imaginary}", format_number_with(self.re, format))
        }
    }
}

fn format_imaginary_with(value: Real, format: &NumberFormat) -> String {
    if (value - 1.0).abs() < 1e-12 {
        "i".into()
    } else {
        format!("{}i", format_number_with(value, format))
    }
}

/// Canonical value produced by the evaluator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Real(Real),
    Rational(Rational),
    Complex(Complex),
    Quantity(Quantity),
    /// Literal or symbolic text produced by the evaluator.
    Text(String),
    /// Numeric matrix stored by rows.
    Matrix(Vec<Vec<Real>>),
    /// A user-defined function definition.
    Function {
        name: String,
        params: Vec<String>,
    },
}

impl Value {
    pub fn real(value: Real) -> Self {
        Self::Real(value)
    }

    pub fn quantity(value: Real, unit: math_units::Unit) -> Self {
        Self::Quantity(Quantity::new(value, unit))
    }

    pub fn matrix(rows: Vec<Vec<Real>>) -> Result<Self, String> {
        if rows.is_empty() {
            return Err("matrix must have at least one row".into());
        }
        let columns = rows[0].len();
        if columns == 0 {
            return Err("matrix must have at least one column".into());
        }
        if rows.iter().any(|row| row.len() != columns) {
            return Err("matrix rows must have the same length".into());
        }
        Ok(Self::Matrix(rows))
    }

    /// Convert to a plain scalar when possible. Dimensionless quantities use
    /// their SI base value, so `sin(90°)` works as expected.
    pub fn scalar(&self) -> Option<Real> {
        match self {
            Self::Real(value) => Some(*value),
            Self::Rational(value) => Some(value.to_f64()),
            Self::Quantity(quantity) => quantity.scalar(),
            Self::Text(_) | Self::Complex(_) | Self::Matrix(_) | Self::Function { .. } => None,
        }
    }

    pub fn as_rational(&self) -> Option<Rational> {
        match self {
            Self::Rational(value) => Some(*value),
            Self::Real(value) => Rational::from_integral_real(*value),
            _ => None,
        }
    }

    pub fn as_complex(&self) -> Option<Complex> {
        match self {
            Self::Complex(value) => Some(*value),
            Self::Real(value) => Some(Complex::real(*value)),
            Self::Rational(value) => Some(Complex::real(value.to_f64())),
            _ => None,
        }
    }

    pub fn as_matrix(&self) -> Option<&Vec<Vec<Real>>> {
        match self {
            Self::Matrix(rows) => Some(rows),
            _ => None,
        }
    }

    pub fn as_quantity(&self) -> Option<&Quantity> {
        match self {
            Self::Quantity(quantity) => Some(quantity),
            _ => None,
        }
    }

    pub fn is_matrix(&self) -> bool {
        matches!(self, Self::Matrix(_))
    }

    pub fn convert_units(&mut self, preferences: &math_units::UnitPreferences) {
        if let Self::Quantity(quantity) = self {
            *quantity = math_units::convert_quantity(quantity, preferences);
        }
    }

    pub fn display_with_preferences(&self, preferences: &math_units::UnitPreferences) -> String {
        let mut value = self.clone();
        value.convert_units(preferences);
        value.display()
    }

    pub fn display_with_options(
        &self,
        preferences: &math_units::UnitPreferences,
        format: &NumberFormat,
    ) -> String {
        let mut value = self.clone();
        value.convert_units(preferences);
        value.display_with_format(format)
    }

    pub fn display(&self) -> String {
        self.display_with_format(&NumberFormat::default())
    }

    pub fn display_with_format(&self, format: &NumberFormat) -> String {
        match self {
            Self::Real(value) => format_number_with(*value, format),
            Self::Rational(value) => value.display_with_format(format),
            Self::Complex(value) => value.display_with_format(format),
            Self::Quantity(quantity) => quantity.display_with_format(format),
            Self::Text(value) => value.clone(),
            Self::Matrix(rows) => {
                let body = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|value| format_number_with(*value, format))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("[{body}]")
            }
            Self::Function { name, params } => format!("{name}({})", params.join(", ")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_real_and_matrix_values() {
        assert_eq!(Value::Real(2.0).display(), "2");
        assert_eq!(Value::Real(1.25).display(), "1.25");
        assert_eq!(
            Value::matrix(vec![vec![1.0, 2.0], vec![3.0, 4.0]])
                .unwrap()
                .display(),
            "[1, 2; 3, 4]"
        );
    }

    #[test]
    fn reduces_and_displays_rationals() {
        assert_eq!(Rational::new(2, 4).unwrap().display(), "1/2");
        assert_eq!(Rational::new(-2, -4).unwrap().display(), "1/2");
        assert_eq!(Rational::new(6, 3).unwrap().display(), "2");
        assert_eq!(
            Rational::new(1, 3)
                .unwrap()
                .add(Rational::new(1, 6).unwrap())
                .unwrap()
                .display(),
            "1/2"
        );
    }

    #[test]
    fn formats_and_calculates_complex_numbers() {
        assert_eq!(Complex::new(0.0, 1.0).display(), "i");
        assert_eq!(Complex::new(0.0, -1.0).display(), "-i");
        assert_eq!(Complex::new(1.0, 2.0).display(), "1 + 2i");
        assert_eq!(
            Complex::new(1.0, -2.0).mul(Complex::new(1.0, 2.0)),
            Complex::new(5.0, 0.0)
        );
        assert_eq!(Complex::new(-1.0, 0.0).sqrt(), Complex::new(0.0, 1.0));
    }

    #[test]
    fn formats_values_with_number_settings() {
        let format = NumberFormat::from_settings("auto", 4, true, "decimal");
        assert_eq!(
            Value::Rational(Rational::new(1, 3).unwrap()).display_with_format(&format),
            "0.3333"
        );
        assert_eq!(Value::Real(2.3).display_with_format(&format), "2.3");
        assert_eq!(
            Value::Complex(Complex::new(1.0, 2.0)).display_with_format(&format),
            "1 + 2i"
        );
        assert_eq!(
            Value::Matrix(vec![vec![1.23456, 2.0]]).display_with_format(&format),
            "[1.2346, 2]"
        );
    }
}
