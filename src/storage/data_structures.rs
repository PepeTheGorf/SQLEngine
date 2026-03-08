use std::collections::HashMap;
use crate::parser::ast::{
    ColumnDef
};

pub struct Table {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub column_index: HashMap<String, usize>
}

pub struct Row {
    pub values: Vec<Value>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Varchar(String),
}
