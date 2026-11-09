// SPDX-License-Identifier: GPL-3.0-or-later

use math_ast::{BinaryOp, Expr, UnaryOp};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("unexpected end of input")]
    UnexpectedEnd,
    #[error("unexpected character '{0}' at {1}")]
    UnexpectedChar(char, usize),
    #[error("unexpected token at {0}")]
    UnexpectedToken(usize),
    #[error("expected '{0}' at {1}")]
    Expected(char, usize),
    #[error("unexpected '=' at {0}; use ':=' to define a variable or omit '=' to evaluate")]
    UnexpectedEquals(usize),
    #[error("empty parentheses at {0}; fill or remove them")]
    EmptyParentheses(usize),
    #[error("assignment target must be a single name at {0}; use a name like x, f_g or \"fg\" and do not assign to an expression")]
    InvalidAssignment(usize),
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Quoted(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Assign,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    End,
}

/// Parse a compact ASCII-math expression.
///
/// Supported syntax:
/// - numbers, identifiers, `+ - * / ^`, parentheses
/// - assignments such as `x := 2`
/// - function calls such as `sqrt(2)`, `sin(x)`, `log(100)`
/// - implicit multiplication such as `2x` or `(x+1)(x-1)`
/// - matrices `[1, 2; 3, 4]`, `[[1,2],[3,4]]` and MathLive's `((1,2),(3,4))`
/// - quoted unit names such as `2"m"` or `30"cm"`
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let normalized = normalize_input(input);
    let trimmed = normalized.trim_end();
    let normalized = trimmed
        .strip_suffix('=')
        .map(str::trim_end)
        .unwrap_or(trimmed);
    let mut parser = Parser::new(normalized)?;
    let expression = parser.parse_expression()?;

    if !matches!(parser.peek(), Token::End) {
        return Err(ParseError::UnexpectedToken(parser.position()));
    }

    Ok(expression)
}

fn normalize_input(input: &str) -> String {
    let normalized = input
        .replace(['≔', '⩴'], ":=")
        .replace(['∶', '：'], ":")
        .replace('−', "-")
        .replace("≤", "<=")
        .replace("≥", ">=")
        .replace("≠", "!=")
        .replace("\\mathop", ":=")
        .replace("-:", "/")
        .replace("^@", "°");

    let mut output = String::with_capacity(normalized.len());
    let mut characters = normalized.chars().peekable();

    while let Some(character) = characters.next() {
        output.push(character);
        if character != ':' {
            continue;
        }

        let mut lookahead = characters.clone();
        while matches!(lookahead.peek(), Some(next) if next.is_whitespace()) {
            lookahead.next();
        }
        if matches!(lookahead.peek(), Some('=')) {
            while matches!(characters.peek(), Some(next) if next.is_whitespace()) {
                characters.next();
            }
            output.push('=');
            characters.next();
        }
    }

    output
}

struct Parser {
    tokens: Vec<(Token, usize)>,
    position: usize,
}

impl Parser {
    fn new(input: &str) -> Result<Self, ParseError> {
        Ok(Self {
            tokens: merge_function_names(tokenize(input)?),
            position: 0,
        })
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position].0
    }

    fn position(&self) -> usize {
        self.tokens[self.position].1
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.position].0.clone();
        self.position += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let position = self.position();
        let found = self.bump();
        if found == expected {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(position))
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_comparison()?;

        if matches!(self.peek(), Token::Assign) {
            let position = self.position();
            self.bump();

            let definition = match left {
                Expr::Symbol(name) => Expr::Assign {
                    name,
                    value: Box::new(self.parse_expression()?),
                },
                Expr::Call { name, args } => {
                    if name == "unit" {
                        if args.len() != 1 {
                            return Err(ParseError::InvalidAssignment(position));
                        }
                        let Expr::Symbol(unit_name) = args.into_iter().next().expect("one arg")
                        else {
                            return Err(ParseError::InvalidAssignment(position));
                        };
                        return Ok(Expr::UnitDef {
                            name: unit_name,
                            value: Box::new(self.parse_expression()?),
                        });
                    }

                    let mut params = Vec::with_capacity(args.len());
                    for arg in args {
                        let Expr::Symbol(param) = arg else {
                            return Err(ParseError::InvalidAssignment(position));
                        };
                        params.push(param);
                    }
                    Expr::FunctionDef {
                        name,
                        params,
                        body: Box::new(self.parse_expression()?),
                    }
                }
                _ => return Err(ParseError::InvalidAssignment(position)),
            };

            return Ok(definition);
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expression = self.parse_additive()?;

        loop {
            let operator = match self.peek() {
                Token::Less => BinaryOp::Lt,
                Token::LessEqual => BinaryOp::Le,
                Token::Greater => BinaryOp::Gt,
                Token::GreaterEqual => BinaryOp::Ge,
                Token::EqualEqual => BinaryOp::Eq,
                Token::NotEqual => BinaryOp::Ne,
                _ => break,
            };
            self.bump();
            let right = self.parse_additive()?;
            expression = Expr::binary(operator, expression, right);
        }

        Ok(expression)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut expression = self.parse_term()?;

        loop {
            match self.peek().clone() {
                Token::Plus => {
                    self.bump();
                    let right = self.parse_term()?;
                    expression = Expr::binary(BinaryOp::Add, expression, right);
                }
                Token::Minus => {
                    self.bump();
                    let right = self.parse_term()?;
                    expression = Expr::binary(BinaryOp::Sub, expression, right);
                }
                _ => break,
            }
        }

        Ok(expression)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut expression = self.parse_unary()?;

        loop {
            match self.peek().clone() {
                Token::Star => {
                    self.bump();
                    let right = self.parse_unary()?;
                    expression = Expr::binary(BinaryOp::Mul, expression, right);
                }
                Token::Slash => {
                    self.bump();
                    let right = self.parse_unary()?;
                    expression = Expr::binary(BinaryOp::Div, expression, right);
                }
                Token::Number(_)
                | Token::Ident(_)
                | Token::Quoted(_)
                | Token::LParen
                | Token::LBracket => {
                    let right = self.parse_unary()?;
                    expression = Expr::binary(BinaryOp::Mul, expression, right);
                }
                _ => break,
            }
        }

        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Token::Plus => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnaryOp::Pos,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            Token::Minus => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            _ => self.parse_power(),
        }
    }

    fn parse_power(&mut self) -> Result<Expr, ParseError> {
        let base = self.parse_postfix()?;

        if matches!(self.peek(), Token::Caret) {
            self.bump();
            let exponent = self.parse_unary()?;
            Ok(Expr::binary(BinaryOp::Pow, base, exponent))
        } else {
            Ok(base)
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expression = self.parse_primary()?;

        while matches!(self.peek(), Token::LBracket) && is_indexable(&expression) {
            self.bump();
            let mut indices = Vec::new();

            if !matches!(self.peek(), Token::RBracket) {
                loop {
                    indices.push(self.parse_expression()?);
                    if matches!(self.peek(), Token::Comma) {
                        self.bump();
                        continue;
                    }
                    break;
                }
            }

            self.expect(Token::RBracket)?;
            expression = Expr::Index {
                target: Box::new(expression),
                indices,
            };
        }

        Ok(expression)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.bump() {
            Token::Number(value) => Ok(Expr::Number(value)),
            Token::Quoted(name) => {
                if matches!(self.peek(), Token::LParen) {
                    Ok(Expr::Call {
                        name,
                        args: self.parse_call_arguments()?,
                    })
                } else {
                    Ok(Expr::Symbol(name))
                }
            }
            Token::Ident(name) => {
                if matches!(self.peek(), Token::LParen) {
                    Ok(Expr::Call {
                        name,
                        args: self.parse_call_arguments()?,
                    })
                } else {
                    Ok(Expr::Symbol(name))
                }
            }
            Token::LParen => {
                if let Some(matrix) = self.try_parse_grouped_matrix() {
                    return Ok(matrix);
                }
                if matches!(self.peek(), Token::RParen) {
                    return Err(ParseError::EmptyParentheses(
                        self.position().saturating_sub(1),
                    ));
                }
                let first = self.parse_expression()?;
                let mut items = vec![first];

                while matches!(self.peek(), Token::Comma) {
                    self.bump();
                    items.push(self.parse_expression()?);
                }

                self.expect(Token::RParen)?;
                if items.len() == 1 {
                    Ok(items.remove(0))
                } else {
                    Ok(Expr::List(items))
                }
            }
            Token::LBracket => self.parse_matrix_literal(),
            Token::End => Err(ParseError::UnexpectedEnd),
            _ => Err(ParseError::UnexpectedToken(
                self.position().saturating_sub(1),
            )),
        }
    }

    fn try_parse_grouped_matrix(&mut self) -> Option<Expr> {
        let saved = self.position;
        // The outer opening parenthesis has already been consumed by `parse_primary`.
        if !matches!(self.peek(), Token::LParen) {
            return None;
        }
        let mut groups: Vec<Vec<Expr>> = Vec::new();

        loop {
            if !matches!(self.peek(), Token::LParen) {
                self.position = saved;
                return None;
            }
            self.bump();
            if matches!(self.peek(), Token::RParen) {
                self.position = saved;
                return None;
            }
            let mut group = Vec::new();
            loop {
                match self.parse_expression() {
                    Ok(value) => group.push(value),
                    Err(_) => {
                        self.position = saved;
                        return None;
                    }
                }
                if matches!(self.peek(), Token::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
            if !matches!(self.peek(), Token::RParen) {
                self.position = saved;
                return None;
            }
            self.bump();
            groups.push(group);

            if matches!(self.peek(), Token::Comma) {
                self.bump();
                continue;
            }
            break;
        }

        if !matches!(self.peek(), Token::RParen) {
            self.position = saved;
            return None;
        }
        self.bump();

        if groups.len() < 2 {
            self.position = saved;
            return None;
        }

        let columns = groups[0].len();
        let column_vector = groups.iter().all(|group| group.len() == 1);
        let rectangular = groups.iter().all(|group| group.len() == columns);
        if column_vector || rectangular {
            Some(Expr::Matrix(groups))
        } else {
            self.position = saved;
            None
        }
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Expr>, ParseError> {
        self.expect(Token::LParen)?;
        let mut args = Vec::new();

        if !matches!(self.peek(), Token::RParen) {
            loop {
                args.push(self.parse_expression()?);
                if matches!(self.peek(), Token::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
        }

        self.expect(Token::RParen)?;
        Ok(args)
    }

    fn parse_matrix_literal(&mut self) -> Result<Expr, ParseError> {
        let mut rows = vec![Vec::new()];

        if matches!(self.peek(), Token::RBracket) {
            self.bump();
            return Ok(Expr::List(Vec::new()));
        }

        loop {
            let value = self.parse_expression()?;
            rows.last_mut()
                .expect("matrix rows are never empty")
                .push(value);

            match self.peek().clone() {
                Token::Comma => {
                    self.bump();
                }
                Token::Semicolon => {
                    self.bump();
                    rows.push(Vec::new());
                }
                Token::RBracket => break,
                _ => return Err(ParseError::UnexpectedToken(self.position())),
            }
        }

        self.expect(Token::RBracket)?;
        if rows.len() == 1 {
            Ok(Expr::List(rows.remove(0)))
        } else {
            Ok(Expr::Matrix(rows))
        }
    }
}

fn is_indexable(expression: &Expr) -> bool {
    !matches!(expression, Expr::Number(_))
}

fn merge_function_names(tokens: Vec<(Token, usize)>) -> Vec<(Token, usize)> {
    let mut result = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        let start = index;
        let mut letters = Vec::new();

        while let Some((Token::Ident(name), position)) = tokens.get(index) {
            if name.chars().count() != 1 || !name.chars().all(char::is_alphabetic) {
                break;
            }
            letters.push((name.clone(), *position));
            index += 1;
        }

        if !letters.is_empty() {
            let combined = letters
                .iter()
                .map(|(letter, _)| letter.as_str())
                .collect::<String>();
            let is_unit = is_known_unit(&combined);
            let is_function = is_multiletter_function(&combined)
                && matches!(tokens.get(index), Some((Token::LParen, _)));

            if letters.len() > 1 && (is_unit || is_function) {
                result.push((Token::Ident(combined), letters[0].1));
                continue;
            }

            result.extend(tokens[start..index].iter().cloned());
            continue;
        }

        result.push(tokens[index].clone());
        index += 1;
    }

    result
}

fn is_multiletter_function(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "sqrt"
            | "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "arcsin"
            | "arccos"
            | "arctan"
            | "atan2"
            | "sinh"
            | "cosh"
            | "tanh"
            | "asinh"
            | "acosh"
            | "atanh"
            | "ln"
            | "log"
            | "exp"
            | "abs"
            | "floor"
            | "ceil"
            | "round"
            | "trunc"
            | "frac"
            | "sign"
            | "mod"
            | "gcd"
            | "lcm"
            | "factorial"
            | "comb"
            | "perm"
            | "min"
            | "max"
            | "equal"
            | "re"
            | "im"
            | "conj"
            | "unit"
            | "det"
            | "transpose"
            | "identity"
            | "inv"
            | "solve"
            | "trace"
            | "row"
            | "col"
            | "augment"
            | "stack"
            | "minor"
            | "cofactor"
            | "sum"
            | "mean"
            | "if"
            | "si"
            | "for"
            | "para"
            | "product"
            | "producto"
            | "sumatoria"
            | "text"
            | "simplify"
            | "expand"
            | "factor"
            | "subs"
            | "substitute"
            | "diff"
            | "derivative"
            | "integrate"
            | "integral"
            | "roots"
            | "polyroots"
            | "eval"
            | "while"
            | "mientras"
            | "line"
            | "bloque"
            | "error"
    )
}

fn is_known_unit(name: &str) -> bool {
    math_units::UnitRegistry::global().lookup(name).is_some()
}

fn tokenize(input: &str) -> Result<Vec<(Token, usize)>, ParseError> {
    let characters: Vec<char> = input.chars().collect();
    let mut index = 0;
    let mut tokens = Vec::new();

    while index < characters.len() {
        let character = characters[index];

        if character.is_whitespace() {
            index += 1;
            continue;
        }

        if character.is_ascii_digit() || character == '.' {
            let start = index;
            let mut seen_dot = false;

            while index < characters.len() {
                let current = characters[index];
                if current.is_ascii_digit() {
                    index += 1;
                } else if current == '.' && !seen_dot {
                    seen_dot = true;
                    index += 1;
                } else {
                    break;
                }
            }

            let text: String = characters[start..index].iter().collect();
            let value = text
                .parse::<f64>()
                .map_err(|_| ParseError::UnexpectedToken(start))?;
            tokens.push((Token::Number(value), start));
            continue;
        }

        if character == '"' {
            let start = index;
            index += 1;
            let mut text = String::new();

            while index < characters.len() && characters[index] != '"' {
                text.push(characters[index]);
                index += 1;
            }

            if index >= characters.len() {
                return Err(ParseError::UnexpectedEnd);
            }
            index += 1;
            tokens.push((Token::Quoted(text), start));
            continue;
        }

        if character.is_alphabetic() || character == '_' || character == '°' {
            let start = index;

            while index < characters.len()
                && (characters[index].is_alphanumeric()
                    || characters[index] == '_'
                    || characters[index] == '°')
            {
                index += 1;
            }

            let text: String = characters[start..index].iter().collect();
            tokens.push((Token::Ident(text), start));
            continue;
        }

        let token = match character {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' | '×' | '·' | '∗' => Token::Star,
            '/' | '÷' => Token::Slash,
            '^' => Token::Caret,
            ':' if characters.get(index + 1) == Some(&'=') => {
                index += 1;
                Token::Assign
            }
            '<' => {
                if characters.get(index + 1) == Some(&'=') {
                    index += 1;
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            '>' => {
                if characters.get(index + 1) == Some(&'=') {
                    index += 1;
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            '!' if characters.get(index + 1) == Some(&'=') => {
                index += 1;
                Token::NotEqual
            }
            '=' if characters.get(index + 1) == Some(&'=') => {
                index += 1;
                Token::EqualEqual
            }
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '%' => Token::Ident("%".into()),
            '=' => return Err(ParseError::UnexpectedEquals(index)),
            _ => return Err(ParseError::UnexpectedChar(character, index)),
        };

        tokens.push((token, index));
        index += 1;
    }

    tokens.push((Token::End, characters.len()));
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_arithmetic() {
        let expression = parse("2 + 3 * 4").expect("expression should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn normalizes_mathlive_division_symbol() {
        let expression = parse("5 -: 2").expect("division should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Div,
                ..
            }
        ));
    }

    #[test]
    fn parses_comparisons() {
        let expression = parse("2 + 3 < 4 * 5").expect("comparison should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Lt,
                ..
            }
        ));

        let expression = parse("x >= 1").expect("comparison should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Ge,
                ..
            }
        ));

        let expression = parse("a != b").expect("comparison should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Ne,
                ..
            }
        ));
    }

    #[test]
    fn parses_comparison_as_assignment_value() {
        let expression = parse("ok := if(x == 1, 10, 20)").expect("assignment should parse");
        assert!(matches!(
            expression,
            Expr::Assign { name, .. } if name == "ok"
        ));
    }

    #[test]
    fn parses_function_calls() {
        let expression = parse("sqrt(2) + sin(x)").expect("expression should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn parses_implicit_multiplication() {
        let expression = parse("2x").expect("expression should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn rejects_trailing_input() {
        assert!(parse("2 +").is_err());
    }

    #[test]
    fn parses_assignment() {
        let expression = parse("x := 2").expect("assignment should parse");
        assert!(matches!(
            expression,
            Expr::Assign { name, .. } if name == "x"
        ));
    }

    #[test]
    fn parses_spaced_and_unicode_assignments() {
        for source in ["x : = 9", "x ≔ 9", "x ∶= 9", "x\\mathop 5"] {
            let expression = parse(source).expect("assignment should parse");
            assert!(matches!(
                expression,
                Expr::Assign { name, .. } if name == "x"
            ));
        }
    }

    #[test]
    fn treats_a_trailing_equals_as_an_evaluation_marker() {
        assert!(parse("x^2 =").is_ok());
        assert!(parse("x = 45").is_err());
    }

    #[test]
    fn reports_helpful_equals_and_assignment_errors() {
        assert!(matches!(
            parse("x^2 = 45"),
            Err(ParseError::UnexpectedEquals(4))
        ));
        assert!(matches!(
            parse("x^2 := 45"),
            Err(ParseError::InvalidAssignment(_))
        ));
        assert!(matches!(parse("5()"), Err(ParseError::EmptyParentheses(_))));
    }

    #[test]
    fn parses_function_definition() {
        let expression = parse("f(x, y) := x^2 + y").expect("function should parse");
        assert!(matches!(
            expression,
            Expr::FunctionDef { name, params, .. }
                if name == "f" && params == vec!["x".to_string(), "y".to_string()]
        ));
    }

    #[test]
    fn parses_unit_definition() {
        let expression =
            parse("unit(mi_pulgada) := 2.54 cm").expect("unit definition should parse");
        assert!(matches!(
            expression,
            Expr::UnitDef { name, .. } if name == "mi_pulgada"
        ));
    }

    #[test]
    fn parses_matrix_literals() {
        let expression = parse("[1, 2; 3, 4]").expect("matrix should parse");
        assert!(matches!(expression, Expr::Matrix(rows) if rows.len() == 2));

        let expression = parse("[[1, 2], [3, 4]]").expect("nested list should parse");
        assert!(matches!(expression, Expr::List(items) if items.len() == 2));

        let expression = parse("((1,2),(3,4))").expect("MathLive matrix should parse");
        assert!(matches!(expression, Expr::Matrix(rows) if rows.len() == 2 && rows[0].len() == 2));

        let expression = parse("((5),(6))").expect("MathLive column vector should parse");
        assert!(matches!(expression, Expr::Matrix(rows) if rows.len() == 2 && rows[0].len() == 1));
    }

    #[test]
    fn parses_quoted_units() {
        let expression = parse("2\"m\" + 30\"cm\"").expect("units should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn parses_quoted_function_names() {
        let expression = parse("\"transpose\"([1,2;3,4])").expect("quoted call should parse");
        assert!(matches!(
            expression,
            Expr::Call { name, .. } if name == "transpose"
        ));
    }

    #[test]
    fn parses_index_access() {
        let expression = parse("A[1, 2]").expect("index should parse");
        assert!(matches!(
            expression,
            Expr::Index { indices, .. } if indices.len() == 2
        ));

        let expression = parse("2[1, 2]").expect("scalar times matrix should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn merges_split_multiletter_function_names() {
        let expression = parse("d e t([1,2;3,4])").expect("split det should parse");
        assert!(matches!(
            expression,
            Expr::Call { name, .. } if name == "det"
        ));
    }

    #[test]
    fn merges_split_localized_unit_names() {
        let expression = parse("2 m e t r o").expect("split metro should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));

        let expression = parse("1 c m").expect("split cm should parse");
        assert!(matches!(
            expression,
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }
}
