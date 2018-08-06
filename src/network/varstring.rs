use std::io::{Read};

use network::NetworkError;
use network::varint::VarInt;

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

        let length = VarInt::new(self.data.len() as u64);
        result.extend_from_slice(&length.as_bytes());
        result.extend_from_slice(self.data.as_bytes());

        result
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<VarString, NetworkError> {
        let length = VarInt::deserialize(reader)?;
        let length: u64 = length.into();

        // TODO: this can be a problem if length is actually too big.
        let mut data = vec![0; length as usize];
        reader.read_exact(&mut data)?;
        let data = String::from_utf8(data)?;

        Ok(VarString {
            data,
        })
    }
}