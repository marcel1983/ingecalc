use math_ast::{BinaryOp, Expr, UnaryOp};
use math_numeric::{Complex, Rational, Value};
use math_units::{Dimension, Quantity, Unit, UnitError, UnitRegistry};
pub use math_units::{NumberFormat, UnitPreferences};
use serde::Deserialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum EvalError {
    #[error("variable o símbolo no definido: {0}; para definirlo usa {0} := valor")]
    UnknownSymbol(String),
    #[error("division by zero")]
    DivisionByZero,
    #[error("unknown function: {0}")]
    UnknownFunction(String),
    #[error("invalid argument count for {0}")]
    InvalidArity(String),
    #[error("domain error: {0}")]
    Domain(String),
    #[error("unit error: {0}")]
    Unit(String),
    #[error("invalid matrix: {0}")]
    InvalidMatrix(String),
    #[error("index out of bounds: {0}")]
    IndexOutOfBounds(String),
    #[error("matrix is singular")]
    SingularMatrix,
    #[error("incompatible values: {0}")]
    IncompatibleValues(String),
    #[error("duplicate definition: {0}")]
    DuplicateDefinition(String),
    #[error("recursive function call: {0}")]
    Recursion(String),
    #[error("invalid loop: {0}")]
    InvalidLoop(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("dependency cycle")]
    DependencyCycle,
}

impl From<UnitError> for EvalError {
    fn from(error: UnitError) -> Self {
        Self::Unit(error.to_string())
    }
}

pub type Variables = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq)]
pub struct UserFunction {
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
    variables: HashMap<String, Value>,
    functions: HashMap<String, UserFunction>,
    units: HashMap<String, Unit>,
    call_stack: Vec<String>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.variables.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn function(&self, name: &str) -> Option<&UserFunction> {
        self.functions.get(name)
    }

    pub fn define_function(&mut self, name: impl Into<String>, params: Vec<String>, body: Expr) {
        self.functions
            .insert(name.into(), UserFunction { params, body });
    }

    pub fn lookup_unit(&self, name: &str) -> Option<Unit> {
        self.units
            .get(name)
            .cloned()
            .or_else(|| UnitRegistry::global().lookup(name))
    }

    pub fn define_unit(&mut self, unit: Unit) -> Result<(), EvalError> {
        if self.lookup_unit(&unit.symbol).is_some() {
            return Err(EvalError::DuplicateDefinition(unit.symbol));
        }
        self.units.insert(unit.symbol.clone(), unit);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DocumentEvaluation {
    pub id: String,
    pub result: Result<Value, EvalError>,
}

#[derive(Debug, Deserialize)]
struct EngineeringConstant {
    name: String,
    value: f64,
    unit: String,
    dimension: [i8; 7],
    #[allow(dead_code)]
    description: String,
    #[allow(dead_code)]
    category: String,
}

static ENGINEERING_CONSTANTS: OnceLock<HashMap<String, EngineeringConstant>> = OnceLock::new();

fn engineering_constants() -> &'static HashMap<String, EngineeringConstant> {
    ENGINEERING_CONSTANTS.get_or_init(|| {
        let constants: Vec<EngineeringConstant> = serde_json::from_str(include_str!(
            "../../../apps/desktop/src/lib/engineering-constants.json"
        ))
        .expect("engineering constants must be valid JSON");
        constants
            .into_iter()
            .map(|constant| (constant.name.clone(), constant))
            .collect()
    })
}

fn builtin_constant(name: &str) -> Option<Value> {
    let constant = engineering_constants().get(name)?;
    let dimension = Dimension::new(
        constant.dimension[0],
        constant.dimension[1],
        constant.dimension[2],
        constant.dimension[3],
        constant.dimension[4],
        constant.dimension[5],
        constant.dimension[6],
    );
    if dimension.is_dimensionless() && constant.unit.is_empty() {
        Some(Value::Real(constant.value))
    } else {
        let unit = Unit::new(constant.unit.clone(), 1.0, 0.0, dimension);
        Some(Value::Quantity(Quantity::new(constant.value, unit)))
    }
}

/// Evaluate an expression with a fresh environment.
pub fn evaluate(expr: &Expr) -> Result<Value, EvalError> {
    let mut environment = Environment::default();
    evaluate_with(expr, &mut environment)
}

/// Evaluate an expression with a mutable variable environment.
pub fn evaluate_with(expr: &Expr, environment: &mut Environment) -> Result<Value, EvalError> {
    match expr {
        Expr::Number(value) => Ok(Value::Real(*value)),
        Expr::Symbol(name) => resolve_symbol(name, environment),
        Expr::Unary { op, expr } => {
            let value = evaluate_with(expr, environment)?;
            Ok(match op {
                UnaryOp::Neg => negate_value(value),
                UnaryOp::Pos => value,
            })
        }
        Expr::Binary { op, lhs, rhs } => {
            let left = evaluate_with(lhs, environment)?;
            let right = evaluate_with(rhs, environment)?;
            match op {
                BinaryOp::Add => add_values(left, right),
                BinaryOp::Sub => sub_values(left, right),
                BinaryOp::Mul => mul_values(left, right),
                BinaryOp::Div => div_values(left, right),
                BinaryOp::Pow => pow_values(left, right),
                BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge
                | BinaryOp::Eq
                | BinaryOp::Ne => compare_values(*op, &left, &right),
            }
        }
        Expr::Call { name, args } => evaluate_call(name, args, environment),
        Expr::List(items) => evaluate_list(items, environment),
        Expr::Matrix(rows) => evaluate_matrix(rows, environment),
        Expr::Index { target, indices } => {
            let value = evaluate_with(target, environment)?;
            let indices = indices
                .iter()
                .map(|index| index_value(&evaluate_with(index, environment)?))
                .collect::<Result<Vec<_>, _>>()?;
            index_matrix(&value, &indices)
        }
        Expr::Assign { name, value } => {
            let value = evaluate_with(value, environment)?;
            environment.set(name.clone(), value.clone());
            Ok(value)
        }
        Expr::FunctionDef { name, params, body } => {
            environment.define_function(name.clone(), params.clone(), body.as_ref().clone());
            Ok(Value::Function {
                name: name.clone(),
                params: params.clone(),
            })
        }
        Expr::UnitDef { name, value } => {
            let value = evaluate_with(value, environment)?;
            let (factor, dimension) = match value {
                Value::Quantity(quantity) => {
                    if !quantity.unit.is_linear() {
                        return Err(EvalError::Unit(
                            "offset units cannot be used to define new units".into(),
                        ));
                    }
                    (quantity.value, quantity.unit.dimension)
                }
                Value::Real(value) => (value, Dimension::default()),
                Value::Rational(value) => (value.to_f64(), Dimension::default()),
                _ => {
                    return Err(EvalError::Unit(
                        "a unit definition expects a scalar or quantity".into(),
                    ))
                }
            };
            if !factor.is_finite() || factor <= 0.0 {
                return Err(EvalError::Unit(
                    "a unit factor must be a positive finite number".into(),
                ));
            }
            let unit = Unit::new(name.clone(), factor, 0.0, dimension);
            environment.define_unit(unit.clone())?;
            Ok(Value::Quantity(Quantity::new(1.0, unit)))
        }
    }
}

fn resolve_symbol(name: &str, environment: &Environment) -> Result<Value, EvalError> {
    if let Some(value) = environment.get(name) {
        return Ok(value.clone());
    }
    match name {
        "pi" | "π" => return Ok(Value::Real(std::f64::consts::PI)),
        "e" => return Ok(Value::Real(std::f64::consts::E)),
        "i" => return Ok(Value::Complex(Complex::new(0.0, 1.0))),
        "%" => return Ok(Value::Real(0.01)),
        _ => {}
    }
    if let Some(unit) = environment.lookup_unit(name) {
        return Ok(Value::Quantity(Quantity::new(1.0, unit)));
    }
    if let Some(value) = builtin_constant(name) {
        return Ok(value);
    }
    Err(EvalError::UnknownSymbol(name.to_string()))
}

fn evaluate_matrix(rows: &[Vec<Expr>], environment: &mut Environment) -> Result<Value, EvalError> {
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        let mut cells = Vec::with_capacity(row.len());
        for cell in row {
            let value = evaluate_with(cell, environment)?;
            cells.push(value_as_real(&value)?);
        }
        values.push(cells);
    }
    Value::matrix(values).map_err(EvalError::InvalidMatrix)
}

fn evaluate_list(items: &[Expr], environment: &mut Environment) -> Result<Value, EvalError> {
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        let value = evaluate_with(item, environment)?;
        values.push(value_as_real(&value)?);
    }
    Value::matrix(vec![values]).map_err(EvalError::InvalidMatrix)
}

fn index_matrix(value: &Value, indices: &[usize]) -> Result<Value, EvalError> {
    let Value::Matrix(rows) = value else {
        return Err(EvalError::Domain("indexing expects a matrix".into()));
    };
    let (row_count, column_count) = matrix_shape(rows)?;
    match indices {
        [row] => {
            if *row == 0 || *row > row_count {
                return Err(EvalError::IndexOutOfBounds(format!(
                    "row {row} is outside 1..={row_count}"
                )));
            }
            Ok(Value::Matrix(vec![rows[row - 1].clone()]))
        }
        [row, column] => {
            if *row == 0 || *row > row_count || *column == 0 || *column > column_count {
                return Err(EvalError::IndexOutOfBounds(format!(
                    "({row}, {column}) is outside {row_count}x{column_count}"
                )));
            }
            Ok(Value::Real(rows[row - 1][column - 1]))
        }
        _ => Err(EvalError::Domain(
            "matrix indexing accepts one or two indices".into(),
        )),
    }
}

fn negate_value(value: Value) -> Value {
    match value {
        Value::Real(value) => Value::Real(-value),
        Value::Rational(value) => Value::Rational(value.neg()),
        Value::Complex(value) => Value::Complex(value.neg()),
        Value::Quantity(quantity) => Value::Quantity(quantity.negate()),
        Value::Matrix(rows) => Value::Matrix(
            rows.into_iter()
                .map(|row| row.into_iter().map(|value| -value).collect())
                .collect(),
        ),
        Value::Text(value) => Value::Text(value),
        Value::Function { name, params } => Value::Function { name, params },
    }
}

fn add_values(left: Value, right: Value) -> Result<Value, EvalError> {
    if let (Value::Text(left), Value::Text(right)) = (&left, &right) {
        return Ok(Value::Text(format!("{left}{right}")));
    }
    if matches!(left, Value::Complex(_)) || matches!(right, Value::Complex(_)) {
        let (Some(left), Some(right)) = (left.as_complex(), right.as_complex()) else {
            return Err(incompatible("add", &left, &right));
        };
        return Ok(Value::Complex(left.add(right)));
    }
    if let (Some(left), Some(right)) = (exact_rational(&left), exact_rational(&right)) {
        if let Some(value) = left.add(right) {
            return Ok(Value::Rational(value));
        }
    }
    match (left, right) {
        (Value::Quantity(left), Value::Quantity(right)) => {
            Ok(Value::Quantity(left.add(&right)?))
        }
        (Value::Quantity(left), right) if left.is_dimensionless() => {
            Ok(Value::Quantity(Quantity {
                value: left.value + value_as_real(&right)?,
                unit: left.unit,
            }))
        }
        (left, Value::Quantity(right)) if right.is_dimensionless() => {
            Ok(Value::Quantity(Quantity {
                value: value_as_real(&left)? + right.value,
                unit: right.unit,
            }))
        }
        (Value::Matrix(left), Value::Matrix(right)) => {
            Ok(Value::Matrix(matrix_add(&left, &right)?))
        }
        (Value::Real(left), Value::Real(right)) => Ok(Value::Real(left + right)),
        (Value::Rational(left), Value::Real(right)) => {
            Ok(Value::Real(left.to_f64() + right))
        }
        (Value::Real(left), Value::Rational(right)) => {
            Ok(Value::Real(left + right.to_f64()))
        }
        (left, right) => Err(incompatible("add", &left, &right)),
    }
}

fn sub_values(left: Value, right: Value) -> Result<Value, EvalError> {
    if matches!(left, Value::Complex(_)) || matches!(right, Value::Complex(_)) {
        let (Some(left), Some(right)) = (left.as_complex(), right.as_complex()) else {
            return Err(incompatible("subtract", &left, &right));
        };
        return Ok(Value::Complex(left.sub(right)));
    }
    if let (Some(left), Some(right)) = (exact_rational(&left), exact_rational(&right)) {
        if let Some(value) = left.sub(right) {
            return Ok(Value::Rational(value));
        }
    }
    match (left, right) {
        (Value::Quantity(left), Value::Quantity(right)) => {
            Ok(Value::Quantity(left.sub(&right)?))
        }
        (Value::Quantity(left), right) if left.is_dimensionless() => {
            Ok(Value::Quantity(Quantity {
                value: left.value - value_as_real(&right)?,
                unit: left.unit,
            }))
        }
        (left, Value::Quantity(right)) if right.is_dimensionless() => {
            Ok(Value::Quantity(Quantity {
                value: value_as_real(&left)? - right.value,
                unit: right.unit,
            }))
        }
        (Value::Matrix(left), Value::Matrix(right)) => {
            Ok(Value::Matrix(matrix_sub(&left, &right)?))
        }
        (Value::Real(left), Value::Real(right)) => Ok(Value::Real(left - right)),
        (Value::Rational(left), Value::Real(right)) => {
            Ok(Value::Real(left.to_f64() - right))
        }
        (Value::Real(left), Value::Rational(right)) => {
            Ok(Value::Real(left - right.to_f64()))
        }
        (left, right) => Err(incompatible("subtract", &left, &right)),
    }
}

fn mul_values(left: Value, right: Value) -> Result<Value, EvalError> {
    if matches!(left, Value::Complex(_)) || matches!(right, Value::Complex(_)) {
        let (Some(left), Some(right)) = (left.as_complex(), right.as_complex()) else {
            return Err(incompatible("multiply", &left, &right));
        };
        return Ok(Value::Complex(left.mul(right)));
    }
    if let (Some(left), Some(right)) = (exact_rational(&left), exact_rational(&right)) {
        if let Some(value) = left.mul(right) {
            return Ok(Value::Rational(value));
        }
    }
    match (left, right) {
        (Value::Quantity(left), Value::Quantity(right)) => {
            Ok(Value::Quantity(left.mul(&right)?))
        }
        (Value::Quantity(quantity), scalar) if scalar.scalar().is_some() => {
            Ok(Value::Quantity(Quantity::new(
                quantity.display_value() * value_as_real(&scalar)?,
                quantity.unit,
            )))
        }
        (scalar, Value::Quantity(quantity)) if scalar.scalar().is_some() => {
            Ok(Value::Quantity(Quantity::new(
                value_as_real(&scalar)? * quantity.display_value(),
                quantity.unit,
            )))
        }
        (Value::Matrix(matrix), scalar) if scalar.scalar().is_some() => Ok(Value::Matrix(
            matrix_scalar(&matrix, value_as_real(&scalar)?),
        )),
        (scalar, Value::Matrix(matrix)) if scalar.scalar().is_some() => Ok(Value::Matrix(
            matrix_scalar(&matrix, value_as_real(&scalar)?),
        )),
        (Value::Matrix(left), Value::Matrix(right)) => {
            Ok(Value::Matrix(matrix_mul(&left, &right)?))
        }
        (Value::Real(left), Value::Real(right)) => Ok(Value::Real(left * right)),
        (Value::Rational(left), Value::Real(right)) => {
            Ok(Value::Real(left.to_f64() * right))
        }
        (Value::Real(left), Value::Rational(right)) => {
            Ok(Value::Real(left * right.to_f64()))
        }
        (left, right) => Err(incompatible("multiply", &left, &right)),
    }
}

fn div_values(left: Value, right: Value) -> Result<Value, EvalError> {
    if matches!(left, Value::Complex(_)) || matches!(right, Value::Complex(_)) {
        let (Some(left), Some(right)) = (left.as_complex(), right.as_complex()) else {
            return Err(incompatible("divide", &left, &right));
        };
        return left
            .div(right)
            .map(Value::Complex)
            .ok_or(EvalError::DivisionByZero);
    }
    if let (Some(left), Some(right)) = (as_rational(&left), as_rational(&right)) {
        if right.numerator == 0 {
            return Err(EvalError::DivisionByZero);
        }
        if let Some(value) = left.div(right) {
            return Ok(Value::Rational(value));
        }
    }
    match (left, right) {
        (Value::Quantity(left), Value::Quantity(right)) => {
            Ok(Value::Quantity(left.div(&right)?))
        }
        (Value::Quantity(quantity), scalar) if scalar.scalar().is_some() => {
            let divisor = value_as_real(&scalar)?;
            if divisor == 0.0 {
                return Err(EvalError::DivisionByZero);
            }
            Ok(Value::Quantity(Quantity::new(
                quantity.display_value() / divisor,
                quantity.unit,
            )))
        }
        (scalar, Value::Quantity(quantity)) if scalar.scalar().is_some() => {
            let numerator = value_as_real(&scalar)?;
            if quantity.value == 0.0 {
                return Err(EvalError::DivisionByZero);
            }
            Ok(Value::Quantity(Quantity {
                value: numerator / quantity.value,
                unit: Unit::new(
                    String::new(),
                    1.0 / quantity.unit.factor,
                    quantity.unit.offset,
                    quantity.unit.dimension.powi(-1),
                ),
            }))
        }
        (Value::Matrix(matrix), scalar) if scalar.scalar().is_some() => {
            let divisor = value_as_real(&scalar)?;
            if divisor == 0.0 {
                return Err(EvalError::DivisionByZero);
            }
            Ok(Value::Matrix(matrix_scalar(&matrix, 1.0 / divisor)))
        }
        (Value::Real(left), Value::Real(right)) => {
            if right == 0.0 {
                Err(EvalError::DivisionByZero)
            } else {
                Ok(Value::Real(left / right))
            }
        }
        (Value::Rational(left), Value::Real(right)) => {
            if right == 0.0 {
                Err(EvalError::DivisionByZero)
            } else {
                Ok(Value::Real(left.to_f64() / right))
            }
        }
        (Value::Real(left), Value::Rational(right)) => {
            if right.numerator == 0 {
                Err(EvalError::DivisionByZero)
            } else {
                Ok(Value::Real(left / right.to_f64()))
            }
        }
        (left, right) => Err(incompatible("divide", &left, &right)),
    }
}

fn pow_values(base: Value, exponent: Value) -> Result<Value, EvalError> {
    if matches!(base, Value::Complex(_)) || matches!(exponent, Value::Complex(_)) {
        let (Some(base), Some(exponent)) = (base.as_complex(), exponent.as_complex()) else {
            return Err(incompatible("raise", &base, &exponent));
        };
        return Ok(Value::Complex(complex_pow(base, exponent)));
    }
    if let (Some(base), Some(power)) = (as_rational(&base), value_as_real(&exponent).ok()) {
        if power.fract() == 0.0 && power.abs() <= 32.0 {
            if let Some(value) = base.powi(power as i32) {
                return Ok(Value::Rational(value));
            }
        }
    }
    if let (Value::Quantity(quantity), Ok(power)) = (&base, value_as_real(&exponent)) {
        if power.fract() == 0.0 && power.abs() <= 32.0 {
            return Ok(Value::Quantity(quantity.powi(power as i32)?));
        }
    }
    if let (Value::Matrix(matrix), Ok(power)) = (&base, value_as_real(&exponent)) {
        if power.fract() == 0.0 && power.abs() <= 32.0 {
            return Ok(Value::Matrix(matrix_power(matrix, power as i32)?));
        }
    }
    let base_value = value_as_real(&base)?;
    let exponent_value = value_as_real(&exponent)?;
    if base_value < 0.0 && exponent_value.fract() != 0.0 {
        return Ok(Value::Complex(complex_pow(
            Complex::new(base_value, 0.0),
            Complex::new(exponent_value, 0.0),
        )));
    }
    Ok(Value::Real(base_value.powf(exponent_value)))
}

fn complex_pow(base: Complex, exponent: Complex) -> Complex {
    if exponent.im.abs() < 1e-12 && exponent.re.fract() == 0.0 && exponent.re.abs() <= 32.0 {
        return base.powi(exponent.re as i32).unwrap_or(Complex::new(f64::NAN, f64::NAN));
    }
    let magnitude = base.abs();
    if magnitude < 1e-15 {
        return Complex::new(0.0, 0.0);
    }
    let angle = base.im.atan2(base.re);
    let factor = magnitude.powf(exponent.re) * (-exponent.im * angle).exp();
    let phase = exponent.im * magnitude.ln() + exponent.re * angle;
    Complex::new(factor * phase.cos(), factor * phase.sin())
}

fn exact_rational(value: &Value) -> Option<Rational> {
    match value {
        Value::Rational(value) => Some(*value),
        _ => None,
    }
}

fn as_rational(value: &Value) -> Option<Rational> {
    match value {
        Value::Rational(value) => Some(*value),
        Value::Real(value)
            if value.is_finite()
                && value.fract() == 0.0
                && value.abs() <= i64::MAX as f64 =>
        {
            Rational::new(*value as i64, 1)
        }
        _ => None,
    }
}

fn value_as_real(value: &Value) -> Result<f64, EvalError> {
    value
        .scalar()
        .ok_or_else(|| EvalError::Domain("expected a real scalar".into()))
}

fn index_value(value: &Value) -> Result<usize, EvalError> {
    let value = value_as_real(value)?;
    if !value.is_finite() || value < 1.0 || value.fract() != 0.0 {
        return Err(EvalError::Domain(
            "indices must be positive integers".into(),
        ));
    }
    Ok(value as usize)
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Real(_) => "a scalar",
        Value::Rational(_) => "a rational",
        Value::Complex(_) => "a complex number",
        Value::Quantity(_) => "a quantity",
        Value::Text(_) => "text",
        Value::Matrix(_) => "a matrix",
        Value::Function { .. } => "a function",
    }
}

fn incompatible(action: &str, left: &Value, right: &Value) -> EvalError {
    EvalError::IncompatibleValues(format!(
        "cannot {action} {} and {}",
        value_kind(left),
        value_kind(right)
    ))
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Text(left), Value::Text(right)) => left == right,
        (Value::Real(left), Value::Real(right)) => (left - right).abs() < 1e-12,
        (Value::Rational(left), Value::Rational(right)) => left == right,
        (Value::Quantity(left), Value::Quantity(right)) => {
            left.unit.dimension == right.unit.dimension
                && (left.value - right.value).abs() < 1e-12
        }
        _ => false,
    }
}

fn compare_values(op: BinaryOp, left: &Value, right: &Value) -> Result<Value, EvalError> {
    if let (Value::Text(left), Value::Text(right)) = (left, right) {
        let result = match op {
            BinaryOp::Eq => left == right,
            BinaryOp::Ne => left != right,
            _ => {
                return Err(EvalError::Domain(
                    "text only supports == and !=".into(),
                ))
            }
        };
        return Ok(Value::Real(if result { 1.0 } else { 0.0 }));
    }

    let (left, right) = comparable_values(left, right)?;
    let result = match op {
        BinaryOp::Lt => left < right,
        BinaryOp::Le => left <= right,
        BinaryOp::Gt => left > right,
        BinaryOp::Ge => left >= right,
        BinaryOp::Eq => left == right,
        BinaryOp::Ne => left != right,
        _ => return Err(EvalError::Domain("invalid comparison".into())),
    };
    Ok(Value::Real(if result { 1.0 } else { 0.0 }))
}

fn comparable_values(left: &Value, right: &Value) -> Result<(f64, f64), EvalError> {
    match (left, right) {
        (Value::Quantity(left), Value::Quantity(right)) => {
            if left.unit.dimension != right.unit.dimension {
                return Err(EvalError::Unit(
                    "cannot compare quantities with different dimensions".into(),
                ));
            }
            Ok((left.value, right.value))
        }
        (Value::Quantity(left), right) if left.is_dimensionless() => {
            Ok((left.value, comparable_scalar(right)?))
        }
        (left, Value::Quantity(right)) if right.is_dimensionless() => {
            Ok((comparable_scalar(left)?, right.value))
        }
        (Value::Quantity(_), _) | (_, Value::Quantity(_)) => Err(EvalError::Unit(
            "cannot compare a quantity with a dimensionless value".into(),
        )),
        (left, right) => Ok((comparable_scalar(left)?, comparable_scalar(right)?)),
    }
}

fn comparable_scalar(value: &Value) -> Result<f64, EvalError> {
    match value {
        Value::Real(value) => Ok(*value),
        Value::Rational(value) => Ok(value.to_f64()),
        Value::Complex(value) if value.is_real() => Ok(value.re),
        _ => Err(EvalError::Domain(
            "comparison expects real scalar values".into(),
        )),
    }
}

fn truthy(value: &Value) -> Result<bool, EvalError> {
    match value {
        Value::Real(value) => Ok(*value != 0.0),
        Value::Rational(value) => Ok(value.to_f64() != 0.0),
        Value::Complex(value) if value.is_real() => Ok(value.re != 0.0),
        Value::Complex(_) => Err(EvalError::Domain("a condition must be real".into())),
        Value::Quantity(quantity) if quantity.is_dimensionless() => Ok(quantity.value != 0.0),
        Value::Quantity(_) => Err(EvalError::Domain(
            "a condition must be dimensionless".into(),
        )),
        Value::Text(value) => Ok(!value.is_empty()),
        Value::Matrix(_) | Value::Function { .. } => Err(EvalError::Domain(
            "a condition must be a scalar value".into(),
        )),
    }
}
