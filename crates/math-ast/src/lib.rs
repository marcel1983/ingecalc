// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

/// A minimal expression tree for the MVP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Number(f64),
    Symbol(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    List(Vec<Expr>),
    /// Matrix literal with explicit rows.
    Matrix(Vec<Vec<Expr>>),
    /// Index access: `target[indices]` with 1-based indices.
    Index {
        target: Box<Expr>,
        indices: Vec<Expr>,
    },
    /// Variable definition: `name := value`.
    Assign {
        name: String,
        value: Box<Expr>,
    },
    /// User function definition: `name(params) := body`.
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Box<Expr>,
    },
    /// User unit definition: `unit(name) := value`.
    UnitDef {
        name: String,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Pos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl Expr {
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Symbol(name.into())
    }

    pub fn binary(op: BinaryOp, lhs: Expr, rhs: Expr) -> Self {
        Self::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
}
