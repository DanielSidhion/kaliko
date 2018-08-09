use std::io::{Read, Write};

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

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        let length = VarInt::new(self.data.len() as u64);
        length.serialize(writer)?;
        writer.write_all(&self.data.as_bytes())?;

        Ok(())
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