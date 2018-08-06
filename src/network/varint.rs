use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read};

use network::NetworkError;

#[derive(Debug)]
pub struct VarInt(u64);

impl VarInt {
    pub fn new(data: u64) -> VarInt {
        VarInt(data)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn length(&self) -> usize {
        match self.0 {
            0 ... 0xFC => 1,
            0xFD ... 0xFFFF => 3,
            0x1_0000 ... 0xFFFF_FFFF => 5,
            _ => 9,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut result = vec![];

        match self.0 {
            0 ... 0xFC => {
                result.write_u8(self.0 as u8).unwrap();
            },
            0xFD ... 0xFFFF => {
                result.write_u8(0xFD).unwrap();
                result.write_u16::<LittleEndian>(self.0 as u16).unwrap();
            },
            0x1_0000 ... 0xFFFF_FFFF => {
                result.write_u8(0xFE).unwrap();
                result.write_u32::<LittleEndian>(self.0 as u32).unwrap();
            },
            _ => {
                result.write_u8(0xFF).unwrap();
                result.write_u64::<LittleEndian>(self.0 as u64).unwrap();
            },
        }

        result
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<VarInt, NetworkError> {
        let length_type = reader.read_u8()?;
        let value;

        match length_type {
            0xFF => {
                value = reader.read_u64::<LittleEndian>()?;
            },
            0xFE => {
                value = reader.read_u32::<LittleEndian>()? as u64;
            },
            0xFD => {
                value = reader.read_u16::<LittleEndian>()? as u64;
            },
            _ => {
                value = length_type as u64;
            },
        }

        Ok(VarInt(value))
    }
}

impl Into<u64> for VarInt {
    fn into(self) -> u64 {
        self.0
    }
}