use std::collections::HashMap;

use crate::executor::ExecutionError;
use crate::parser::ast::{Expr, SelectColumns, SelectItem};

pub fn bind_select_columns(
    columns: SelectColumns,
    column_index: &HashMap<String, usize>,
) -> Result<SelectColumns, ExecutionError> {
    match columns {
        SelectColumns::All => Ok(SelectColumns::All),
        SelectColumns::Expressions(items) => {
            let mut out = Vec::with_capacity(items.len());
            for SelectItem { expr, alias } in items {
                let bound = bind_expr(expr, column_index)?;
                out.push(SelectItem { expr: bound, alias });
            }
            Ok(SelectColumns::Expressions(out))
        }
    }
}

pub fn bind_where_clause(
    where_clause: Option<Expr>,
    column_index: &HashMap<String, usize>,
) -> Result<Option<Expr>, ExecutionError> {
    where_clause
        .map(|e| bind_expr(e, column_index))
        .transpose()
}

pub fn bind_expr(
    expr: Expr,
    column_index: &HashMap<String, usize>,
) -> Result<Expr, ExecutionError> {
    match expr {
        Expr::Identifier(name) => {
            let idx = column_index
                .get(&name)
                .copied()
                .ok_or_else(|| ExecutionError::ColumnNotFound(name.clone()))?;
            Ok(Expr::ColumnIndex(idx))
        }
        Expr::Number(_) | Expr::StringLit(_) | Expr::ColumnIndex(_) => Ok(expr),
        Expr::BinaryOp { left, op, right } => Ok(Expr::BinaryOp {
            left: Box::new(bind_expr(*left, column_index)?),
            op,
            right: Box::new(bind_expr(*right, column_index)?),
        }),
        Expr::UnaryOp { op, expr } => Ok(Expr::UnaryOp {
            op,
            expr: Box::new(bind_expr(*expr, column_index)?),
        }),
    }
}
