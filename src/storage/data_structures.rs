use std::collections::HashMap;
use crate::parser::ast::{
    ColumnDef
};

pub struct Table {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub column_index: HashMap<String, usize>,
    //Below is temporary storage for testing DDL, DML and QL before file storage is implemented. This will be removed once file storage is implemented.
    pub rows: Vec<Row>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub values: Vec<Value>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Varchar(String),
}
