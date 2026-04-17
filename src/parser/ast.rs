use serde::{Deserialize, Serialize};
use crate::executor::ExecutionError;
use crate::storage::data_structures::Value;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    StringLit(String),
    Identifier(String),
    ColumnIndex(usize),
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectItem {
    pub expr: Expr,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectColumns {
    All,
    Expressions(Vec<SelectItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    pub column: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    Varchar(u32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select {
        columns: SelectColumns,
        from: String,
        where_clause: Option<Expr>,
        order_by: Option<OrderBy>,
    },
    CreateTable {
        name: String,
        columns: Vec<ColumnDef>,
    },
    Insert {
        table: String,
        values: Vec<Vec<Expr>>,
    },
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Eq => "=",
            BinOp::Neq => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Lte => "<=",
            BinOp::Gte => ">=",
            BinOp::And => "AND",
            BinOp::Or => "OR",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "NOT",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn precedence(expr: &Expr) -> u8 {
            match expr {
                Expr::BinaryOp { op, .. } => match op {
                    BinOp::Or => 1,
                    BinOp::And => 2,
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => 3,
                    BinOp::Add | BinOp::Sub => 4,
                    BinOp::Mul | BinOp::Div => 5,
                },
                Expr::UnaryOp { .. } => 6,
                _ => 7,
            }
        }

        fn write_child(parent: &Expr, child: &Expr, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let p = precedence(parent);
            let c = precedence(child);
            if c < p {
                write!(f, "({})", child)
            } else {
                write!(f, "{}", child)
            }
        }

        match self {
            Expr::Number(n) => write!(f, "{}", n),
            Expr::StringLit(s) => {
                let escaped = s.replace('\'', "''");
                write!(f, "'{}'", escaped)
            }
            Expr::Identifier(name) => write!(f, "{}", name),
            Expr::ColumnIndex(i) => write!(f, "#{}", i),
            Expr::UnaryOp { op, expr } => {
                match op {
                    UnaryOp::Neg => {
                        write!(f, "-")?;
                        write_child(self, expr, f)
                    }
                    UnaryOp::Not => {
                        write!(f, "NOT ")?;
                        write_child(self, expr, f)
                    }
                }
            }
            Expr::BinaryOp { left, op, right } => {
                write_child(self, left, f)?;
                write!(f, " {} ", op)?;
                write_child(self, right, f)
            }
        }
    }
}
