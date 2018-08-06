use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read};

use network::NetworkError;

// TODO: this whole thing doesn't need to exist if we have a NetworkDeserializer trait or something like this and implement it for string.
#[derive(Debug)]
pub struct VarString {
    data: String,
}

impl VarString {
    pub fn new() -> VarString {
        VarString{
            data: String::new(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut result = vec![];

        match self.data.len() {
            0 ... 0xFC => {
                result.write_u8(self.data.len() as u8).unwrap();
                result.extend_from_slice(self.data.as_bytes());
            },
            0xFD ... 0xFFFF => {
                result.write_u8(0xFD).unwrap();
                result.write_u16::<LittleEndian>(self.data.len() as u16).unwrap();
            },
            0x1_0000 ... 0xFFFF_FFFF => {
                result.write_u8(0xFE).unwrap();
                result.write_u32::<LittleEndian>(self.data.len() as u32).unwrap();
            },
            _ => {
                result.write_u8(0xFF).unwrap();
                result.write_u64::<LittleEndian>(self.data.len() as u64).unwrap();
            }
        }

        result
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<VarString, NetworkError> {
        let length_type = reader.read_u8()?;
        let length;

        match length_type {
            0xFF => {
                length = reader.read_u64::<LittleEndian>()?;
            },
            0xFE => {
                length = reader.read_u32::<LittleEndian>()? as u64;
            },
            0xFD => {
                length = reader.read_u16::<LittleEndian>()? as u64;
            },
            _ => {
                length = length_type as u64;
            }
        }

        // TODO: this can be a problem if length is actually too big.
        let mut data = vec![0; length as usize];
        reader.read_exact(&mut data)?;
        let data = String::from_utf8(data)?;

        Ok(VarString {
            data,
        })
    }
}