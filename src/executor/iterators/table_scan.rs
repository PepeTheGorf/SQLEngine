use std::rc::Rc;
use crate::executor::iterators::Operator;
use crate::storage::codec::decode_row;
use crate::storage::data_structures::{Page, Row, Slot, Table};
use crate::storage::files::serial::SerialFile;

pub struct TableScan {
    table: SerialFile,
    table_definition: Table,
    current_page_id: u32,
    current_slot_id: u16,
    current_page: Option<Page>,
}

impl TableScan {
    pub fn new(table: SerialFile, table_definition: Table) -> Self {
        Self {
            table,
            table_definition,
            current_page_id: 0,
            current_slot_id: 0,
            current_page: None,
        }
    }
}

impl Operator for TableScan {
    fn open(&mut self) {
        self.current_page = self.table.read_page(self.current_page_id as u64).ok();
        self.current_page_id += 1;
    }

    fn next(&mut self) -> Option<Row> {
        loop {
            let page = match &self.current_page {
                Some(page) => page,
                None => return None,
            };

            if let Some(slot) = page.read_slot(self.current_slot_id) {
                self.current_slot_id += 1;
                return Some(
                    //TODO: remove unwrap and handle error properly and make return value be row reference (zero copy!)
                    decode_row(
                        &self.table_definition,
                        &page.data[slot.offset as usize..(slot.offset + slot.len) as usize],
                    ).unwrap()
                );
            }
            self.current_page = self.table.read_page(self.current_page_id as u64).ok();
            self.current_page_id += 1;
            self.current_slot_id = 0;

        }
    }

    fn close(&mut self) {
        self.current_page = None;
        self.table.close().unwrap();
    }
}