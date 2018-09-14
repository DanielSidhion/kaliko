use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hex::FromHex;
use network::NetworkError;
use network::varint::VarInt;
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct GetBlocksOrHeadersPayload {
    version: u32,
    hash_count: VarInt,
    block_locator_hashes: Vec<[u8; 32]>,
    hash_stop: [u8; 32],
}

impl GetBlocksOrHeadersPayload {
    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        writer.write_u32::<LittleEndian>(self.version)?;
        self.hash_count.serialize(writer)?;

        for hash in self.block_locator_hashes.iter() {
            writer.write_all(hash)?;
        }

        writer.write_all(&self.hash_stop)?;

        Ok(())
    }

    pub fn length(&self) -> usize {
        4 + self.hash_count.length() + 32 * self.block_locator_hashes.len() + 32
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<GetBlocksOrHeadersPayload, NetworkError> {
        let version = reader.read_u32::<LittleEndian>()?;
        let hash_count = VarInt::deserialize(reader)?;
        
        let total_locator_hashes = hash_count.value();
        let mut block_locator_hashes = vec![];

        // TODO: do something against huge lengths.
        for _ in 0..total_locator_hashes {
            let mut curr_result = [0u8; 32];
            reader.read_exact(&mut curr_result)?;
            block_locator_hashes.push(curr_result);
        }

        let mut hash_stop = [0u8; 32];
        reader.read_exact(&mut hash_stop)?;

        Ok(GetBlocksOrHeadersPayload {
            version,
            hash_count,
            block_locator_hashes,
            hash_stop,
        })
    }

    pub fn new(block_locator: Vec<Vec<u8>>) -> GetBlocksOrHeadersPayload {
        let mut block_locator_hashes = vec![];

        for hash in block_locator {
            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&hash);
            block_locator_hashes.push(hash_array);
        }

        GetBlocksOrHeadersPayload {
            version: 70015,
            hash_count: VarInt::new(1),
            block_locator_hashes,
            hash_stop: [0u8; 32],
        }
    }
}