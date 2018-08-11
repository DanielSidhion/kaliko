use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hex::FromHex;
use network::NetworkError;
use network::varint::VarInt;
use std::io::{Read, Write};

#[derive(Debug)]
pub struct GetBlocksPayload {
    version: u32,
    hash_count: VarInt,
    block_locator_hashes: Vec<[u8; 32]>,
    hash_stop: [u8; 32],
}

impl GetBlocksPayload {
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

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<GetBlocksPayload, NetworkError> {
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

        Ok(GetBlocksPayload {
            version,
            hash_count,
            block_locator_hashes,
            hash_stop,
        })
    }

    pub fn new() -> GetBlocksPayload {
        let genesis_hash = Vec::from_hex("00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206").unwrap().iter().cloned().rev().collect::<Vec<u8>>();
        let mut genesis_block_hash = [0u8; 32];
        genesis_block_hash.copy_from_slice(&genesis_hash);
        let block4 = Vec::from_hex("000000008b5d0af9ffb1741e38b17b193bd12d7683401cecd2fd94f548b6e5dd").unwrap().iter().cloned().rev().collect::<Vec<u8>>();
        let mut hash_stop = [0u8; 32];
        hash_stop.copy_from_slice(&block4);

        GetBlocksPayload {
            version: 70015,
            hash_count: VarInt::new(1),
            block_locator_hashes: vec![genesis_block_hash],
            hash_stop,
        }
    }
}