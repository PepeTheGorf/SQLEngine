use std::collections::HashMap;
use crate::storage::data_structures::Table;

pub struct ExecutionContext {
    pub tables: HashMap<String, Table>,
}