use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read};

use network::NetworkError;
use network::networkaddress::NetworkAddress;
use network::varint::VarInt;

pub enum InventoryType {
    Error,
    Msg_Tx,
    Msg_Block,
    Msg_Filtered_Block,
    Msg_Cmpct_Block,
}

pub struct InventoryVector {
    type: InventoryType,
    hash: [u8; 32],
}

#[derive(Debug)]
pub struct InvPayload {
    count: VarInt,
    inventory: Vec<NetworkAddress>,
}

impl InvPayload {
    pub fn serialize(&self) -> Vec<u8> {
    }

    pub fn length(&self) -> usize {
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<InvPayload, NetworkError> {
    }
}