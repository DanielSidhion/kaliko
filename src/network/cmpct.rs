use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read};

use network::NetworkError;

#[derive(Debug)]
pub struct SendCmpctPayload {
    announce: bool,
    version: u64,
}

impl SendCmpctPayload {
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = vec![];

        result.write_u8(self.announce as u8).unwrap();
        result.write_u64::<LittleEndian>(self.version).unwrap();

        result
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<SendCmpctPayload, NetworkError> {
        let announce = reader.read_u8()? == 1;
        let version = reader.read_u64::<LittleEndian>()?;

        Ok(SendCmpctPayload{
            announce,
            version,
        })
    }

    pub fn length() -> usize {
        9
    }

    pub fn new() -> SendCmpctPayload {
        SendCmpctPayload {
            announce: false,
            version: 1,
        }
    }
}