use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::storage::codec;
use crate::storage::data_structures::{Page, PAGE_SIZE};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct FileHeader {
    pub page_count: u32,
    pub free_page_list_head: u32
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
struct HeaderPage {
    header: FileHeader,
}

#[derive(Debug)]
pub struct SerialFile {
    pub header: FileHeader,

    pub file: Option<File>
}

impl SerialFile {
    fn header_page_offset() -> u64 {
        0
    }

    fn data_page_offset(page_id: u64) -> u64 {
        PAGE_SIZE as u64 + page_id * PAGE_SIZE as u64
    }

    fn write_header_page(file: &mut File, header: &FileHeader) -> Result<(), String> {
        let encoded = codec::encode_to_vec(header)?;
        if encoded.len() > PAGE_SIZE {
            return Err(format!("Header too large: {} bytes (max {})", encoded.len(), PAGE_SIZE));
        }

        let mut buf = [0u8; PAGE_SIZE];
        buf[..encoded.len()].copy_from_slice(&encoded);

        file.seek(SeekFrom::Start(Self::header_page_offset()))
            .map_err(|e| format!("Failed to seek to header page: {e}"))?;
        file.write_all(&buf)
            .map_err(|e| format!("Failed to write header page: {e}"))?;
        Ok(())
    }

    fn read_header_page(file: &mut File) -> Result<FileHeader, String> {
        file.seek(SeekFrom::Start(Self::header_page_offset()))
            .map_err(|e| format!("Failed to seek to header page: {e}"))?;

        let mut buf = [0u8; PAGE_SIZE];
        file.read_exact(&mut buf)
            .map_err(|e| format!("Failed to read header page: {e}"))?;

        codec::decode_from_slice::<FileHeader>(&buf)
            .map_err(|e| format!("Failed to decode file header: {e}"))
    }

    pub fn create(path: impl Into<PathBuf>) -> Result<(), String> {
        let path = path.into();

        if path.exists() {
            return Err(format!("File already exists at path: {}", path.display()));
        }

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        let header = FileHeader {
            page_count: 0,
            free_page_list_head: 0,
        };

        Self::write_header_page(&mut file, &header)?;
        Ok(())
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.clone())
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let file_size = file.metadata().map_err(|e| e.to_string())?.len();

        let header = if file_size < PAGE_SIZE as u64 {
            let header = FileHeader { page_count: 0, free_page_list_head: 0 };
            Self::write_header_page(&mut file, &header)?;
            header
        } else {
            Self::read_header_page(&mut file)?
        };

        Ok(Self {
            header,
            file: Some(file),
        })
    }

    pub fn close(&mut self) -> Result<(), String> {
        if let Some(file) = &mut self.file {
            Self::write_header_page(file, &self.header)?;
            self.file = None;
        }
        Ok(())
    }

    pub fn insert_record(&mut self, record: &[u8]) -> Result<(u64, u16), String> {
        let page_id = self.header.free_page_list_head as u64;
        let mut page = self.read_page(page_id)?;

        match page.insert_record(record) {
            Ok(slot_id) => {
                self.write_page(page_id, &codec::encode_to_vec(&page)?)?;
                Ok((page_id, slot_id))
            }
            Err(_) => {
                let mut new_page = self.append_page()?;
                let new_page_id = self.header.page_count as u64 - 1;
                let slot_id = new_page.insert_record(record)?;

                self.write_page(new_page_id, &codec::encode_to_vec(&new_page)?)?;
                self.header.free_page_list_head = new_page_id as u32;
                Ok((new_page_id, slot_id))
            }
        }
    }

    pub fn page_iterate(
        &mut self,
    ) -> Result<impl Iterator<Item = Result<Page, String>>, String> {
        let page_count = self.header.page_count as u64;

        Ok((0..page_count).map(move |page_id| {
            self.read_page(page_id)
        }))
    }

    fn write_page(&mut self, page_id: u64, data: &[u8]) -> Result<(), String> {
        let Some(file) = &mut self.file else {
            return Err("File is not open".to_string());
        };

        if data.len() != PAGE_SIZE {
            return Err(format!("Page write requires exactly {} bytes, got {}", PAGE_SIZE, data.len()));
        }

        file.seek(SeekFrom::Start(Self::data_page_offset(page_id)))
            .map_err(|e| e.to_string())?;
        file.write_all(data)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn read_page(&mut self, page_id: u64) -> Result<Page, String> {
        let Some(file) = &mut self.file else {
            return Err("File is not open".to_string());
        };

        let offset = Self::data_page_offset(page_id);

        file.seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;

        let mut page_buffer = [0u8; PAGE_SIZE];
        file.read_exact(&mut page_buffer)
            .map_err(|_| "Failed to fill page buffer".to_string())?;

        Ok(Page { data: page_buffer })
    }

    pub fn append_page(&mut self) -> Result<Page, String> {
        let Some(file) = &mut self.file else {
            return Err("File is not open".to_string());
        };

        let page_id = self.header.page_count as u64;
        file.seek(SeekFrom::Start(Self::data_page_offset(page_id)))
            .map_err(|e| e.to_string())?;

        let new_page = Page::new();
        let data = codec::encode_to_vec(&new_page)?;

        file.write_all(&data)
            .map_err(|e| e.to_string())?;
        self.header.page_count += 1;

        Ok(new_page)
    }
}