use crate::executor::{ExecutionError, ExecutionResult, Executor};
use crate::parser::ast::{Expr};
use crate::parser::evaluator::Evaluator;
use crate::storage::data_structures::Row;

impl Executor {
    pub fn execute_insert(
        &mut self,
        table_name: String,
        values: Vec<Vec<Expr>>,
    ) -> Result<ExecutionResult, ExecutionError> {

        let table = self.context
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| ExecutionError::TableNotFound(table_name.clone()))?;

        for row_values in &values {
            if row_values.len() != table.columns.len() {
                return Err(ExecutionError::Other(format!(
                    "Column count mismatch: expected {}, got {}",
                    table.columns.len(),
                    row_values.len()
                )));
            }
            let mut row = Row { values: Vec::new() };
            for (column, expression) in table.columns.iter().zip(row_values.iter()) {
                let value = Evaluator::evaluate(expression.clone(), &None)
                    .map_err(|e| ExecutionError::Other(format!("Failed to evaluate expression: {:?}", e)))?;

                column.data_type.validate(&value)?;

                row.values.push(value);
            }
            table.rows.push(row);
        }
        Ok(ExecutionResult::AffectedRows(values.len()))
    }
}