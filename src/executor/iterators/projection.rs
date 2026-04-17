use crate::executor::iterators::Operator;
use crate::parser::ast::{Expr, SelectColumns};
use crate::parser::evaluator::{EvaluationError, Evaluator};
use crate::storage::data_structures::{Row, Value};

pub struct Projection {
    child: Box<dyn Operator>,
    columns: SelectColumns,
}

impl Projection {
    pub fn new(child: Box<dyn Operator>, columns: SelectColumns) -> Self {
        Self { child, columns }
    }
}

impl Operator for Projection {
    fn open(&mut self) {
        self.child.open();
    }

    fn next(&mut self) -> Option<Row> {
        let row = self.child.next()?;

        match &self.columns {
            SelectColumns::All => Some(row),

            SelectColumns::Expressions(items) => {
                let mut projected_row = Row {
                    values: Vec::with_capacity(items.len()),
                };

                for item in items {
                    match Evaluator::evaluate(&item.expr, &row.values) {
                        Ok(value) => projected_row.values.push(value),
                        Err(e) => {
                            eprintln!("Error evaluating projection expression: {:?}", e);
                            return None;
                        }
                    }
                }

                Some(projected_row)
            }
        }
    }

    fn close(&mut self) {
        self.child.close();
    }
}


