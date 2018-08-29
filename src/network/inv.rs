use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use network::NetworkError;
use network::varint::VarInt;

#[derive(Clone, Debug)]
pub enum InventoryType {
    Error,
    Msg_Tx,
    Msg_Block,
    Msg_Filtered_Block,
    Msg_Cmpct_Block,
}

impl InventoryType {
    pub fn value(&self) -> u32 {
        match *self {
            InventoryType::Error => 0,
            InventoryType::Msg_Tx => 1,
            InventoryType::Msg_Block => 2,
            InventoryType::Msg_Filtered_Block => 3,
            InventoryType::Msg_Cmpct_Block => 4,
        }
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<InventoryType, NetworkError> {
        let object_type = reader.read_u32::<LittleEndian>()?;

        match object_type {
            0 => Ok(InventoryType::Error),
            1 => Ok(InventoryType::Msg_Tx),
            2 => Ok(InventoryType::Msg_Block),
            3 => Ok(InventoryType::Msg_Filtered_Block),
            4 => Ok(InventoryType::Msg_Cmpct_Block),
            _ => Err(NetworkError::InvalidValue),
        }
    }
}

#[derive(Clone)]
pub struct InventoryVector {
    object_type: InventoryType,
    hash: [u8; 32],
}

impl InventoryVector {
    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        writer.write_u32::<LittleEndian>(self.object_type.value())?;
        writer.write_all(&self.hash)?;

        Ok(())
    }

    pub fn length() -> usize {
        36
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<InventoryVector, NetworkError> {
        let object_type = InventoryType::deserialize(reader)?;
        let mut hash = [0; 32];
        reader.read_exact(&mut hash)?;

        Ok(InventoryVector {
            object_type,
            hash,
        })
    }
}

fn byte_slice_as_hex(slice: &[u8]) -> String {
    let mut result = String::new();
    for byte in slice {
        result.push_str(&format!("{:02x}", byte));
    }
    result
}

impl ::std::fmt::Debug for InventoryVector {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let reversed_hash = self.hash.iter().cloned().rev().collect::<Vec<u8>>();
        write!(f, "InventoryVector {{ {:?}, hash: {} }}", self.object_type, byte_slice_as_hex(&reversed_hash))
    }
}

#[derive(Clone, Debug)]
pub struct InvPayload {
    count: VarInt,
    inventory: Vec<InventoryVector>,
}

impl InvPayload {
    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        self.count.serialize(writer)?;

        for object in self.inventory.iter() {
            object.serialize(writer)?;
        }

        Ok(())
    }

    pub fn length(&self) -> usize {
        self.count.length() + self.inventory.len() * InventoryVector::length()
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<InvPayload, NetworkError> {
        let count = VarInt::deserialize(reader)?;
        let length = count.value();

        let mut inventory = vec![];

        for _ in 0..length {
            let curr_vector = InventoryVector::deserialize(reader)?;
            inventory.push(curr_vector);
        }

        Ok(InvPayload {
            count,
            inventory,
        })
    }
}