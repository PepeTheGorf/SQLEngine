pub mod table_scan;
pub mod projection;
pub mod selection;

use crate::storage::data_structures::Row;

pub trait Operator {
    fn open(&mut self);
    fn next(&mut self) -> Option<Row>;
    fn close(&mut self);
}


