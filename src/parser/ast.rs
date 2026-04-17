use serde::{Deserialize, Serialize};
use crate::executor::ExecutionError;
use crate::storage::data_structures::Value;

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
