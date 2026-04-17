use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::time::Instant;
use crate::executor::context::ExecutionContext;
use crate::executor::iterators::Operator;
use crate::executor::iterators::projection::Projection;
use crate::executor::iterators::selection::Selection;
use crate::executor::iterators::table_scan::TableScan;
use crate::parser::ast::Statement;
use crate::storage::codec::decode_row;
use crate::storage::data_structures::{Row, Table};
use crate::storage::files::serial::SerialFile;

pub(crate) mod context;
mod plan;
mod iterators;
mod insert;
mod create;

pub struct Executor {
    pub(crate) context: ExecutionContext,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            context: ExecutionContext {
                tables: HashMap::new(),
            },
        }
    }

    pub fn set_context(&mut self, context: ExecutionContext) {
        self.context = context;
    }

    pub fn load_tables(&mut self) {
        !todo!("Load tables metadata from disk into memory")
    }

    pub fn execute(&mut self, statement: Statement) -> Result<ExecutionResult, ExecutionError> {
        match statement {
            Statement::Select { from, columns, where_clause, order_by } => {
                let mut start = Instant::now();

                let table = self
                    .context
                    .tables
                    .get(&from)
                    .ok_or_else(|| ExecutionError::TableNotFound(from.clone()))?;

                let columns = crate::parser::binder::bind_select_columns(columns, &table.column_index)?;
                let where_clause = crate::parser::binder::bind_where_clause(where_clause, &table.column_index)?;

                let storage_path = Path::new("storage");
                let table_file = SerialFile::open(storage_path.join(format!("{}.bin", from)))
                    .map_err(|e| ExecutionError::Other(format!("Failed to open table file: {}", e)))?;


                let scan = Box::new(TableScan::new(table_file,table.clone()));
                let selection: Box<dyn Operator> = Box::new(Selection::new(scan, where_clause));
                let mut projection: Box<dyn Operator> = Box::new(Projection::new(selection, columns));

                let mut row_count = 0;
                projection.open();
                while let Some(row) = projection.next() {
                    println!("{:?}", row);
                    row_count += 1;
                }
                projection.close();


                let duration = start.elapsed();

                let micros = duration.as_micros();
                let millis = duration.as_secs_f64() * 1000.0;

                println!("Time taken: {} µs ({:.3} ms)", micros, millis);

                Ok(ExecutionResult::AffectedRows(row_count))
            }
            Statement::CreateTable { name, columns } => {
                self.execute_create_table(name, columns)
            }
            Statement::Insert { table, values } => {
                self.execute_insert(table, values)
            }
        }
    }
}

#[derive(Debug)]
pub enum ExecutionResult {
    Rows(Vec<Row>),
    AffectedRows(usize),
    Ok,
}

pub enum ExecutionError {
    TableNotFound(String),
    ColumnNotFound(String),
    TypeMismatch(String),
    SyntaxError(String),
    Other(String),
}


impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::TableNotFound(name) => write!(f, "Table '{}' not found", name),
            ExecutionError::ColumnNotFound(name) => write!(f, "Column '{}' not found", name),
            ExecutionError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            ExecutionError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            ExecutionError::Other(msg) => write!(f, "{}", msg),
        }
    }
}