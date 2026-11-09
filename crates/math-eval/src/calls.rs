const MAX_LOOP_ITERATIONS: u128 = 100_000;

fn evaluate_call(
    name: &str,
    args: &[Expr],
    environment: &mut Environment,
) -> Result<Value, EvalError> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "if" | "si" => return evaluate_if(args, environment),
        "for" | "para" => {
            return if args.len() == 3 {
                evaluate_matrix_for(args, environment)
            } else {
                evaluate_for(args, environment)
            }
        }
        "sum" | "sumatoria" if args.len() == 4 => {
            return evaluate_reduction("sum", args, environment, false)
        }
        "sum" | "sumatoria" if args.len() == 3 => {
            return evaluate_matrix_reduction("sum", args, environment, false)
        }
        "product" | "producto" if args.len() == 4 => {
            return evaluate_reduction("product", args, environment, true)
        }
        "product" | "producto" if args.len() == 3 => {
            return evaluate_matrix_reduction("product", args, environment, true)
        }
        "while" | "mientras" => return evaluate_while(args, environment),
        "line" | "bloque" => return evaluate_line(args, environment),
        "error" => return evaluate_error(args),
        "text" => return evaluate_text_literal(args),
        "simplify" => return evaluate_symbolic_unary("simplify", args, math_symbolic::simplify),
        "expand" => return evaluate_symbolic_unary("expand", args, math_symbolic::expand),
        "factor" => return evaluate_symbolic_unary("factor", args, math_symbolic::factor),
        "subs" | "substitute" => return evaluate_substitute(args),
        "diff" | "derivative" => return evaluate_derivative(args, environment),
        "integrate" => return evaluate_integral(args, environment),
        "integral" => return evaluate_definite_integral(args, environment),
        "solve" => {
            if args.len() == 1 {
                return evaluate_solve(args, environment);
            }
            if args.len() == 2 && !is_matrix_solve(args, environment) {
                return evaluate_solve(args, environment);
            }
        }
        "roots" => return evaluate_roots(args, environment),
        "polyroots" => return evaluate_polyroots(args, environment),
        "eval" => {
            if args.len() != 1 {
                return Err(EvalError::InvalidArity("eval".into()));
            }
            return evaluate_with(&args[0], environment);
        }
        _ => {}
    }

    let values = args
        .iter()
        .map(|arg| evaluate_with(arg, environment))
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(function) = environment.function(name).cloned() {
        return call_user_function(name, &function, &values, environment);
    }

    evaluate_builtin(name, &values)
}

fn is_matrix_solve(args: &[Expr], environment: &Environment) -> bool {
    if args.len() != 2 {
        return false;
    }
    let mut local = environment.clone();
    let Ok(Value::Matrix(_)) = evaluate_with(&args[0], &mut local) else {
        return false;
    };
    matches!(
        evaluate_with(&args[1], &mut local),
        Ok(Value::Matrix(_))
    )
}

fn call_user_function(
    name: &str,
    function: &UserFunction,
    args: &[Value],
    environment: &Environment,
) -> Result<Value, EvalError> {
    if args.len() != function.params.len() {
        return Err(EvalError::InvalidArity(name.to_string()));
    }
    if environment.call_stack.iter().any(|item| item == name) {
        return Err(EvalError::Recursion(name.to_string()));
    }

    let mut local = environment.clone();
    local.call_stack.push(name.to_string());
    for (param, value) in function.params.iter().zip(args) {
        local.set(param.clone(), value.clone());
    }
    evaluate_with(&function.body, &mut local)
}

fn evaluate_if(args: &[Expr], environment: &mut Environment) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::InvalidArity("if".into()));
    }
    let condition = evaluate_with(&args[0], environment)?;
    if truthy(&condition)? {
        evaluate_with(&args[1], environment)
    } else {
        evaluate_with(&args[2], environment)
    }
}

fn evaluate_text_literal(args: &[Expr]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidArity("text".into()));
    }
    let text = match &args[0] {
        Expr::Symbol(name) => name.clone(),
        other => math_symbolic::to_ascii(other),
    };
    Ok(Value::Text(text))
}

fn evaluate_symbolic_unary(
    name: &str,
    args: &[Expr],
    function: fn(&Expr) -> Expr,
) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidArity(name.into()));
    }
    Ok(Value::Text(math_symbolic::to_ascii(&function(&args[0]))))
}

fn symbol_name(expr: &Expr, function: &str) -> Result<String, EvalError> {
    match expr {
        Expr::Symbol(name) => Ok(name.clone()),
        _ => Err(EvalError::Domain(format!(
            "{function} expects a variable name, for example {function}(x^2, x)"
        ))),
    }
}

fn evaluate_substitute(args: &[Expr]) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::InvalidArity("substitute".into()));
    }
    let variable = symbol_name(&args[1], "substitute")?;
    let result = math_symbolic::simplify(&math_symbolic::substitute(
        &args[0],
        &variable,
        &args[2],
    ));
    Ok(Value::Text(math_symbolic::to_ascii(&result)))
}

fn evaluate_derivative(
    args: &[Expr],
    environment: &mut Environment,
) -> Result<Value, EvalError> {
    if args.len() != 2 && args.len() != 3 {
        return Err(EvalError::InvalidArity("diff".into()));
    }
    let variable = symbol_name(&args[1], "diff")?;
    let order = if args.len() == 3 {
        let value = evaluate_with(&args[2], environment)?;
        let value = value_as_real(&value)?.round();
        if value < 0.0 || value > 32.0 {
            return Err(EvalError::Domain(
                "the derivative order must be between 0 and 32".into(),
            ));
        }
        value as usize
    } else {
        1
    };

    let mut result = args[0].clone();
    for _ in 0..order {
        result = math_symbolic::differentiate(&result, &variable);
    }
    Ok(Value::Text(math_symbolic::to_ascii(
        &math_symbolic::simplify(&result),
    )))
}

fn evaluate_integral(
    args: &[Expr],
    environment: &mut Environment,
) -> Result<Value, EvalError> {
    match args.len() {
        2 => {
            let variable = symbol_name(&args[1], "integrate")?;
            match math_symbolic::integrate(&args[0], &variable) {
                Some(result) => Ok(Value::Text(math_symbolic::to_ascii(&result))),
                None => Err(EvalError::Domain(
                    "no elementary antiderivative was found".into(),
                )),
            }
        }
        4 => evaluate_definite_integral(args, environment),
        _ => Err(EvalError::InvalidArity("integrate".into())),
    }
}

fn evaluate_definite_integral(
    args: &[Expr],
    environment: &mut Environment,
) -> Result<Value, EvalError> {
    if args.len() != 4 {
        return Err(EvalError::InvalidArity("integral".into()));
    }
    let variable = symbol_name(&args[1], "integral")?;
    let lower = value_as_real(&evaluate_with(&args[2], environment)?)?;
    let upper = value_as_real(&evaluate_with(&args[3], environment)?)?;
    let result = integrate_numeric(&args[0], &variable, lower, upper, environment)?;
    Ok(Value::Real(result))
}

fn integrate_numeric(
    expression: &Expr,
    variable: &str,
    lower: f64,
    upper: f64,
    environment: &Environment,
) -> Result<f64, EvalError> {
    if !lower.is_finite() || !upper.is_finite() {
        return Err(EvalError::Domain(
            "integration bounds must be finite".into(),
        ));
    }
    if (upper - lower).abs() < 1e-15 {
        return Ok(0.0);
    }

    let mut local = environment.clone();
    let mut evaluate_at = |value: f64| -> Result<f64, EvalError> {
        local.set(variable.to_string(), Value::Real(value));
        let result = evaluate_with(expression, &mut local)?;
        value_as_real(&result)
    };

    let a = lower.min(upper);
    let b = lower.max(upper);
    let sign = if upper < lower { -1.0 } else { 1.0 };
    let fa = evaluate_at(a)?;
    let fb = evaluate_at(b)?;
    let middle = (a + b) / 2.0;
    let fm = evaluate_at(middle)?;
    let value = adaptive_simpson(&mut evaluate_at, a, b, fa, fm, fb, 1e-8, 20)?;
    Ok(sign * value)
}

#[allow(clippy::too_many_arguments)]
fn adaptive_simpson(
    function: &mut dyn FnMut(f64) -> Result<f64, EvalError>,
    a: f64,
    b: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    tolerance: f64,
    depth: u32,
) -> Result<f64, EvalError> {
    let middle = (a + b) / 2.0;
    let left_middle = (a + middle) / 2.0;
    let right_middle = (middle + b) / 2.0;
    let flm = function(left_middle)?;
    let frm = function(right_middle)?;
    let left = (middle - a) / 6.0 * (fa + 4.0 * flm + fm);
    let right = (b - middle) / 6.0 * (fm + 4.0 * frm + fb);
    let whole = (b - a) / 6.0 * (fa + 4.0 * fm + fb);
    let delta = left + right - whole;

    if depth == 0 || delta.abs() < 15.0 * tolerance {
        return Ok(left + right + delta / 15.0);
    }

    let left_value = adaptive_simpson(
        function,
        a,
        middle,
        fa,
        flm,
        fm,
        tolerance / 2.0,
        depth - 1,
    )?;
    let right_value = adaptive_simpson(
        function,
        middle,
        b,
        fm,
        frm,
        fb,
        tolerance / 2.0,
        depth - 1,
    )?;
    Ok(left_value + right_value)
}

fn evaluate_solve(args: &[Expr], environment: &mut Environment) -> Result<Value, EvalError> {
    if args.len() != 1 && args.len() != 2 {
        return Err(EvalError::InvalidArity("solve".into()));
    }
    let variable = if args.len() == 2 {
        symbol_name(&args[1], "solve")?
    } else {
        math_symbolic::single_variable(&args[0]).unwrap_or_else(|| "x".into())
    };
    let equation = match &args[0] {
        Expr::Binary {
            op: BinaryOp::Eq,
            lhs,
            rhs,
        } => Expr::binary(BinaryOp::Sub, *lhs.clone(), *rhs.clone()),
        other => other.clone(),
    };

    if let Some(mut coefficients) = math_symbolic::polynomial_coefficients(&equation, &variable) {
        while coefficients.len() > 1
            && coefficients
                .last()
                .is_some_and(|value| value.abs() < 1e-12)
        {
            coefficients.pop();
        }
        if coefficients.len() == 1 {
            return Ok(Value::Text(if coefficients[0].abs() < 1e-12 {
                "infinitas soluciones".into()
            } else {
                "sin solución".into()
            }));
        }
        let roots = math_symbolic::polynomial_roots(&coefficients);
        return Ok(Value::Text(format_roots(&variable, &roots)));
    }

    let roots = numeric_roots(&equation, &variable, -100.0, 100.0, environment)?;
    if roots.is_empty() {
        return Ok(Value::Text("sin raíces reales en [-100, 100]".into()));
    }
    Ok(Value::Text(format_roots(
        &variable,
        &roots
            .into_iter()
            .map(|root| (root, 0.0))
            .collect::<Vec<_>>(),
    )))
}

fn evaluate_roots(args: &[Expr], environment: &mut Environment) -> Result<Value, EvalError> {
    if args.len() != 4 {
        return Err(EvalError::InvalidArity("roots".into()));
    }
    let variable = symbol_name(&args[1], "roots")?;
    let lower = value_as_real(&evaluate_with(&args[2], environment)?)?;
    let upper = value_as_real(&evaluate_with(&args[3], environment)?)?;
    let roots = numeric_roots(&args[0], &variable, lower, upper, environment)?;
    Ok(Value::Text(format_roots(
        &variable,
        &roots
            .into_iter()
            .map(|root| (root, 0.0))
            .collect::<Vec<_>>(),
    )))
}

fn evaluate_polyroots(args: &[Expr], environment: &mut Environment) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidArity("polyroots".into()));
    }
    let value = evaluate_with(&args[0], environment)?;
    let Value::Matrix(rows) = value else {
        return Err(EvalError::Domain(
            "polyroots expects a vector of coefficients".into(),
        ));
    };
    let coefficients = if rows.len() == 1 {
        rows[0].clone()
    } else if rows.iter().all(|row| row.len() == 1) {
        rows.iter().map(|row| row[0]).collect()
    } else {
        return Err(EvalError::Domain(
            "polyroots expects a row or column vector".into(),
        ));
    };
    let roots = math_symbolic::polynomial_roots(&coefficients);
    Ok(Value::Text(format_roots("x", &roots)))
}

fn numeric_roots(
    expression: &Expr,
    variable: &str,
    lower: f64,
    upper: f64,
    environment: &Environment,
) -> Result<Vec<f64>, EvalError> {
    if !lower.is_finite() || !upper.is_finite() || upper <= lower {
        return Err(EvalError::Domain(
            "roots expects a finite interval with upper > lower".into(),
        ));
    }

    let mut local = environment.clone();
    let mut evaluate_at = |value: f64| -> Result<f64, EvalError> {
        local.set(variable.to_string(), Value::Real(value));
        let result = evaluate_with(expression, &mut local)?;
        value_as_real(&result)
    };

    let samples = 2000usize;
    let step = (upper - lower) / samples as f64;
    let mut roots = Vec::new();
    let mut previous_x = lower;
    let mut previous_value = evaluate_at(previous_x)?;

    if previous_value.abs() < 1e-9 {
        roots.push(previous_x);
    }

    for index in 1..=samples {
        let current_x = lower + step * index as f64;
        let current_value = evaluate_at(current_x)?;
        if current_value.abs() < 1e-9 {
            push_root(&mut roots, current_x);
        } else if previous_value * current_value < 0.0 {
            let mut left = previous_x;
            let mut right = current_x;
            let mut left_value = previous_value;
            for _ in 0..80 {
                let middle = (left + right) / 2.0;
                let middle_value = evaluate_at(middle)?;
                if middle_value.abs() < 1e-12 || (right - left).abs() < 1e-12 {
                    left = middle;
                    right = middle;
                    break;
                }
                if left_value * middle_value <= 0.0 {
                    right = middle;
                } else {
                    left = middle;
                    left_value = middle_value;
                }
            }
            push_root(&mut roots, (left + right) / 2.0);
        }
        previous_x = current_x;
        previous_value = current_value;
    }

    Ok(roots)
}

fn push_root(roots: &mut Vec<f64>, value: f64) {
    if !roots.iter().any(|root| (root - value).abs() < 1e-6) {
        roots.push(value);
    }
}

fn format_roots(variable: &str, roots: &[(f64, f64)]) -> String {
    if roots.is_empty() {
        return "sin solución".into();
    }
    roots
        .iter()
        .map(|(re, im)| format!("{variable} = {}", format_complex_root(*re, *im)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_complex_root(real: f64, imaginary: f64) -> String {
    if imaginary.abs() < 1e-8 {
        return format_real(real);
    }
    if real.abs() < 1e-8 {
        if (imaginary - 1.0).abs() < 1e-8 {
            return "i".into();
        }
        if (imaginary + 1.0).abs() < 1e-8 {
            return "-i".into();
        }
        return format!("{}i", format_real(imaginary));
    }
    let sign = if imaginary < 0.0 { "-" } else { "+" };
    format!(
        "{} {} {}i",
        format_real(real),
        sign,
        format_real(imaginary.abs())
    )
}

fn format_real(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    if value.abs() < 1e-10 {
        return "0".into();
    }
    let mut text = format!("{value:.10}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

struct LoopParts<'a> {
    variable: String,
    start: i64,
    end: i64,
    body: &'a Expr,
}

fn loop_parts<'a>(
    name: &str,
    args: &'a [Expr],
    environment: &mut Environment,
) -> Result<LoopParts<'a>, EvalError> {
    if args.len() != 4 {
        return Err(EvalError::InvalidArity(name.into()));
    }
    let Expr::Symbol(variable) = &args[0] else {
        return Err(EvalError::InvalidLoop(format!(
            "{name} expects a loop variable as its first argument, for example {name}(i, 1, 10, i^2)"
        )));
    };
    let start = loop_bound(&evaluate_with(&args[1], environment)?)?;
    let end = loop_bound(&evaluate_with(&args[2], environment)?)?;
    if (start as i128 - end as i128).unsigned_abs() > MAX_LOOP_ITERATIONS {
        return Err(EvalError::InvalidLoop(format!(
            "the loop has more than {MAX_LOOP_ITERATIONS} iterations"
        )));
    }
    Ok(LoopParts {
        variable: variable.clone(),
        start,
        end,
        body: &args[3],
    })
}

fn loop_bound(value: &Value) -> Result<i64, EvalError> {
    let number = value_as_real(value)?;
    if !number.is_finite() {
        return Err(EvalError::InvalidLoop(
            "loop bounds must be finite numbers".into(),
        ));
    }
    let rounded = number.round();
    if (number - rounded).abs() > 1e-9 {
        return Err(EvalError::InvalidLoop(
            "loop bounds must be integers".into(),
        ));
    }
    if rounded.abs() > 1e9 {
        return Err(EvalError::InvalidLoop("loop bounds are too large".into()));
    }
    Ok(rounded as i64)
}

fn loop_values(parts: &LoopParts<'_>) -> Vec<i64> {
    if parts.start <= parts.end {
        (parts.start..=parts.end).collect()
    } else {
        (parts.end..=parts.start).rev().collect()
    }
}

fn evaluate_for(args: &[Expr], environment: &mut Environment) -> Result<Value, EvalError> {
    let parts = loop_parts("for", args, environment)?;
    let mut local = environment.clone();
    let mut values = Vec::new();
    for index in loop_values(&parts) {
        local.set(parts.variable.clone(), Value::Real(index as f64));
        let value = evaluate_with(parts.body, &mut local)?;
        values.push(value_as_real(&value)?);
    }
    Ok(Value::Matrix(vec![values]))
}

fn evaluate_reduction(
    name: &str,
    args: &[Expr],
    environment: &mut Environment,
    product: bool,
) -> Result<Value, EvalError> {
    let parts = loop_parts(name, args, environment)?;
    let mut local = environment.clone();
    let initial = if product {
        Value::Real(1.0)
    } else {
        Value::Real(0.0)
    };
    let mut indices = loop_values(&parts).into_iter();
    let Some(first) = indices.next() else {
        return Ok(initial);
    };
    local.set(parts.variable.clone(), Value::Real(first as f64));
    let mut accumulator = evaluate_with(parts.body, &mut local)?;
    for index in indices {
        local.set(parts.variable.clone(), Value::Real(index as f64));
        let value = evaluate_with(parts.body, &mut local)?;
        accumulator = if product {
            mul_values(accumulator, value)?
        } else {
            add_values(accumulator, value)?
        };
    }
    Ok(accumulator)
}

fn evaluate_matrix_for(args: &[Expr], environment: &mut Environment) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::InvalidArity("for".into()));
    }
    let Expr::Symbol(variable) = &args[0] else {
        return Err(EvalError::InvalidLoop(
            "for expects a loop variable as its first argument".into(),
        ));
    };
    let value = evaluate_with(&args[1], environment)?;
    let Value::Matrix(rows) = value else {
        return Err(EvalError::Domain(
            "for over a matrix expects a matrix as its second argument".into(),
        ));
    };
    let cells = rows.into_iter().flatten().collect::<Vec<_>>();
    if cells.len() > MAX_LOOP_ITERATIONS as usize {
        return Err(EvalError::InvalidLoop(format!(
            "the loop has more than {MAX_LOOP_ITERATIONS} iterations"
        )));
    }
    let mut local = environment.clone();
    let mut values = Vec::with_capacity(cells.len());
    for cell in cells {
        local.set(variable.clone(), Value::Real(cell));
        let value = evaluate_with(&args[2], &mut local)?;
        values.push(value_as_real(&value)?);
    }
    Ok(Value::Matrix(vec![values]))
}

fn evaluate_matrix_reduction(
    name: &str,
    args: &[Expr],
    environment: &mut Environment,
    product: bool,
) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::InvalidArity(name.into()));
    }
    let Expr::Symbol(variable) = &args[0] else {
        return Err(EvalError::InvalidLoop(format!(
            "{name} expects a loop variable as its first argument"
        )));
    };
    let value = evaluate_with(&args[1], environment)?;
    let Value::Matrix(rows) = value else {
        return Err(EvalError::Domain(format!(
            "{name} over a matrix expects a matrix as its second argument"
        )));
    };
    let cells = rows.into_iter().flatten().collect::<Vec<_>>();
    if cells.len() > MAX_LOOP_ITERATIONS as usize {
        return Err(EvalError::InvalidLoop(format!(
            "the loop has more than {MAX_LOOP_ITERATIONS} iterations"
        )));
    }
    let mut local = environment.clone();
    let mut accumulator = if product {
        Value::Real(1.0)
    } else {
        Value::Real(0.0)
    };
    for cell in cells {
        local.set(variable.clone(), Value::Real(cell));
        let value = evaluate_with(&args[2], &mut local)?;
        accumulator = if product {
            mul_values(accumulator, value)?
        } else {
            add_values(accumulator, value)?
        };
    }
    Ok(accumulator)
}

fn evaluate_while(args: &[Expr], environment: &mut Environment) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::InvalidArity("while".into()));
    }
    let mut last = Value::Real(0.0);
    let mut iterations = 0u128;
    while truthy(&evaluate_with(&args[0], environment)?)? {
        last = evaluate_with(&args[1], environment)?;
        iterations += 1;
        if iterations > MAX_LOOP_ITERATIONS {
            return Err(EvalError::InvalidLoop(format!(
                "the while loop has more than {MAX_LOOP_ITERATIONS} iterations"
            )));
        }
    }
    Ok(last)
}

fn evaluate_line(args: &[Expr], environment: &mut Environment) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::InvalidArity("line".into()));
    }
    let mut local = environment.clone();
    let mut last = Value::Real(0.0);
    for expression in args {
        last = evaluate_with(expression, &mut local)?;
    }
    Ok(last)
}

fn evaluate_error(args: &[Expr]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidArity("error".into()));
    }
    let message = match &args[0] {
        Expr::Symbol(name) => name.clone(),
        other => math_symbolic::to_ascii(other),
    };
    Err(EvalError::Domain(message))
}

fn abs_value(value: Value) -> Value {
    match value {
        Value::Real(value) => Value::Real(value.abs()),
        Value::Rational(value) => Value::Rational(value.abs()),
        Value::Complex(value) => Value::Real(value.abs()),
        Value::Quantity(quantity) => Value::Quantity(quantity.abs()),
        Value::Matrix(rows) => Value::Matrix(
            rows.into_iter()
                .map(|row| row.into_iter().map(f64::abs).collect())
                .collect(),
        ),
        Value::Text(value) => Value::Text(value),
        Value::Function { name, params } => Value::Function { name, params },
    }
}

fn map_quantity<F: Fn(f64) -> f64>(value: Value, function: F) -> Result<Value, EvalError> {
    match value {
        Value::Real(value) => Ok(Value::Real(function(value))),
        Value::Rational(value) => Ok(Value::Real(function(value.to_f64()))),
        Value::Quantity(quantity) => Ok(Value::Quantity(Quantity::new(
            function(quantity.display_value()),
            quantity.unit,
        ))),
        other => Err(EvalError::Domain(format!(
            "expected a real or quantity, found {}",
            value_kind(&other)
        ))),
    }
}

fn evaluate_builtin(name: &str, values: &[Value]) -> Result<Value, EvalError> {
    match (name, values) {
        ("sqrt", [value]) => match value {
            Value::Complex(value) => Ok(Value::Complex(value.sqrt())),
            Value::Real(value) if *value >= 0.0 => Ok(Value::Real(value.sqrt())),
            Value::Real(value) => Ok(Value::Complex(Complex::new(0.0, (-value).sqrt()))),
            Value::Rational(value) if value.numerator >= 0 => {
                Ok(Value::Real(value.to_f64().sqrt()))
            }
            Value::Quantity(quantity) if quantity.is_dimensionless() => {
                Ok(Value::Real(quantity.value.sqrt()))
            }
            other => Err(EvalError::Domain(format!(
                "sqrt is not defined for {}",
                value_kind(other)
            ))),
        },
        ("abs", [value]) => Ok(abs_value(value.clone())),
        ("sin", [value]) => Ok(Value::Real(value_as_real(value)?.sin())),
        ("cos", [value]) => Ok(Value::Real(value_as_real(value)?.cos())),
        ("tan", [value]) => Ok(Value::Real(value_as_real(value)?.tan())),
        ("asin", [value]) => Ok(Value::Real(value_as_real(value)?.asin())),
        ("acos", [value]) => Ok(Value::Real(value_as_real(value)?.acos())),
        ("atan", [value]) => Ok(Value::Real(value_as_real(value)?.atan())),
        ("atan2", [y, x]) => Ok(Value::Real(value_as_real(y)?.atan2(value_as_real(x)?))),
        ("sinh", [value]) => Ok(Value::Real(value_as_real(value)?.sinh())),
        ("cosh", [value]) => Ok(Value::Real(value_as_real(value)?.cosh())),
        ("tanh", [value]) => Ok(Value::Real(value_as_real(value)?.tanh())),
        ("asinh", [value]) => Ok(Value::Real(value_as_real(value)?.asinh())),
        ("acosh", [value]) => Ok(Value::Real(value_as_real(value)?.acosh())),
        ("atanh", [value]) => Ok(Value::Real(value_as_real(value)?.atanh())),
        ("ln", [value]) => {
            let value = value_as_real(value)?;
            if value <= 0.0 {
                return Err(EvalError::Domain(
                    "ln expects a positive real value".into(),
                ));
            }
            Ok(Value::Real(value.ln()))
        }
        ("log", [value]) => {
            let value = value_as_real(value)?;
            if value <= 0.0 {
                return Err(EvalError::Domain(
                    "log expects a positive real value".into(),
                ));
            }
            Ok(Value::Real(value.log10()))
        }
        ("exp", [value]) => Ok(Value::Real(value_as_real(value)?.exp())),
        ("floor", [value]) => map_quantity(value.clone(), f64::floor),
        ("ceil", [value]) => map_quantity(value.clone(), f64::ceil),
        ("round", [value]) => map_quantity(value.clone(), f64::round),
        ("round", [value, digits]) => {
            let digits = value_as_real(digits)?.round() as i32;
            let factor = 10f64.powi(digits);
            map_quantity(value.clone(), |value| (value * factor).round() / factor)
        }
        ("trunc", [value]) => map_quantity(value.clone(), f64::trunc),
        ("frac", [value]) => map_quantity(value.clone(), f64::fract),
        ("sign", [value]) => map_quantity(value.clone(), f64::signum),
        ("mod", [left, right]) => {
            let left = value_as_real(left)?;
            let right = value_as_real(right)?;
            if right == 0.0 {
                return Err(EvalError::DivisionByZero);
            }
            Ok(Value::Real(left.rem_euclid(right)))
        }
        ("gcd", [left, right]) => Ok(Value::Real(gcd(
            value_as_real(left)?.round() as i64,
            value_as_real(right)?.round() as i64,
        ) as f64)),
        ("lcm", [left, right]) => {
            let left = value_as_real(left)?.round() as i64;
            let right = value_as_real(right)?.round() as i64;
            if left == 0 || right == 0 {
                Ok(Value::Real(0.0))
            } else {
                Ok(Value::Real((left / gcd(left, right) * right).abs() as f64))
            }
        }
        ("factorial", [value]) => {
            let value = value_as_real(value)?.round() as i64;
            if !(0..=170).contains(&value) {
                return Err(EvalError::Domain(
                    "factorial expects an integer between 0 and 170".into(),
                ));
            }
            let mut result = 1.0;
            for factor in 2..=value {
                result *= factor as f64;
            }
            Ok(Value::Real(result))
        }
        ("comb", [n, k]) => {
            let n = value_as_real(n)?.round() as i64;
            let k = value_as_real(k)?.round() as i64;
            if n < 0 || k < 0 || k > n {
                return Err(EvalError::Domain(
                    "comb expects 0 <= k <= n".into(),
                ));
            }
            Ok(Value::Real(combinations(n, k) as f64))
        }
        ("perm", [n, k]) => {
            let n = value_as_real(n)?.round() as i64;
            let k = value_as_real(k)?.round() as i64;
            if n < 0 || k < 0 || k > n {
                return Err(EvalError::Domain(
                    "perm expects 0 <= k <= n".into(),
                ));
            }
            Ok(Value::Real(permutations(n, k) as f64))
        }
        ("min", [_, _, ..]) => select_extreme(values, true),
        ("max", [_, _, ..]) => select_extreme(values, false),
        ("equal", [left, right]) => Ok(Value::Real(if values_equal(left, right) {
            1.0
        } else {
            0.0
        })),
        ("re", [value]) => Ok(Value::Real(value.as_complex().map(|value| value.re).unwrap_or_else(
            || value_as_real(value).unwrap_or(f64::NAN),
        ))),
        ("im", [value]) => Ok(Value::Real(
            value.as_complex().map(|value| value.im).unwrap_or(0.0),
        )),
        ("conj", [value]) => Ok(match value {
            Value::Complex(value) => Value::Complex(value.conj()),
            other => other.clone(),
        }),
        ("det", [Value::Matrix(matrix)]) => Ok(Value::Real(matrix_det(matrix)?)),
        ("inv", [Value::Matrix(matrix)]) => Ok(Value::Matrix(matrix_inverse(matrix)?)),
        ("solve", [Value::Matrix(matrix), Value::Matrix(vector)]) => {
            Ok(Value::Matrix(matrix_solve(matrix, vector)?))
        }
        ("transpose", [Value::Matrix(matrix)]) => Ok(Value::Matrix(matrix_transpose(matrix))),
        ("identity", [value]) => {
            let size = value_as_real(value)?.round();
            if size < 0.0 || size > 1000.0 {
                return Err(EvalError::Domain(
                    "identity size must be between 0 and 1000".into(),
                ));
            }
            Ok(Value::Matrix(identity_matrix(size as usize)))
        }
        ("trace", [Value::Matrix(matrix)]) => Ok(Value::Real(matrix_trace(matrix)?)),
        ("row", [Value::Matrix(matrix), index]) => {
            Ok(Value::Matrix(matrix_row(matrix, index_value(index)?)?))
        }
        ("col", [Value::Matrix(matrix), index]) => {
            Ok(Value::Matrix(matrix_col(matrix, index_value(index)?)?))
        }
        ("augment", [Value::Matrix(left), Value::Matrix(right)]) => {
            Ok(Value::Matrix(matrix_augment(left, right)?))
        }
        ("stack", [Value::Matrix(top), Value::Matrix(bottom)]) => {
            Ok(Value::Matrix(matrix_stack(top, bottom)?))
        }
        ("minor", [Value::Matrix(matrix), row, column]) => Ok(Value::Matrix(matrix_minor(
            matrix,
            index_value(row)?,
            index_value(column)?,
        )?)),
        ("cofactor", [Value::Matrix(matrix), row, column]) => Ok(Value::Real(matrix_cofactor(
            matrix,
            index_value(row)?,
            index_value(column)?,
        )?)),
        ("sum", [Value::Matrix(matrix)]) => Ok(Value::Real(matrix_sum(matrix)?)),
        ("mean", [Value::Matrix(matrix)]) => Ok(Value::Real(matrix_mean(matrix)?)),
        ("range", [start, end]) => {
            let values = range_values(value_as_real(start)?, value_as_real(end)?, 1.0)?;
            Ok(Value::Matrix(vec![values]))
        }
        ("range", [start, end, step]) => {
            let values = range_values(
                value_as_real(start)?,
                value_as_real(end)?,
                value_as_real(step)?,
            )?;
            Ok(Value::Matrix(vec![values]))
        }
        ("matrix", [rows, columns, value]) => Ok(Value::Matrix(filled_matrix(
            index_value(rows)?,
            index_value(columns)?,
            value_as_real(value)?,
        )?)),
        ("resize", [Value::Matrix(matrix), rows, columns]) => Ok(Value::Matrix(resize_matrix(
            matrix,
            index_value(rows)?,
            index_value(columns)?,
        )?)),
        ("reshape", [Value::Matrix(matrix), rows, columns]) => Ok(Value::Matrix(
            reshape_matrix(matrix, index_value(rows)?, index_value(columns)?)?,
        )),
        ("flatten", [Value::Matrix(matrix)]) => Ok(Value::Matrix(vec![matrix
            .iter()
            .flatten()
            .copied()
            .collect()])),
        ("size", [Value::Matrix(matrix)]) => {
            let (rows, columns) = matrix_shape(matrix)?;
            Ok(Value::Matrix(vec![vec![rows as f64, columns as f64]]))
        }
        ("length", [Value::Matrix(matrix)]) => {
            let (rows, columns) = matrix_shape(matrix)?;
            Ok(Value::Real((rows * columns) as f64))
        }
        ("vminor", [Value::Matrix(matrix), row, column]) => Ok(Value::Matrix(matrix_minor(
            matrix,
            index_value(row)?,
            index_value(column)?,
        )?)),
        ("findrows", [Value::Matrix(matrix), value]) => Ok(Value::Matrix(vec![find_rows(
            matrix,
            value_as_real(value)?,
        )?
        .into_iter()
        .map(|row| row as f64)
        .collect()])),
        ("sort" | "sorting", [Value::Matrix(matrix)]) => {
            Ok(Value::Matrix(vec![sorted_values(matrix)?]))
        }
        ("sortrows", [Value::Matrix(matrix), column]) => Ok(Value::Matrix(sort_rows(
            matrix,
            index_value(column)?,
        )?)),
        ("dot", [Value::Matrix(left), Value::Matrix(right)]) => {
            Ok(Value::Real(vector_dot(left, right)?))
        }
        ("cross", [Value::Matrix(left), Value::Matrix(right)]) => {
            Ok(Value::Matrix(vec![vector_cross(left, right)?]))
        }
        ("norm", [Value::Matrix(matrix)]) => Ok(Value::Real(vector_norm(matrix)?)),
        ("lu", [Value::Matrix(matrix)]) => Ok(Value::Text(matrix_lu(matrix)?)),
        ("qr", [Value::Matrix(matrix)]) => Ok(Value::Text(matrix_qr(matrix)?)),
        ("rank", [Value::Matrix(matrix)]) => Ok(Value::Real(matrix_rank(matrix)? as f64)),
        ("rref", [Value::Matrix(matrix)]) => Ok(Value::Matrix(matrix_rref(matrix)?)),
        ("eigenvalues", [Value::Matrix(matrix)]) => {
            Ok(Value::Matrix(vec![matrix_eigenvalues(matrix)?]))
        }
        ("trace", [value]) => Ok(value.clone()),
        ("trace", [_, value]) => Ok(value.clone()),
        (name, _) => Err(EvalError::UnknownFunction(name.to_string())),
    }
}

fn select_extreme(values: &[Value], minimum: bool) -> Result<Value, EvalError> {
    let mut selected = values
        .first()
        .cloned()
        .ok_or_else(|| EvalError::InvalidArity("min/max".into()))?;
    for value in values.iter().skip(1) {
        let (current, candidate) = comparable_values(&selected, value)?;
        if (minimum && candidate < current) || (!minimum && candidate > current) {
            selected = value.clone();
        }
    }
    Ok(selected)
}

fn gcd(mut left: i64, mut right: i64) -> i64 {
    left = left.abs();
    right = right.abs();
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
}

fn combinations(n: i64, k: i64) -> i64 {
    let k = k.min(n - k);
    let mut result = 1i64;
    for index in 0..k {
        result = result.saturating_mul(n - index) / (index + 1);
    }
    result
}

fn permutations(n: i64, k: i64) -> i64 {
    let mut result = 1i64;
    for index in 0..k {
        result = result.saturating_mul(n - index);
    }
    result
}
