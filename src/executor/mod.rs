use std::collections::HashMap;
use std::fmt::Display;
use crate::executor::context::ExecutionContext;
use crate::parser::ast::Statement;
use crate::storage::data_structures::{Row};

mod context;
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

    pub fn load_tables(&mut self) {
        !todo!("Load tables metadata from disk into memory")
    }

    pub fn execute(&mut self, statement: Statement) -> Result<ExecutionResult, ExecutionError> {
        match statement {
            Statement::Select { from, columns, where_clause, order_by } => {
                !todo!()
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


impl Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::TableNotFound(name) => write!(f, "Table '{}' not found", name),
            ExecutionError::ColumnNotFound(name) => write!(f, "Column '{}' not found", name),
            ExecutionError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            ExecutionError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            ExecutionError::Other(msg) => write!(f, "{}", msg),
        }
    }
}