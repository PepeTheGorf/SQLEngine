use std::path::Path;

use crate::executor::{ExecutionError, ExecutionResult, Executor};
use crate::parser::ast::Expr;
use crate::parser::evaluator::Evaluator;
use crate::storage::codec::encode_row;
use crate::storage::data_structures::{Row, Value};
use crate::storage::files::serial::SerialFile;

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

        let table_path = Path::new("catalog").join(format!("{}.json", table_name));
        if !table_path.exists() {
            return Err(ExecutionError::TableNotFound(table_name));
        }

        let storage_path = Path::new("storage");
        let mut table_file = SerialFile::open(storage_path.join(format!("{}.bin", table_name)))
            .map_err(|e| ExecutionError::Other(format!("Failed to open table file: {}", e)))?;

        for row_values in &values {
            if row_values.len() != table.columns.len() {
                return Err(ExecutionError::Other(format!(
                    "Column count mismatch: expected {}, got {}",
                    table.columns.len(),
                    row_values.len()
                )));
            }
            let mut row = Row { values: Vec::new() };
            for (_, expression) in table.columns.iter().zip(row_values.iter()) {
                let empty_row: [Value; 0] = [];
                let value = Evaluator::evaluate(expression, &empty_row)
                    .map_err(|e| ExecutionError::Other(format!("Failed to evaluate expression: {:?}", e)))?;

                row.values.push(value);
            }
            let encoded_row = encode_row(&table, &row);

            table_file
                .insert_record(
                    &encoded_row.map_err(|e| ExecutionError::Other(format!("Failed to encode row: {}", e)))?
                )
                .map_err(|e| ExecutionError::Other(format!("Insert failed: {}", e)))?;
        }
        table_file.close()
            .map_err(|e| ExecutionError::Other(format!("Failed to close table file: {}", e)))?;
        Ok(ExecutionResult::AffectedRows(values.len()))
    }
}