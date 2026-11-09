// SPDX-License-Identifier: GPL-3.0-or-later

//! Small rule-based symbolic engine used by the evaluator.
//!
//! It intentionally stays conservative: when a rule does not apply, the
//! original expression is preserved instead of returning a wrong result.

use math_ast::{BinaryOp, Expr, UnaryOp};

fn number(value: f64) -> Expr {
    Expr::Number(value)
}

fn symbol(name: impl Into<String>) -> Expr {
    Expr::Symbol(name.into())
}

fn neg(value: Expr) -> Expr {
    Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(value),
    }
}

fn add(left: Expr, right: Expr) -> Expr {
    Expr::binary(BinaryOp::Add, left, right)
}

fn sub(left: Expr, right: Expr) -> Expr {
    Expr::binary(BinaryOp::Sub, left, right)
}

fn mul(left: Expr, right: Expr) -> Expr {
    Expr::binary(BinaryOp::Mul, left, right)
}

fn div(left: Expr, right: Expr) -> Expr {
    Expr::binary(BinaryOp::Div, left, right)
}

fn pow(left: Expr, right: Expr) -> Expr {
    Expr::binary(BinaryOp::Pow, left, right)
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        name: name.into(),
        args,
    }
}

fn is_number(expr: &Expr, expected: f64) -> bool {
    matches!(expr, Expr::Number(value) if (*value - expected).abs() < 1e-12)
}

fn is_zero(expr: &Expr) -> bool {
    is_number(expr, 0.0)
}

fn is_one(expr: &Expr) -> bool {
    is_number(expr, 1.0)
}

fn is_integer(expr: &Expr) -> Option<i64> {
    let Expr::Number(value) = expr else {
        return None;
    };
    if !value.is_finite() {
        return None;
    }
    let rounded = value.round();
    if (value - rounded).abs() < 1e-9 && rounded.abs() <= i64::MAX as f64 {
        Some(rounded as i64)
    } else {
        None
    }
}

fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    if (value - value.round()).abs() < 1e-12 && value.abs() < 1e15 {
        return format!("{}", value.round() as i64);
    }
    let mut text = format!("{value:.12}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

/// Simplify arithmetic identities and fold numeric constants.
pub fn simplify(expr: &Expr) -> Expr {
    match expr {
        Expr::Number(_) | Expr::Symbol(_) => expr.clone(),
        Expr::Unary { op, expr } => {
            let inner = simplify(expr);
            match op {
                UnaryOp::Pos => inner,
                UnaryOp::Neg => negate_simplified(inner),
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            let left = simplify(lhs);
            let right = simplify(rhs);
            simplify_binary(*op, left, right)
        }
        Expr::Call { name, args } => {
            let args = args.iter().map(simplify).collect::<Vec<_>>();
            simplify_call(name, args)
        }
        Expr::List(items) => Expr::List(items.iter().map(simplify).collect()),
        Expr::Matrix(rows) => Expr::Matrix(
            rows.iter()
                .map(|row| row.iter().map(simplify).collect())
                .collect(),
        ),
        Expr::Index { target, indices } => Expr::Index {
            target: Box::new(simplify(target)),
            indices: indices.iter().map(simplify).collect(),
        },
        Expr::Assign { name, value } => Expr::Assign {
            name: name.clone(),
            value: Box::new(simplify(value)),
        },
        Expr::FunctionDef { name, params, body } => Expr::FunctionDef {
            name: name.clone(),
            params: params.clone(),
            body: Box::new(simplify(body)),
        },
        Expr::UnitDef { name, value } => Expr::UnitDef {
            name: name.clone(),
            value: Box::new(simplify(value)),
        },
    }
}

fn negate_simplified(expr: Expr) -> Expr {
    match expr {
        Expr::Number(value) => number(-value),
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } => *expr,
        Expr::Binary {
            op: BinaryOp::Sub,
            lhs,
            rhs,
        } if is_zero(&lhs) => *rhs,
        other => neg(other),
    }
}

fn simplify_add_sub(left: Expr, right: Expr, right_sign: f64) -> Expr {
    let mut terms = Vec::new();
    let mut constant = 0.0;
    collect_terms(&left, 1.0, &mut terms, &mut constant);
    collect_terms(&right, right_sign, &mut terms, &mut constant);

    let mut combined: Vec<(f64, Expr)> = Vec::new();
    for (coefficient, expr) in terms {
        if coefficient.abs() < 1e-12 {
            continue;
        }
        if let Some((existing, _)) = combined
            .iter_mut()
            .find(|(_, candidate)| *candidate == expr)
        {
            *existing += coefficient;
        } else {
            combined.push((coefficient, expr));
        }
    }
    combined.retain(|(coefficient, _)| coefficient.abs() > 1e-12);

    let mut result: Option<Expr> = None;
    for (coefficient, expr) in combined {
        let absolute = coefficient.abs();
        let term = if (absolute - 1.0).abs() < 1e-12 {
            expr
        } else {
            mul(number(absolute), expr)
        };
        result = Some(match result {
            None if coefficient < 0.0 => neg(term),
            None => term,
            Some(current) if coefficient < 0.0 => sub(current, term),
            Some(current) => add(current, term),
        });
    }

    if constant.abs() > 1e-12 {
        let term = number(constant.abs());
        result = Some(match result {
            None if constant < 0.0 => neg(term),
            None => term,
            Some(current) if constant < 0.0 => sub(current, term),
            Some(current) => add(current, term),
        });
    }

    result.unwrap_or_else(|| number(0.0))
}

fn collect_terms(expr: &Expr, sign: f64, terms: &mut Vec<(f64, Expr)>, constant: &mut f64) {
    match expr {
        Expr::Binary {
            op: BinaryOp::Add,
            lhs,
            rhs,
        } => {
            collect_terms(lhs, sign, terms, constant);
            collect_terms(rhs, sign, terms, constant);
        }
        Expr::Binary {
            op: BinaryOp::Sub,
            lhs,
            rhs,
        } => {
            collect_terms(lhs, sign, terms, constant);
            collect_terms(rhs, -sign, terms, constant);
        }
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } => collect_terms(expr, -sign, terms, constant),
        Expr::Number(value) => *constant += sign * value,
        Expr::Binary {
            op: BinaryOp::Mul,
            lhs,
            rhs,
        } => {
            if let Expr::Number(value) = &**lhs {
                terms.push((sign * value, *rhs.clone()));
            } else if let Expr::Number(value) = &**rhs {
                terms.push((sign * value, *lhs.clone()));
            } else {
                terms.push((sign, expr.clone()));
            }
        }
        _ => terms.push((sign, expr.clone())),
    }
}

fn simplify_binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    match op {
        BinaryOp::Add => simplify_add_sub(left, right, 1.0),
        BinaryOp::Sub => simplify_add_sub(left, right, -1.0),
        BinaryOp::Mul => {
            if is_zero(&left) || is_zero(&right) {
                return number(0.0);
            }
            if is_one(&left) {
                return right;
            }
            if is_one(&right) {
                return left;
            }
            if let (Expr::Number(a), Expr::Number(b)) = (&left, &right) {
                return number(a * b);
            }
            if left == right {
                return pow(left, number(2.0));
            }
            if let Expr::Unary {
                op: UnaryOp::Neg,
                expr,
            } = &right
            {
                return negate_simplified(mul(left, *expr.clone()));
            }
            if let Expr::Unary {
                op: UnaryOp::Neg,
                expr,
            } = &left
            {
                return negate_simplified(mul(*expr.clone(), right));
            }
            mul(left, right)
        }
        BinaryOp::Div => {
            if is_zero(&left) {
                return number(0.0);
            }
            if is_one(&right) {
                return left;
            }
            if left == right {
                return number(1.0);
            }
            if let (Expr::Number(a), Expr::Number(b)) = (&left, &right) {
                if *b != 0.0 {
                    return number(a / b);
                }
            }
            div(left, right)
        }
        BinaryOp::Pow => {
            if is_zero(&right) {
                return number(1.0);
            }
            if is_one(&right) {
                return left;
            }
            if is_one(&left) {
                return number(1.0);
            }
            if let (Expr::Number(base), Expr::Number(exponent)) = (&left, &right) {
                let valid = base.is_finite()
                    && exponent.is_finite()
                    && (exponent.fract() == 0.0 || *base > 0.0);
                if valid {
                    return number(base.powf(*exponent));
                }
            }
            if let Expr::Binary {
                op: BinaryOp::Pow,
                lhs,
                rhs,
            } = &left
            {
                if let (Some(inner), Some(outer)) = (is_integer(rhs), is_integer(&right)) {
                    return pow(*lhs.clone(), number((inner * outer) as f64));
                }
            }
            pow(left, right)
        }
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge | BinaryOp::Eq | BinaryOp::Ne => {
            Expr::binary(op, left, right)
        }
    }
}

fn simplify_call(name: &str, args: Vec<Expr>) -> Expr {
    if args.len() == 1 {
        let argument = &args[0];
        if let Expr::Number(value) = argument {
            match name {
                "abs" => return number(value.abs()),
                "sqrt" if *value >= 0.0 => {
                    let root = value.sqrt();
                    if (root - root.round()).abs() < 1e-12 {
                        return number(root.round());
                    }
                }
                "sin" if *value == 0.0 => return number(0.0),
                "cos" if *value == 0.0 => return number(1.0),
                "tan" if *value == 0.0 => return number(0.0),
                "exp" if *value == 0.0 => return number(1.0),
                "ln" if *value == 1.0 => return number(0.0),
                _ => {}
            }
        }
        if is_zero(argument) {
            match name {
                "abs" => return number(0.0),
                "sqrt" => return number(0.0),
                "sin" | "tan" => return number(0.0),
                "cos" | "exp" => return number(1.0),
                _ => {}
            }
        }
        if is_one(argument) {
            match name {
                "ln" => return number(0.0),
                "abs" | "sqrt" => return number(1.0),
                _ => {}
            }
        }
        if name == "abs" {
            if let Expr::Unary {
                op: UnaryOp::Neg,
                expr,
            } = argument
            {
                return call("abs", vec![*expr.clone()]);
            }
        }
    }
    call(name, args)
}

/// Replace every occurrence of `name` with `replacement`.
pub fn substitute(expr: &Expr, name: &str, replacement: &Expr) -> Expr {
    match expr {
        Expr::Symbol(current) if current == name => replacement.clone(),
        Expr::Number(_) | Expr::Symbol(_) => expr.clone(),
        Expr::Unary { op, expr } => Expr::Unary {
            op: *op,
            expr: Box::new(substitute(expr, name, replacement)),
        },
        Expr::Binary { op, lhs, rhs } => Expr::binary(
            *op,
            substitute(lhs, name, replacement),
            substitute(rhs, name, replacement),
        ),
        Expr::Call {
            name: call_name,
            args,
        } => Expr::Call {
            name: call_name.clone(),
            args: args
                .iter()
                .map(|arg| substitute(arg, name, replacement))
                .collect(),
        },
        Expr::List(items) => Expr::List(
            items
                .iter()
                .map(|item| substitute(item, name, replacement))
                .collect(),
        ),
        Expr::Matrix(rows) => Expr::Matrix(
            rows.iter()
                .map(|row| {
                    row.iter()
                        .map(|item| substitute(item, name, replacement))
                        .collect()
                })
                .collect(),
        ),
        Expr::Index { target, indices } => Expr::Index {
            target: Box::new(substitute(target, name, replacement)),
            indices: indices
                .iter()
                .map(|index| substitute(index, name, replacement))
                .collect(),
        },
        Expr::Assign {
            name: target,
            value,
        } => Expr::Assign {
            name: target.clone(),
            value: Box::new(substitute(value, name, replacement)),
        },
        Expr::FunctionDef {
            name: function_name,
            params,
            body,
        } => Expr::FunctionDef {
            name: function_name.clone(),
            params: params.clone(),
            body: Box::new(substitute(body, name, replacement)),
        },
        Expr::UnitDef {
            name: unit_name,
            value,
        } => Expr::UnitDef {
            name: unit_name.clone(),
            value: Box::new(substitute(value, name, replacement)),
        },
    }
}

/// Distribute products over sums and expand small integer powers of sums.
pub fn expand(expr: &Expr) -> Expr {
    let expanded = match expr {
        Expr::Number(_) | Expr::Symbol(_) => expr.clone(),
        Expr::Unary { op, expr } => Expr::Unary {
            op: *op,
            expr: Box::new(expand(expr)),
        },
        Expr::Binary { op, lhs, rhs } => {
            let left = expand(lhs);
            let right = expand(rhs);
            match op {
                BinaryOp::Add | BinaryOp::Sub => Expr::binary(*op, left, right),
                BinaryOp::Mul => expand_product(left, right),
                BinaryOp::Div => div(left, right),
                BinaryOp::Pow => expand_power(left, right),
                _ => Expr::binary(*op, left, right),
            }
        }
        Expr::Call { name, args } => Expr::Call {
            name: name.clone(),
            args: args.iter().map(expand).collect(),
        },
        Expr::List(items) => Expr::List(items.iter().map(expand).collect()),
        Expr::Matrix(rows) => Expr::Matrix(
            rows.iter()
                .map(|row| row.iter().map(expand).collect())
                .collect(),
        ),
        _ => expr.clone(),
    };
    simplify(&expanded)
}

fn expand_product(left: Expr, right: Expr) -> Expr {
    match (left, right) {
        (
            Expr::Binary {
                op: BinaryOp::Add,
                lhs,
                rhs,
            },
            right,
        ) => add(
            expand_product(*lhs, right.clone()),
            expand_product(*rhs, right),
        ),
        (
            left,
            Expr::Binary {
                op: BinaryOp::Add,
                lhs,
                rhs,
            },
        ) => add(
            expand_product(left.clone(), *lhs),
            expand_product(left, *rhs),
        ),
        (
            Expr::Binary {
                op: BinaryOp::Sub,
                lhs,
                rhs,
            },
            right,
        ) => sub(
            expand_product(*lhs, right.clone()),
            expand_product(*rhs, right),
        ),
        (
            left,
            Expr::Binary {
                op: BinaryOp::Sub,
                lhs,
                rhs,
            },
        ) => sub(
            expand_product(left.clone(), *lhs),
            expand_product(left, *rhs),
        ),
        (left, right) => mul(left, right),
    }
}

fn expand_power(base: Expr, exponent: Expr) -> Expr {
    if let (
        Expr::Binary {
            op: BinaryOp::Add | BinaryOp::Sub,
            ..
        },
        Some(power),
    ) = (&base, is_integer(&exponent))
    {
        if (2..=8).contains(&power) {
            let mut result = number(1.0);
            for _ in 0..power {
                result = expand_product(result, base.clone());
            }
            return result;
        }
    }
    pow(base, exponent)
}

/// Apply the most common factorizations: difference of squares and a shared
/// factor in a sum of two products.
pub fn factor(expr: &Expr) -> Expr {
    let simplified = simplify(expr);
    match &simplified {
        Expr::Binary {
            op: BinaryOp::Sub,
            lhs,
            rhs,
        } => {
            if let (Some(left_base), Some(right_base)) =
                (square_root_expr(lhs), square_root_expr(rhs))
            {
                return simplify(&mul(
                    sub(left_base.clone(), right_base.clone()),
                    add(left_base, right_base),
                ));
            }
            simplified
        }
        Expr::Binary {
            op: BinaryOp::Add,
            lhs,
            rhs,
        } => {
            if let (Some(left_factor), Some(right_factor)) =
                (common_factor(lhs), common_factor(rhs))
            {
                if left_factor == right_factor && !is_one(&left_factor) {
                    let left_rest = divide_by(lhs, &left_factor);
                    let right_rest = divide_by(rhs, &left_factor);
                    return simplify(&mul(left_factor, add(left_rest, right_rest)));
                }
            }
            simplified
        }
        _ => simplified,
    }
}

fn square_root_expr(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Binary {
            op: BinaryOp::Pow,
            lhs,
            rhs,
        } if is_number(rhs, 2.0) => Some(*lhs.clone()),
        Expr::Number(value) if *value >= 0.0 => {
            let root = value.sqrt();
            if (root - root.round()).abs() < 1e-12 {
                Some(number(root.round()))
            } else {
                None
            }
        }
        Expr::Binary {
            op: BinaryOp::Mul,
            lhs,
            rhs,
        } => {
            let left = square_root_expr(lhs)?;
            let right = square_root_expr(rhs)?;
            Some(mul(left, right))
        }
        _ => None,
    }
}

fn common_factor(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Binary {
            op: BinaryOp::Mul,
            lhs,
            ..
        } => Some(*lhs.clone()),
        _ => None,
    }
}

fn divide_by(expr: &Expr, factor: &Expr) -> Expr {
    match expr {
        Expr::Binary {
            op: BinaryOp::Mul,
            lhs,
            rhs,
        } if **lhs == *factor => *rhs.clone(),
        Expr::Binary {
            op: BinaryOp::Mul,
            lhs,
            rhs,
        } if **rhs == *factor => *lhs.clone(),
        _ => expr.clone(),
    }
}

/// Symbolic derivative with respect to `variable`.
pub fn differentiate(expr: &Expr, variable: &str) -> Expr {
    let derivative = match expr {
        Expr::Number(_) => number(0.0),
        Expr::Symbol(name) => {
            if name == variable {
                number(1.0)
            } else {
                number(0.0)
            }
        }
        Expr::Unary { op, expr } => match op {
            UnaryOp::Neg => neg(differentiate(expr, variable)),
            UnaryOp::Pos => differentiate(expr, variable),
        },
        Expr::Binary { op, lhs, rhs } => match op {
            BinaryOp::Add => add(differentiate(lhs, variable), differentiate(rhs, variable)),
            BinaryOp::Sub => sub(differentiate(lhs, variable), differentiate(rhs, variable)),
            BinaryOp::Mul => add(
                mul(differentiate(lhs, variable), *rhs.clone()),
                mul(*lhs.clone(), differentiate(rhs, variable)),
            ),
            BinaryOp::Div => div(
                sub(
                    mul(differentiate(lhs, variable), *rhs.clone()),
                    mul(*lhs.clone(), differentiate(rhs, variable)),
                ),
                pow(*rhs.clone(), number(2.0)),
            ),
            BinaryOp::Pow => {
                let base_derivative = differentiate(lhs, variable);
                let exponent_derivative = differentiate(rhs, variable);
                if is_zero(&exponent_derivative) {
                    mul(
                        mul(
                            *rhs.clone(),
                            pow(*lhs.clone(), sub(*rhs.clone(), number(1.0))),
                        ),
                        base_derivative,
                    )
                } else {
                    mul(
                        pow(*lhs.clone(), *rhs.clone()),
                        add(
                            mul(exponent_derivative, call("ln", vec![*lhs.clone()])),
                            div(mul(*rhs.clone(), base_derivative), *lhs.clone()),
                        ),
                    )
                }
            }
            _ => number(0.0),
        },
        Expr::Call { name, args } if args.len() == 1 => {
            let argument = &args[0];
            let inner = differentiate(argument, variable);
            match name.as_str() {
                "sin" => mul(call("cos", vec![argument.clone()]), inner),
                "cos" => neg(mul(call("sin", vec![argument.clone()]), inner)),
                "tan" => div(inner, pow(call("cos", vec![argument.clone()]), number(2.0))),
                "exp" => mul(call("exp", vec![argument.clone()]), inner),
                "ln" => div(inner, argument.clone()),
                "sqrt" => div(
                    inner,
                    mul(number(2.0), call("sqrt", vec![argument.clone()])),
                ),
                "abs" => mul(call("sign", vec![argument.clone()]), inner),
                "asin" => div(
                    inner,
                    call(
                        "sqrt",
                        vec![sub(number(1.0), pow(argument.clone(), number(2.0)))],
                    ),
                ),
                "acos" => neg(div(
                    inner,
                    call(
                        "sqrt",
                        vec![sub(number(1.0), pow(argument.clone(), number(2.0)))],
                    ),
                )),
                "atan" => div(inner, add(number(1.0), pow(argument.clone(), number(2.0)))),
                "sinh" => mul(call("cosh", vec![argument.clone()]), inner),
                "cosh" => mul(call("sinh", vec![argument.clone()]), inner),
                "tanh" => div(
                    inner,
                    pow(call("cosh", vec![argument.clone()]), number(2.0)),
                ),
                _ => number(0.0),
            }
        }
        Expr::Call { args, .. } => {
            let _ = args;
            number(0.0)
        }
        _ => number(0.0),
    };
    simplify(&derivative)
}

/// Basic symbolic antiderivative. Returns `None` when no elementary rule
/// applies.
pub fn integrate(expr: &Expr, variable: &str) -> Option<Expr> {
    let result = match expr {
        Expr::Number(value) => mul(number(*value), symbol(variable)),
        Expr::Symbol(name) if name == variable => {
            div(pow(symbol(variable), number(2.0)), number(2.0))
        }
        Expr::Symbol(name) => mul(symbol(name.clone()), symbol(variable)),
        Expr::Unary { op, expr } => match op {
            UnaryOp::Neg => neg(integrate(expr, variable)?),
            UnaryOp::Pos => integrate(expr, variable)?,
        },
        Expr::Binary { op, lhs, rhs } => match op {
            BinaryOp::Add => add(integrate(lhs, variable)?, integrate(rhs, variable)?),
            BinaryOp::Sub => sub(integrate(lhs, variable)?, integrate(rhs, variable)?),
            BinaryOp::Mul => {
                if !contains_symbol(lhs, variable) {
                    mul(*lhs.clone(), integrate(rhs, variable)?)
                } else if !contains_symbol(rhs, variable) {
                    mul(integrate(lhs, variable)?, *rhs.clone())
                } else {
                    return None;
                }
            }
            BinaryOp::Div => {
                if contains_symbol(rhs, variable) {
                    if let Expr::Symbol(name) = &**rhs {
                        if name == variable && is_one(lhs) {
                            call("ln", vec![call("abs", vec![symbol(variable)])])
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    div(integrate(lhs, variable)?, *rhs.clone())
                }
            }
            BinaryOp::Pow => {
                let Expr::Symbol(name) = &**lhs else {
                    return None;
                };
                if name != variable {
                    return None;
                }
                let power = is_integer(rhs)?;
                if power == -1 {
                    call("ln", vec![call("abs", vec![symbol(variable)])])
                } else {
                    div(
                        pow(symbol(variable), number((power + 1) as f64)),
                        number((power + 1) as f64),
                    )
                }
            }
            _ => return None,
        },
        Expr::Call { name, args } if args.len() == 1 => {
            let argument = &args[0];
            if !contains_symbol(argument, variable) {
                return None;
            }
            if !matches!(argument, Expr::Symbol(symbol) if symbol == variable) {
                return None;
            }
            match name.as_str() {
                "sin" => neg(call("cos", vec![symbol(variable)])),
                "cos" => call("sin", vec![symbol(variable)]),
                "exp" => call("exp", vec![symbol(variable)]),
                _ => return None,
            }
        }
        _ => return None,
    };
    Some(simplify(&result))
}

fn contains_symbol(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Symbol(current) => current == name,
        Expr::Number(_) => false,
        Expr::Unary { expr, .. } => contains_symbol(expr, name),
        Expr::Binary { lhs, rhs, .. } => contains_symbol(lhs, name) || contains_symbol(rhs, name),
        Expr::Call { args, .. } => args.iter().any(|arg| contains_symbol(arg, name)),
        Expr::List(items) => items.iter().any(|item| contains_symbol(item, name)),
        Expr::Matrix(rows) => rows
            .iter()
            .flatten()
            .any(|item| contains_symbol(item, name)),
        Expr::Index { target, indices } => {
            contains_symbol(target, name)
                || indices.iter().any(|index| contains_symbol(index, name))
        }
        _ => false,
    }
}

/// Readable ASCII form used to display symbolic results.
pub fn to_ascii(expr: &Expr) -> String {
    format_expr(expr, 0)
}

fn precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::Binary { op, .. } => match op {
            BinaryOp::Add | BinaryOp::Sub => 1,
            BinaryOp::Mul | BinaryOp::Div => 2,
            BinaryOp::Pow => 4,
            _ => 1,
        },
        Expr::Unary { .. } => 3,
        _ => 5,
    }
}

fn format_expr(expr: &Expr, parent: u8) -> String {
    let priority = precedence(expr);
    let rendered = match expr {
        Expr::Number(value) => format_number(*value),
        Expr::Symbol(name) => name.clone(),
        Expr::Unary { op, expr } => match op {
            UnaryOp::Neg => format!("-{}", format_expr(expr, 4)),
            UnaryOp::Pos => format_expr(expr, parent),
        },
        Expr::Binary { op, lhs, rhs } => {
            let symbol = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Pow => "^",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
            };
            format!(
                "{} {} {}",
                format_expr(lhs, priority),
                symbol,
                format_expr(rhs, priority + 1)
            )
        }
        Expr::Call { name, args } => format!(
            "{name}({})",
            args.iter()
                .map(|arg| format_expr(arg, 0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|item| format_expr(item, 0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Matrix(rows) => format!(
            "[{}]",
            rows.iter()
                .map(|row| {
                    format!(
                        "[{}]",
                        row.iter()
                            .map(|item| format_expr(item, 0))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Index { target, indices } => format!(
            "{}[{}]",
            format_expr(target, 5),
            indices
                .iter()
                .map(|index| format_expr(index, 0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Assign { name, value } => {
            format!("{name} := {}", format_expr(value, 0))
        }
        Expr::FunctionDef { name, params, body } => {
            format!("{name}({}) := {}", params.join(", "), format_expr(body, 0))
        }
        Expr::UnitDef { name, value } => {
            format!("unit({name}) := {}", format_expr(value, 0))
        }
    };

    if priority < parent
        && !matches!(
            expr,
            Expr::Unary {
                op: UnaryOp::Neg,
                ..
            }
        )
    {
        format!("({rendered})")
    } else {
        rendered
    }
}

/// Numeric polynomial coefficients in `variable`, from degree 0 upwards.
pub fn polynomial_coefficients(expr: &Expr, variable: &str) -> Option<Vec<f64>> {
    let simplified = simplify(expr);
    poly_coefficients(&simplified, variable)
}

fn poly_coefficients(expr: &Expr, variable: &str) -> Option<Vec<f64>> {
    match expr {
        Expr::Number(value) => Some(vec![*value]),
        Expr::Symbol(name) if name == variable => Some(vec![0.0, 1.0]),
        Expr::Symbol(_) => None,
        Expr::Unary { op, expr } => {
            let coefficients = poly_coefficients(expr, variable)?;
            match op {
                UnaryOp::Pos => Some(coefficients),
                UnaryOp::Neg => Some(coefficients.into_iter().map(|value| -value).collect()),
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            let left = poly_coefficients(lhs, variable)?;
            let right = poly_coefficients(rhs, variable)?;
            match op {
                BinaryOp::Add => Some(poly_add(&left, &right, 1.0)),
                BinaryOp::Sub => Some(poly_add(&left, &right, -1.0)),
                BinaryOp::Mul => Some(poly_mul(&left, &right)),
                BinaryOp::Div => {
                    if right.len() == 1 {
                        let divisor = right[0];
                        if divisor.abs() < 1e-15 {
                            return None;
                        }
                        Some(left.into_iter().map(|value| value / divisor).collect())
                    } else {
                        None
                    }
                }
                BinaryOp::Pow => {
                    let exponent = is_integer(rhs)?;
                    if !(0..=32).contains(&exponent) {
                        return None;
                    }
                    let mut result = vec![1.0];
                    for _ in 0..exponent {
                        result = poly_mul(&result, &left);
                    }
                    Some(result)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn poly_add(left: &[f64], right: &[f64], right_scale: f64) -> Vec<f64> {
    let length = left.len().max(right.len());
    let mut result = vec![0.0; length];
    for (index, value) in left.iter().enumerate() {
        result[index] += value;
    }
    for (index, value) in right.iter().enumerate() {
        result[index] += value * right_scale;
    }
    result
}

fn poly_mul(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; left.len() + right.len() - 1];
    for (left_index, left_value) in left.iter().enumerate() {
        for (right_index, right_value) in right.iter().enumerate() {
            result[left_index + right_index] += left_value * right_value;
        }
    }
    result
}

/// Numeric complex roots of a polynomial given its coefficients from degree 0
/// upwards. Uses the Durand-Kerner iteration.
pub fn polynomial_roots(coefficients: &[f64]) -> Vec<(f64, f64)> {
    let mut coefficients = coefficients.to_vec();
    while coefficients.len() > 1 && coefficients.last().is_some_and(|value| value.abs() < 1e-12) {
        coefficients.pop();
    }
    let degree = coefficients.len().saturating_sub(1);
    if degree == 0 {
        return Vec::new();
    }

    let leading = coefficients[degree];
    if leading.abs() < 1e-15 {
        return Vec::new();
    }
    for coefficient in &mut coefficients {
        *coefficient /= leading;
    }

    let mut roots = (0..degree)
        .map(|index| {
            let angle = 2.0 * std::f64::consts::PI * index as f64 / degree as f64 + 0.4;
            (0.9 * angle.cos(), 0.9 * angle.sin())
        })
        .collect::<Vec<_>>();

    for _ in 0..300 {
        let mut largest = 0.0_f64;
        for index in 0..degree {
            let current = roots[index];
            let value = evaluate_complex_polynomial(&coefficients, current);
            let mut denominator = (1.0, 0.0);
            for (other_index, other) in roots.iter().enumerate() {
                if other_index != index {
                    denominator =
                        complex_mul(denominator, (current.0 - other.0, current.1 - other.1));
                }
            }
            if complex_abs(denominator) < 1e-18 {
                continue;
            }
            let delta = complex_div(value, denominator);
            largest = largest.max(complex_abs(delta));
            roots[index] = (current.0 - delta.0, current.1 - delta.1);
        }
        if largest < 1e-12 {
            break;
        }
    }

    roots
        .into_iter()
        .map(
            |(re, im)| {
                if im.abs() < 1e-8 {
                    (re, 0.0)
                } else {
                    (re, im)
                }
            },
        )
        .collect()
}

fn evaluate_complex_polynomial(coefficients: &[f64], value: (f64, f64)) -> (f64, f64) {
    let mut result = (0.0, 0.0);
    for coefficient in coefficients.iter().rev() {
        result = complex_mul(result, value);
        result.0 += coefficient;
    }
    result
}

fn complex_mul(left: (f64, f64), right: (f64, f64)) -> (f64, f64) {
    (
        left.0 * right.0 - left.1 * right.1,
        left.0 * right.1 + left.1 * right.0,
    )
}

fn complex_div(left: (f64, f64), right: (f64, f64)) -> (f64, f64) {
    let denominator = right.0 * right.0 + right.1 * right.1;
    (
        (left.0 * right.0 + left.1 * right.1) / denominator,
        (left.1 * right.0 - left.0 * right.1) / denominator,
    )
}

fn complex_abs(value: (f64, f64)) -> f64 {
    (value.0 * value.0 + value.1 * value.1).sqrt()
}

/// Return the single variable used by an expression, if there is exactly one.
pub fn single_variable(expr: &Expr) -> Option<String> {
    let mut variables = Vec::new();
    collect_variables(expr, &mut variables);
    variables.sort();
    variables.dedup();
    if variables.len() == 1 {
        variables.pop()
    } else {
        None
    }
}

fn collect_variables(expr: &Expr, variables: &mut Vec<String>) {
    match expr {
        Expr::Symbol(name) => variables.push(name.clone()),
        Expr::Number(_) => {}
        Expr::Unary { expr, .. } => collect_variables(expr, variables),
        Expr::Binary { lhs, rhs, .. } => {
            collect_variables(lhs, variables);
            collect_variables(rhs, variables);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_variables(arg, variables);
            }
        }
        Expr::List(items) => {
            for item in items {
                collect_variables(item, variables);
            }
        }
        Expr::Matrix(rows) => {
            for item in rows.iter().flatten() {
                collect_variables(item, variables);
            }
        }
        Expr::Index { target, indices } => {
            collect_variables(target, variables);
            for index in indices {
                collect_variables(index, variables);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use math_parse::parse;

    fn parse_expr(source: &str) -> Expr {
        parse(source).expect("expression should parse")
    }

    #[test]
    fn simplifies_arithmetic_identities() {
        assert_eq!(to_ascii(&simplify(&parse_expr("0 + x + 0"))), "x");
        assert_eq!(to_ascii(&simplify(&parse_expr("1*x*1"))), "x");
        assert_eq!(to_ascii(&simplify(&parse_expr("x - x"))), "0");
        assert_eq!(to_ascii(&simplify(&parse_expr("x*x"))), "x ^ 2");
        assert_eq!(to_ascii(&simplify(&parse_expr("2 + 3*4"))), "14");
    }

    #[test]
    fn expands_products() {
        assert_eq!(
            to_ascii(&expand(&parse_expr("(x + 1)*(x - 1)"))),
            "x ^ 2 - 1"
        );
        assert_eq!(
            to_ascii(&expand(&parse_expr("(x + 1)^2"))),
            "x ^ 2 + 2 * x + 1"
        );
    }

    #[test]
    fn factors_difference_of_squares() {
        assert_eq!(
            to_ascii(&factor(&parse_expr("x^2 - 4"))),
            "(x - 2) * (x + 2)"
        );
    }

    #[test]
    fn differentiates_polynomials_and_functions() {
        assert_eq!(
            to_ascii(&differentiate(&parse_expr("x^3 + 2*x"), "x")),
            "3 * x ^ 2 + 2"
        );
        assert_eq!(
            to_ascii(&differentiate(&parse_expr("sin(x)"), "x")),
            "cos(x)"
        );
        assert_eq!(
            to_ascii(&differentiate(&parse_expr("exp(2*x)"), "x")),
            "exp(2 * x) * 2"
        );
    }

    #[test]
    fn integrates_basic_terms() {
        assert_eq!(
            to_ascii(&integrate(&parse_expr("x^2"), "x").expect("integral")),
            "x ^ 3 / 3"
        );
        assert_eq!(
            to_ascii(&integrate(&parse_expr("sin(x)"), "x").expect("integral")),
            "-cos(x)"
        );
        assert_eq!(
            to_ascii(&integrate(&parse_expr("1/x"), "x").expect("integral")),
            "ln(abs(x))"
        );
    }

    #[test]
    fn extracts_polynomial_coefficients() {
        assert_eq!(
            polynomial_coefficients(&parse_expr("2*x^2 - 3*x + 5"), "x"),
            Some(vec![5.0, -3.0, 2.0])
        );
    }

    #[test]
    fn finds_polynomial_roots() {
        let mut roots = polynomial_roots(&[-2.0, 1.0]);
        roots.sort_by(|left, right| left.0.total_cmp(&right.0));
        assert_eq!(roots, vec![(2.0, 0.0)]);

        let mut roots = polynomial_roots(&[1.0, 0.0, 1.0]);
        roots.sort_by(|left, right| left.1.total_cmp(&right.1));
        assert_eq!(roots.len(), 2);
        assert!((roots[0].1 + 1.0).abs() < 1e-6);
        assert!((roots[1].1 - 1.0).abs() < 1e-6);
    }
}
