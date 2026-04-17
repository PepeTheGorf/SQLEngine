use std::fmt;

use crate::parser::ast::{BinOp, Expr, UnaryOp};
use crate::storage::data_structures::Value;

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(expression: &Expr, row_values: &[Value]) -> Result<Value, EvaluationError> {
        match expression {
            Expr::Number(n) => Ok(Value::Integer(*n)),
            Expr::StringLit(s) => Ok(Value::Varchar(s.clone())),
            Expr::ColumnIndex(index) => Ok(row_values[*index].clone()),
            Expr::BinaryOp { left, op, right } => {
                let left_val = Self::evaluate(left, row_values)?;
                let right_val = Self::evaluate(right, row_values)?;
                Self::apply_binary_op(left_val, op.clone(), right_val)
            }
            Expr::UnaryOp { op, expr } => {
                let val = Self::evaluate(expr, row_values)?;
                Self::apply_unary_op(op.clone(), val)
            }
            _ => Err(EvaluationError::NoRowContext(format!("{:?}", expression))),
        }
    }

    fn apply_binary_op(left: Value, op: BinOp, right: Value) -> Result<Value, EvaluationError> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                let result = match op {
                    BinOp::Add =>  {
                        l + r
                    }
                    BinOp::Sub => {
                        l - r
                    }
                    BinOp::Mul => {
                        l * r
                    }
                    BinOp::Div => {
                        if r == 0 {
                            return Err(EvaluationError::DivisionByZero);
                        }
                        l / r
                    }
                    BinOp::Eq => (l == r) as i64,
                    BinOp::Neq => (l != r) as i64,
                    BinOp::Lt => (l < r) as i64,
                    BinOp::Gt => (l > r) as i64,
                    BinOp::Lte => (l <= r) as i64,
                    BinOp::Gte => (l >= r) as i64,
                    BinOp::And => ((l != 0) && (r != 0)) as i64,
                    BinOp::Or => ((l != 0) || (r != 0)) as i64,
                };
                Ok(Value::Integer(result))
            }
            (Value::Varchar(l), Value::Varchar(r)) => {
                let result = match op {
                    BinOp::Eq => (l == r) as i64,
                    BinOp::Neq => (l != r) as i64,
                    BinOp::Lt => (l < r) as i64,
                    BinOp::Gt => (l > r) as i64,
                    BinOp::Lte => (l <= r) as i64,
                    BinOp::Gte => (l >= r) as i64,
                    _ => return Err(EvaluationError::TypeMismatch {
                        op,
                        left_type: "VARCHAR".to_string(),
                        right_type: "VARCHAR".to_string(),
                    }),
                };
                Ok(Value::Integer(result))
            }
            (left_val, right_val) => {
                let left_type = match left_val {
                    Value::Integer(_) => "INTEGER",
                    Value::Varchar(_) => "VARCHAR",
                };
                let right_type = match right_val {
                    Value::Integer(_) => "INTEGER",
                    Value::Varchar(_) => "VARCHAR",
                };
                Err(EvaluationError::TypeMismatch {
                    op,
                    left_type: left_type.to_string(),
                    right_type: right_type.to_string(),
                })
            }
        }
    }

    fn apply_unary_op(op: UnaryOp, value: Value) -> Result<Value, EvaluationError> {
        match (op, value) {
            (UnaryOp::Neg, Value::Integer(n)) => {
                Ok(Value::Integer(-n))
            }
            (UnaryOp::Not, Value::Integer(n)) => {
                Ok(Value::Integer((n == 0) as i64))
            }
            (op, val) => {
                let val_type = match val {
                    Value::Integer(_) => "INTEGER",
                    Value::Varchar(_) => "VARCHAR",
                };
                Err(EvaluationError::UnaryTypeMismatch {
                    op,
                    value_type: val_type.to_string(),
                })
            }
        }
    }
}

#[derive(Debug)]
pub enum EvaluationError {
    ColumnNotFound(String),
    NoRowContext(String),
    TypeMismatch {
        op: BinOp,
        left_type: String,
        right_type: String,
    },
    UnaryTypeMismatch {
        op: UnaryOp,
        value_type: String,
    },
    DivisionByZero,
}


impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvaluationError::ColumnNotFound(col) => {
                write!(f, "Column '{}' not found in current row", col)
            }
            EvaluationError::NoRowContext(col) => {
                write!(f, "Cannot evaluate identifier '{}' without row context", col)
            }
            EvaluationError::TypeMismatch { op, left_type, right_type } => {
                write!(f, "Type mismatch: cannot apply operator {:?} to types {} and {}",
                    op, left_type, right_type)
            }
            EvaluationError::UnaryTypeMismatch { op, value_type } => {
                write!(f, "Type mismatch: cannot apply unary operator {:?} to type {}",
                    op, value_type)
            }
            EvaluationError::DivisionByZero => {
                write!(f, "Division by zero")
            }
        }
    }
}

mod tests {
    use super::*;
    use crate::parser::ast::{BinOp, UnaryOp};

    #[test]
    fn test_evaluate_simple_expressions() {
        let empty_row: [Value; 0] = [];

        let expr = Expr::Number(42);
        assert_eq!(
            Evaluator::evaluate(&expr, &empty_row).unwrap(),
            Value::Integer(42)
        );

        let expr = Expr::StringLit("hello".to_string());
        assert_eq!(
            Evaluator::evaluate(&expr, &empty_row).unwrap(),
            Value::Varchar("hello".to_string())
        );
    }

    #[test]
    fn test_evaluate_binary_ops() {
        let empty_row: [Value; 0] = [];
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(1)),
            op: BinOp::Add,
            right: Box::new(Expr::Number(2)),
        };
        assert_eq!(Evaluator::evaluate(&expr, &empty_row).unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_evaluate_unary_ops() {
        let empty_row: [Value; 0] = [];
        let expr = Expr::UnaryOp {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::Number(5)),
        };
        assert_eq!(Evaluator::evaluate(&expr, &empty_row).unwrap(), Value::Integer(-5));
    }

    #[test]
    fn test_evaluate_complex_expression() {
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::UnaryOp {
                op: UnaryOp::Not,
                expr: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::ColumnIndex(0)),
                    op: BinOp::Eq,
                    right: Box::new(Expr::Number(1)),
                }),
            }),
            op: BinOp::Or,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::ColumnIndex(1)),
                op: BinOp::Gt,
                right: Box::new(Expr::Number(2)),
            }),
        };
        let row_values = vec![Value::Integer(0), Value::Integer(3)];
        assert_eq!(Evaluator::evaluate(&expr, &row_values).unwrap(), Value::Integer(1)); // true
    }

    #[test]
    fn test_evaluate_complex_math_expression() {
        let empty_row: [Value; 0] = [];
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Number(1)),
            op: BinOp::Add,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Number(2)),
                op: BinOp::Mul,
                right: Box::new(Expr::Number(3)),
            }),
        };
        assert_eq!(Evaluator::evaluate(&expr, &empty_row).unwrap(), Value::Integer(7)); // tests precedence
    }

    #[test]
    fn test_evaluate_parenthesized_expression() {
        let empty_row: [Value; 0] = [];
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Number(1)),
                op: BinOp::Add,
                right: Box::new(Expr::Number(2)),
            }),
            op: BinOp::Mul,
            right: Box::new(Expr::Number(3)),
        };
        assert_eq!(Evaluator::evaluate(&expr, &empty_row).unwrap(), Value::Integer(9)); // tests parentheses
    }
}

impl std::error::Error for EvaluationError {}
