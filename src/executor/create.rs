use crate::executor::{ExecutionError, ExecutionResult, Executor};
use crate::parser::ast::{ColumnDef};
use crate::storage::data_structures::Table;

impl Executor {
    pub fn execute_create_table(
        &mut self,
        name: String,
        columns: Vec<ColumnDef>,
    ) -> Result<ExecutionResult, ExecutionError> {
        if self.context.tables.contains_key(&name) {
            return Err(ExecutionError::Other(format!("Table '{}' already exists", name)));
        }
        let column_index = columns.iter()
            .enumerate()
            .map(|(i, col)| (col.name.clone(), i))
            .collect::<std::collections::HashMap<_, _>>();

        let table = Table {
            name: name.clone(),
            columns,
            column_index,
            rows: Vec::new(),
        };
        //todo: create table on disk
        self.context.tables.insert(name, table);
        Ok(ExecutionResult::Ok)
    }
}