use crate::parser::ast::DataType;
use crate::storage::data_structures::{Row, Table, Value};

pub fn config() -> impl bincode::config::Config {
    bincode::config::standard().with_fixed_int_encoding()
}

pub fn encode_to_vec<T>(value: &T) -> Result<Vec<u8>, String>
where
    T: bincode::Encode,
{
    bincode::encode_to_vec(value, config()).map_err(|e| format!("bincode encode failed: {e}"))
}

pub fn encode_into_slice<T>(value: &T, destination: &mut [u8]) -> Result<(), String>
where
    T: bincode::Encode,
{
    let bytes = encode_to_vec(value)?;
    if bytes.len() != destination.len() {
        return Err(format!(
            "Encoded size mismatch: expected {} bytes, got {}",
            destination.len(),
            bytes.len()
        ));
    }
    destination.copy_from_slice(&bytes);
    Ok(())
}

pub fn decode_from_slice<T>(src: &[u8]) -> Result<T, String>
where
    T: bincode::Decode<()>,
{
    bincode::decode_from_slice(src, config())
        .map(|(v, _len)| v)
        .map_err(|e| format!("bincode decode failed: {e}"))
}

pub fn encode_row(table: &Table, row: &Row) -> Result<Vec<u8>, String> {
    if row.values.len() != table.columns.len() {
        return Err(format!(
            "Column count mismatch: expected {}, got {}",
            table.columns.len(),
            row.values.len()
        ))
    }
    let mut bytes = Vec::new();
    for (value, column) in row.values.iter().zip(table.columns.iter()) {
        match (value, &column.data_type) {
            (Value::Integer(i), DataType::Integer) => {
                bytes.extend(i.to_le_bytes());
            }
            (Value::Varchar(s), DataType::Varchar(len)) => {
                if *len < s.len() as u32 {
                    return Err(format!(
                        "String length exceeds defined limit for column '{}': max {}, but got {}",
                        column.name, len, s.len()
                    ));
                }
                let str_bytes = s.as_bytes();
                let len = str_bytes.len() as u16;
                bytes.extend(len.to_le_bytes());
                bytes.extend(str_bytes);
            }
            _ => {
                return Err(format!(
                    "Type mismatch for column '{}': expected {:?}, but got {:?}",
                    column.name, column.data_type, value
                ));
            }
        }
    }
    Ok(bytes)
}

pub fn decode_row(table: &Table, bytes: &[u8]) -> Result<Row, String> {
    let mut values = Vec::new();
    let mut offset = 0;

    for column in &table.columns {
        match &column.data_type {
            DataType::Integer => {
                values.push(Value::Integer(i64::from_le_bytes(
                    bytes[offset..offset + 8]
                        .try_into()
                        .map_err(|_| format!("Failed to decode integer for column '{}'", column.name))?
                )));
                offset += 8;
            }
            DataType::Varchar(_) => {
                let str_len = u16::from_le_bytes(
                    bytes[offset..offset + 2]
                        .try_into()
                        .map_err(|_| format!("Failed to decode string length for column '{}'", column.name))?
                ) as usize;
                offset += 2;
                values.push(Value::Varchar(str::from_utf8(
                    &bytes[offset..offset + str_len]
                ).map_err(|_| format!("Failed to decode string for column '{}'", column.name))?.to_string()));
                offset += str_len;
            }
        }
    }

    Ok(Row { values })
}
