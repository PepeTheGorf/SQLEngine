use std::collections::HashMap;
use crate::executor::context::ExecutionContext;
use crate::parser::ast::Statement;
use crate::storage::data_structures::{Row, Table};

mod context;
mod plan;
mod iterators;
mod insert;

pub struct Executor {
    context: ExecutionContext,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            context: ExecutionContext {
                tables: HashMap::new(),
            },
        }
    }

    pub fn execute(&mut self, statement: Statement) -> Result<ExecutionResult, ExecutionError> {
        match statement {
            Statement::Select { from, columns, where_clause, order_by } => {
                !todo!()
            }
            Statement::CreateTable { name, columns } => {
                if self.context.tables.contains_key(&name) {
                    return Err(ExecutionError::Other(format!("Table '{}' already exists", name)));
                }
                let column_index = columns.iter()
                    .enumerate()
                    .map(|(i, col)| (col.name.clone(), i))
                    .collect::<HashMap<_, _>>();
                let table = Table {
                    name: name.clone(),
                    columns,
                    column_index,
                };

                self.context.tables.insert(name, table);
                Ok(ExecutionResult::Ok)
            }
            Statement::Insert { table, values } => {
                self.execute_insert(table, values)
            }
        }
    }
}

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