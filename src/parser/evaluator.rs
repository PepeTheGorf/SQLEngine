use std::collections::HashMap;
use crate::parser::ast::{BinOp, Expr, UnaryOp};
use crate::storage::data_structures::Value;

struct Evaluator {

}

impl Evaluator {
    pub fn evaluate(expression: Expr, row_values: &HashMap<String, Value>) -> Value {
        match expression {
            Expr::Number(n) => Value::Integer(n),
            Expr::StringLit(s) => Value::Varchar(s),
            Expr::Identifier(name) => {
                row_values.get(&name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Undefined identifier '{}'", name))
            }
            Expr::BinaryOp { left, op, right } => {
                let left_val = Self::evaluate(*left, row_values);
                let right_val = Self::evaluate(*right, row_values);
                match (left_val, right_val) {
                    (Value::Integer(l), Value::Integer(r)) => {
                        match op {
                            BinOp::Add => {
                                Value::Integer(l + r)
                            }
                            BinOp::Sub => {
                                Value::Integer(l - r)
                            }
                            BinOp::Mul => {
                                Value::Integer(l * r)
                            }
                            BinOp::Div => {
                                Value::Integer(l / r)
                            }
                            BinOp::Eq => {
                                Value::Integer((l == r) as i64)
                            }
                            BinOp::Neq => {
                                Value::Integer((l != r) as i64)
                            }
                            BinOp::Lt => {
                                Value::Integer((l < r) as i64)
                            }
                            BinOp::Gt => {
                                Value::Integer((l > r) as i64)
                            }
                            BinOp::Lte => {
                                Value::Integer((l <= r) as i64)
                            }
                            BinOp::Gte => {
                                Value::Integer((l >= r) as i64)
                            }
                            BinOp::And => {
                                    Value::Integer(((l != 0) && (r != 0)) as i64)
                            }
                            BinOp::Or => {
                                    Value::Integer(((l != 0) || (r != 0)) as i64)
                            }
                        }
                    }
                    _ => panic!("Unsupported operand types for operator '{:?}'", op),
                }
            }
            Expr::UnaryOp { op, expr } => {
                let val = Self::evaluate(*expr, row_values);
                match (op, val) {
                    (UnaryOp::Neg, Value::Integer(n)) => Value::Integer(-n),
                    (UnaryOp::Not, Value::Integer(n)) => Value::Integer((n == 0) as i64),
                    _ => panic!("Unsupported operand type for unary operator '{:?}'", op),
                }
            }
        }
    }
}