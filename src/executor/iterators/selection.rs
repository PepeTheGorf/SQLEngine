use crate::executor::iterators::Operator;
use crate::parser::ast::Expr;
use crate::parser::evaluator::Evaluator;
use crate::storage::data_structures::{Row, Value};

pub struct Selection {
    child: Box<dyn Operator>,
    predicate: Option<Expr>
}

impl Selection {
    pub fn new(child: Box<dyn Operator>, predicate: Option<Expr>) -> Self {
        Self { child, predicate }
    }


}

impl Operator for Selection {
    fn open(&mut self) {
        self.child.open();
    }

    fn next(&mut self) -> Option<Row> {
        loop {
            let row = self.child.next()?;

            if let Some(predicate) = &self.predicate {
                match Evaluator::evaluate(predicate, &row.values) {
                    Ok(Value::Integer(1)) => return Some(row),
                    Ok(Value::Integer(0)) => continue,
                    Ok(_) => continue,
                    Err(e) => {
                        eprintln!("Error evaluating predicate: {:?}", e);
                        continue;
                    }
                }
            } else {
                return Some(row);
            }
        }
    }

    fn close(&mut self) {
        self.child.close();
    }
}


