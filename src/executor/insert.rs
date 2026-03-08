use crate::executor::{ExecutionError, ExecutionResult, Executor};
use crate::parser::ast::Expr;
use crate::storage::data_structures::{Row, Value};

impl Executor {
    pub fn execute_insert(
        &mut self,
        table_name: String,
        values: Vec<Vec<Expr>>,
    ) -> Result<ExecutionResult, ExecutionError> {

        let table = self
            .context()
            .tables()
            .get_mut(&table_name)
            .ok_or(ExecutionError::TableNotFound(table_name.clone()))?;

        let mut affected_rows = 0;

        for row_values in values {
            let mut row = Row {
                values: Vec::new(),
            };

        }
    }
}