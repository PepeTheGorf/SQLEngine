use std::collections::HashMap;
use std::fmt;
use std::mem::size_of;

use crate::parser::ast::ColumnDef;
use crate::storage::codec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub column_index: HashMap<String, usize>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub values: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Varchar(String),
}

impl fmt::Display for &Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Varchar(s) => {
                let escaped = s.replace('\'', "''");
                write!(f, "'{}'", escaped)
            }
        }
    }
}

impl fmt::Display for &Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, v) in self.values.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", &v)?;
        }
        write!(f, ")")
    }
}

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
pub struct Page {
    pub data: [u8; PAGE_SIZE],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct PageHeader {
    pub slot_number: u16,
    pub start_offset: u16,
    pub end_offset: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Slot {
    pub offset: u16,
    pub len: u16,
}

const PAGE_HEADER_SIZE: usize = size_of::<PageHeader>();
const SLOT_SIZE: usize = size_of::<Slot>();

impl Slot {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < SLOT_SIZE {
            return None;
        }
        codec::decode_from_slice(&bytes[0..SLOT_SIZE]).ok()
    }

    fn to_bytes(&self) -> [u8; SLOT_SIZE] {
        let mut out = [0u8; SLOT_SIZE];
        let _ = codec::encode_into_slice(self, &mut out);
        out
    }
}

impl Page {
    pub fn new() -> Self {
        let mut page = Self { data: [0; PAGE_SIZE] };

        let header = PageHeader {
            slot_number: 0,
            start_offset: PAGE_HEADER_SIZE as u16,
            end_offset: PAGE_SIZE as u16,
        };
        let _ = codec::encode_into_slice(&header, &mut page.data[0..PAGE_HEADER_SIZE]);

        page
    }

    pub fn header(&self) -> PageHeader {
        codec::decode_from_slice(&self.data[0..PAGE_HEADER_SIZE])
            .expect("PageHeader decoding must succeed")
    }

    fn set_header(&mut self, header: PageHeader) {
        codec::encode_into_slice(&header, &mut self.data[0..PAGE_HEADER_SIZE])
            .expect("PageHeader encoding must succeed")
    }

    pub fn free_space(&self) -> usize {
        let h = self.header();
        h.end_offset.saturating_sub(h.start_offset) as usize
    }

    pub fn insert_record(&mut self, record: &[u8]) -> Result<u16, String> {
        let record_len: u16 = record
            .len()
            .try_into()
            .map_err(|_| "Record too large".to_string())?;

        let mut h = self.header();

        let required = record.len() + SLOT_SIZE;
        if required > self.free_space() {
            return Err("Not enough space in page".to_string());
        }

        let new_end = h.end_offset - record_len;
        self.data[(h.end_offset - record_len) as usize..h.end_offset as usize].copy_from_slice(record);

        let slot_id = h.slot_number;
        self.write_slot(slot_id, Slot { offset: new_end, len: record_len })?;

        h.start_offset += SLOT_SIZE as u16;
        h.end_offset = new_end;
        h.slot_number += 1;
        self.set_header(h);

        Ok(slot_id)
    }

    pub fn record_iterate(
        &self,
    ) -> Result<impl Iterator<Item=&[u8]>, String> {
        let slot_count = self.header().slot_number;
        Ok((0..slot_count).filter_map(|slot_id| {
            let slot = self.read_slot(slot_id)?;
            Some(&self.data[slot.offset as usize..(slot.offset + slot.len) as usize])
        }))
    }

    fn slot_position(slot_number: u16) -> usize {
        PAGE_HEADER_SIZE + (slot_number as usize * SLOT_SIZE)
    }

    pub fn read_slot(&self, slot_number: u16) -> Option<Slot> {
        let pos = Self::slot_position(slot_number);
        if pos + SLOT_SIZE > PAGE_SIZE {
            return None;
        }
        if slot_number >= self.header().slot_number {
            return None;
        }
        Slot::from_bytes(&self.data[pos..pos + SLOT_SIZE])
    }

    fn write_slot(&mut self, slot_number: u16, slot: Slot) -> Result<(), String> {
        let pos = Self::slot_position(slot_number);

        self.data[pos..pos + SLOT_SIZE].copy_from_slice(&slot.to_bytes());
        Ok(())
    }
}
