use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

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

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        match self.0 {
            0 ... 0xFC => {
                writer.write_u8(self.0 as u8)?;
            },
            0xFD ... 0xFFFF => {
                writer.write_u8(0xFD)?;
                writer.write_u16::<LittleEndian>(self.0 as u16)?;
            },
            0x1_0000 ... 0xFFFF_FFFF => {
                writer.write_u8(0xFE)?;
                writer.write_u32::<LittleEndian>(self.0 as u32)?;
            },
            _ => {
                writer.write_u8(0xFF)?;
                writer.write_u64::<LittleEndian>(self.0 as u64)?;
            },
        }

        Ok(())
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