use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::executor::{ExecutionError, ExecutionResult, Executor};
use crate::parser::ast::{ColumnDef};
use crate::storage::data_structures::Table;
use crate::storage::files::serial::SerialFile;

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
            .collect::<HashMap<String, usize>>();

        let table = Table {
            name: name.clone(),
            columns,
            column_index,
        };

        let path = Path::new("catalog").join(format!("{}.json", name));
        serde_json::to_writer_pretty(
            fs::File::create(&path)
                .map_err(|e| ExecutionError::Other(format!("Failed to create table file: {}", e)))?,
            &table,
        ).map_err(|e| ExecutionError::Other(format!("Failed to write table to file: {}", e)))?;

        self.context.tables.insert(name.clone(), table);

        let file_name = String::from(name + ".bin");
        let storage_path = Path::new("storage");
        let serial_file = SerialFile::create(storage_path.join(file_name.clone()));

        let mut file = SerialFile::open(storage_path.join(file_name))
            .map_err(|e| ExecutionError::Other(format!("Failed to create table file: {}", e)))?;
        file.append_page()
            .map_err(|e| ExecutionError::Other(format!("Failed to initialize table file: {}", e)))?;
        file.close();


        Ok(ExecutionResult::Ok)
    }
}