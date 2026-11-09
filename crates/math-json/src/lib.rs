// SPDX-License-Identifier: GPL-3.0-or-later

use math_ast::{BinaryOp, Expr, UnaryOp};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MathJsonError {
    #[error("unsupported MathJSON expression: {0}")]
    Unsupported(String),
    #[error("invalid MathJSON: {0}")]
    Invalid(String),
}

/// Convert a MathJSON value into the internal expression tree.
///
/// The MVP supports numbers, symbols, arithmetic arrays and a small set of
/// function heads. Unknown heads are kept as function calls so the evaluator
/// can return a useful "unknown function" error.
pub fn from_value(value: &Value) -> Result<Expr, MathJsonError> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .map(Expr::Number)
            .ok_or_else(|| MathJsonError::Invalid("number is out of range".into())),
        Value::String(symbol) => Ok(Expr::Symbol(symbol.clone())),
        Value::Array(items) => {
            let head = items.first().and_then(Value::as_str).ok_or_else(|| {
                MathJsonError::Invalid("a MathJSON array must start with a string head".into())
            })?;

            let args = items[1..]
                .iter()
                .map(from_value)
                .collect::<Result<Vec<_>, _>>()?;

            match head {
                "Add" => fold_binary(args, BinaryOp::Add),
                "Subtract" => fold_binary(args, BinaryOp::Sub),
                "Multiply" => fold_binary(args, BinaryOp::Mul),
                "Divide" => fold_binary(args, BinaryOp::Div),
                "Power" => fold_binary(args, BinaryOp::Pow),
                "Negate" => unary(args, UnaryOp::Neg),
                "Sqrt" => call("sqrt", args),
                "Sin" => call("sin", args),
                "Cos" => call("cos", args),
                "Tan" => call("tan", args),
                "Ln" => call("ln", args),
                "Log" => call("log", args),
                "Abs" => call("abs", args),
                "Exp" => call("exp", args),
                "Equal" | "Equation" => call("equal", args),
                other => call(other.to_ascii_lowercase(), args),
            }
        }
        other => Err(MathJsonError::Unsupported(other.to_string())),
    }
}

fn fold_binary(mut args: Vec<Expr>, op: BinaryOp) -> Result<Expr, MathJsonError> {
    if args.is_empty() {
        return Err(MathJsonError::Invalid("empty binary expression".into()));
    }

    let mut acc = args.remove(0);
    for rhs in args {
        acc = Expr::binary(op, acc, rhs);
    }
    Ok(acc)
}

fn unary(mut args: Vec<Expr>, op: UnaryOp) -> Result<Expr, MathJsonError> {
    if args.len() != 1 {
        return Err(MathJsonError::Invalid(format!(
            "unary operator expects 1 argument, got {}",
            args.len()
        )));
    }
    Ok(Expr::Unary {
        op,
        expr: Box::new(args.remove(0)),
    })
}

fn call(name: impl Into<String>, args: Vec<Expr>) -> Result<Expr, MathJsonError> {
    Ok(Expr::Call {
        name: name.into(),
        args,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_addition() {
        let value = serde_json::json!(["Add", 2, 2]);
        let expression = from_value(&value).expect("MathJSON should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn parses_function_call() {
        let value = serde_json::json!(["Sqrt", 9]);
        let expression = from_value(&value).expect("MathJSON should parse");
        assert!(matches!(expression, Expr::Call { .. }));
    }
}
