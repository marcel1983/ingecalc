// SPDX-License-Identifier: GPL-3.0-or-later

use math_ast::{BinaryOp, Expr, UnaryOp};
use math_parse::parse;

pub(crate) enum MathMl {
    Formula(String),
    Fallback(String),
}

pub fn mathml_for(ascii: &str, latex: &str) -> MathMl {
    match parse(ascii) {
        Ok(expression) => {
            let body = expression_to_mathml(&expression);
            MathMl::Formula(format!(
                "<math:math><math:semantics><math:mrow>{body}</math:mrow><math:annotation encoding=\"application/x-tex\">{}</math:annotation></math:semantics></math:math>",
                xml_escape(latex)
            ))
        }
        Err(_) => MathMl::Fallback(format!(
            "<text:span text:style-name=\"MathFallback\">{}</text:span>",
            xml_escape(latex)
        )),
    }
}

fn expression_to_mathml(expression: &Expr) -> String {
    match expression {
        Expr::Number(value) => format!("<math:mn>{}</math:mn>", format_number(*value)),
        Expr::Symbol(name) => symbol_mathml(name),
        Expr::Unary { op, expr } => match op {
            UnaryOp::Neg => format!(
                "<math:mrow><math:mo>-</math:mo>{}</math:mrow>",
                expression_to_mathml(expr)
            ),
            UnaryOp::Pos => expression_to_mathml(expr),
        },
        Expr::Binary { op, lhs, rhs } => {
            let left = expression_to_mathml(lhs);
            let right = expression_to_mathml(rhs);
            match op {
                BinaryOp::Add => {
                    format!("<math:mrow>{left}<math:mo>+</math:mo>{right}</math:mrow>")
                }
                BinaryOp::Sub => {
                    format!("<math:mrow>{left}<math:mo>-</math:mo>{right}</math:mrow>")
                }
                BinaryOp::Mul => {
                    format!("<math:mrow>{left}<math:mo>&#x00D7;</math:mo>{right}</math:mrow>")
                }
                BinaryOp::Div => {
                    format!("<math:mfrac>{left}{right}</math:mfrac>")
                }
                BinaryOp::Pow => {
                    format!("<math:msup>{left}{right}</math:msup>")
                }
                BinaryOp::Lt => {
                    format!("<math:mrow>{left}<math:mo>&lt;</math:mo>{right}</math:mrow>")
                }
                BinaryOp::Le => {
                    format!("<math:mrow>{left}<math:mo>&#x2264;</math:mo>{right}</math:mrow>")
                }
                BinaryOp::Gt => {
                    format!("<math:mrow>{left}<math:mo>&gt;</math:mo>{right}</math:mrow>")
                }
                BinaryOp::Ge => {
                    format!("<math:mrow>{left}<math:mo>&#x2265;</math:mo>{right}</math:mrow>")
                }
                BinaryOp::Eq => {
                    format!("<math:mrow>{left}<math:mo>=</math:mo>{right}</math:mrow>")
                }
                BinaryOp::Ne => {
                    format!("<math:mrow>{left}<math:mo>&#x2260;</math:mo>{right}</math:mrow>")
                }
            }
        }
        Expr::Call { name, args } => {
            if name.as_str() == "sqrt" && args.len() == 1 {
                return format!(
                    "<math:msqrt>{}</math:msqrt>",
                    expression_to_mathml(&args[0])
                );
            }

            if name.as_str() == "abs" && args.len() == 1 {
                return format!(
                    "<math:mrow><math:mo>|</math:mo>{}<math:mo>|</math:mo></math:mrow>",
                    expression_to_mathml(&args[0])
                );
            }

            let arguments = args
                .iter()
                .map(expression_to_mathml)
                .collect::<Vec<_>>()
                .join("<math:mo>,</math:mo>");

            format!(
                "{}<math:mo>(</math:mo><math:mrow>{arguments}</math:mrow><math:mo>)</math:mo>",
                symbol_mathml(name)
            )
        }
        Expr::List(items) => {
            if let Some(rows) = nested_rows(items) {
                return matrix_mathml(rows);
            }
            matrix_mathml(vec![items.as_slice()])
        }
        Expr::Matrix(rows) => matrix_mathml(rows.iter().map(|row| row.as_slice()).collect()),
        Expr::Index { target, indices } => {
            let indices = indices
                .iter()
                .map(expression_to_mathml)
                .collect::<Vec<_>>()
                .join("<math:mo>,</math:mo>");
            format!(
                "<math:mrow>{}<math:mo>[</math:mo>{}<math:mo>]</math:mo></math:mrow>",
                expression_to_mathml(target),
                indices
            )
        }
        Expr::Assign { name, value } => format!(
            "<math:mrow>{}<math:mo>:</math:mo><math:mo>=</math:mo>{}</math:mrow>",
            symbol_mathml(name),
            expression_to_mathml(value)
        ),
        Expr::FunctionDef { name, params, body } => {
            let params = params
                .iter()
                .map(|param| symbol_mathml(param))
                .collect::<Vec<_>>()
                .join("<math:mo>,</math:mo>");
            format!(
                "<math:mrow>{}<math:mo>(</math:mo>{}<math:mo>)</math:mo><math:mo>:</math:mo><math:mo>=</math:mo>{}</math:mrow>",
                symbol_mathml(name),
                params,
                expression_to_mathml(body)
            )
        }
        Expr::UnitDef { name, value } => format!(
            "<math:mrow>{}<math:mo>(</math:mo>{}<math:mo>)</math:mo><math:mo>:</math:mo><math:mo>=</math:mo>{}</math:mrow>",
            symbol_mathml("unit"),
            symbol_mathml(name),
            expression_to_mathml(value)
        ),
    }
}

fn symbol_mathml(name: &str) -> String {
    let greek = match name {
        "alpha" => Some("α"),
        "beta" => Some("β"),
        "gamma" => Some("γ"),
        "delta" => Some("δ"),
        "epsilon" | "varepsilon" => Some("ε"),
        "zeta" => Some("ζ"),
        "eta" => Some("η"),
        "theta" | "vartheta" => Some("θ"),
        "iota" => Some("ι"),
        "kappa" => Some("κ"),
        "lambda" => Some("λ"),
        "mu" => Some("μ"),
        "nu" => Some("ν"),
        "xi" => Some("ξ"),
        "omicron" => Some("ο"),
        "pi" => Some("π"),
        "rho" => Some("ρ"),
        "sigma" | "varsigma" => Some("σ"),
        "tau" => Some("τ"),
        "upsilon" => Some("υ"),
        "phi" | "varphi" => Some("φ"),
        "chi" => Some("χ"),
        "psi" => Some("ψ"),
        "omega" => Some("ω"),
        "Gamma" => Some("Γ"),
        "Delta" => Some("Δ"),
        "Theta" => Some("Θ"),
        "Lambda" => Some("Λ"),
        "Xi" => Some("Ξ"),
        "Pi" => Some("Π"),
        "Sigma" => Some("Σ"),
        "Upsilon" => Some("Υ"),
        "Phi" => Some("Φ"),
        "Psi" => Some("Ψ"),
        "Omega" => Some("Ω"),
        _ => None,
    };

    if let Some(character) = greek {
        return format!("<math:mi>{character}</math:mi>");
    }

    if name.chars().count() > 1 {
        return format!("<math:mtext>{}</math:mtext>", xml_escape(name));
    }
    format!("<math:mi>{}</math:mi>", xml_escape(name))
}

fn nested_rows(items: &[Expr]) -> Option<Vec<&[Expr]>> {
    let rows = items
        .iter()
        .map(|item| match item {
            Expr::List(row) => Some(row.as_slice()),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let columns = rows.first()?.len();
    if columns == 0 || rows.iter().any(|row| row.len() != columns) {
        return None;
    }
    Some(rows)
}

fn matrix_mathml(rows: Vec<&[Expr]>) -> String {
    let rows = rows
        .iter()
        .map(|row| {
            let cells = row
                .iter()
                .map(expression_to_mathml)
                .map(|cell| format!("<math:mtd>{cell}</math:mtd>"))
                .collect::<String>();
            format!("<math:mtr>{cells}</math:mtr>")
        })
        .collect::<String>();
    format!(
        "<math:mrow><math:mo>(</math:mo><math:mtable>{rows}</math:mtable><math:mo>)</math:mo></math:mrow>"
    )
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        let text = format!("{value}");
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub(crate) fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_nested_lists_as_math_table() {
        match mathml_for("((1,2),(3,4))", "") {
            MathMl::Formula(body) => {
                assert!(body.contains("<math:mtable>"));
                assert!(body.contains("<math:mtd>"));
            }
            MathMl::Fallback(_) => panic!("expected a formula"),
        }
    }

    #[test]
    fn renders_function_calls_with_parentheses_and_greek_names() {
        match mathml_for("fib(1)", "") {
            MathMl::Formula(body) => {
                assert!(body.contains("<math:mtext>fib</math:mtext>"));
                assert!(body.contains("<math:mo>(</math:mo>"));
                assert!(body.contains("<math:mo>)</math:mo>"));
            }
            MathMl::Fallback(_) => panic!("expected a formula"),
        }

        match mathml_for("phi + Gamma", "") {
            MathMl::Formula(body) => {
                assert!(body.contains("φ"));
                assert!(body.contains("Γ"));
            }
            MathMl::Fallback(_) => panic!("expected a formula"),
        }

        match mathml_for("x := 1", "") {
            MathMl::Formula(body) => {
                assert!(body.contains(":</math:mo><math:mo>="));
                assert!(!body.contains(":=</math:mo>"));
            }
            MathMl::Fallback(_) => panic!("expected a formula"),
        }
    }
}
