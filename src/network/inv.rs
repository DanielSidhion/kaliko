use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read};

use network::NetworkError;
use network::varint::VarInt;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct InventoryVector {
    object_type: InventoryType,
    hash: [u8; 32],
}

impl InventoryVector {
    pub fn as_bytes(&self) -> [u8; 36] {
        let mut result = [0; 36];

        LittleEndian::write_u32(&mut result[0..4], self.object_type.value());
        &result[4..36].copy_from_slice(&self.hash);

        result
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

#[derive(Debug)]
pub struct InvPayload {
    count: VarInt,
    inventory: Vec<InventoryVector>,
}

impl InvPayload {
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = vec![];

        result.extend_from_slice(&self.count.as_bytes());
        for object in self.inventory.iter() {
            result.extend_from_slice(&object.as_bytes());
        }

        result
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