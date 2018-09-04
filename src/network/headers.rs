use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use hex::FromHex;
use network::NetworkError;
use network::varint::VarInt;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

#[derive(Clone, Copy)]
pub struct BlockHeader {
    version: i32,
    pub prev_block: [u8; 32],
    merkle_root: [u8; 32],
    timestamp: u32,
    bits: u32,
    nonce: u32,
    txn_count: VarInt,
}

impl PartialEq for BlockHeader {
    fn eq(&self, other: &BlockHeader) -> bool {
        self.hash() == other.hash()
    }
}
impl Eq for BlockHeader {}

impl BlockHeader {
    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        self.serialize_no_txn_count(writer)?;
        self.txn_count.serialize(writer)?;

        Ok(())
    }

    pub fn serialize_no_txn_count<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        writer.write_i32::<LittleEndian>(self.version)?;
        writer.write_all(&self.prev_block)?;
        writer.write_all(&self.merkle_root)?;
        writer.write_u32::<LittleEndian>(self.timestamp)?;
        writer.write_u32::<LittleEndian>(self.bits)?;
        writer.write_u32::<LittleEndian>(self.nonce)?;

        Ok(())
    }

    // TODO: likely make this a property in the struct so we don't have to calculate it all the time.
    pub fn hash(&self) -> Vec<u8> {
        let mut header_bytes = vec![];
        self.serialize_no_txn_count(&mut header_bytes).unwrap();

        let mut result = Vec::new();
        result.extend_from_slice(&Sha256::digest(&Sha256::digest(&header_bytes)));

        result
    }

    pub fn length(&self) -> usize {
        80 + self.txn_count.length()
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<BlockHeader, NetworkError> {
        let version = reader.read_i32::<LittleEndian>()?;
        let mut prev_block = [0; 32];
        reader.read_exact(&mut prev_block)?;
        let mut merkle_root = [0; 32];
        reader.read_exact(&mut merkle_root)?;
        let timestamp = reader.read_u32::<LittleEndian>()?;
        let bits = reader.read_u32::<LittleEndian>()?;;
        let nonce = reader.read_u32::<LittleEndian>()?;
        let txn_count = VarInt::deserialize(reader)?;

        Ok(BlockHeader {
            version,
            prev_block,
            merkle_root,
            timestamp,
            bits,
            nonce,
            txn_count,
        })
    }

    pub fn new_genesis() -> BlockHeader {
        let mut merkle_root = [0u8; 32];
        merkle_root.copy_from_slice(&Vec::from_hex("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b").unwrap().iter().cloned().rev().collect::<Vec<u8>>());

        BlockHeader {
            version: 1,
            prev_block: [0; 32],
            merkle_root,
            timestamp: 1296688602,
            bits: 0x1d00ffff,
            nonce: 414098458,
            txn_count: VarInt::new(1),
        }
    }
}

fn byte_slice_as_hex(slice: &[u8]) -> String {
    let mut result = String::new();
    for byte in slice {
        result.push_str(&format!("{:02x}", byte));
    }
    result
}

impl ::std::fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let reversed_prev_block = self.prev_block.iter().cloned().rev().collect::<Vec<u8>>();
        let reversed_merkle_root = self.merkle_root.iter().cloned().rev().collect::<Vec<u8>>();
        write!(f, "BlockHeader {{ version: {}, prev_block: {}, merkle_root: {}, timestamp: {}, bits: {}, nonce: {}, txn_count: {:?} }}",
            self.version,
            byte_slice_as_hex(&reversed_prev_block),
            byte_slice_as_hex(&reversed_merkle_root),
            self.timestamp,
            self.bits,
            self.nonce,
            self.txn_count)
    }
}

#[derive(Clone, Debug)]
pub struct HeadersPayload {
    pub headers: Vec<BlockHeader>,
    count: VarInt,
}

impl HeadersPayload {
    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        self.count.serialize(writer)?;

        for header in self.headers.iter() {
            header.serialize(writer)?;
        }

        Ok(())
    }

    pub fn length(&self) -> usize {
        self.count.length() + self.headers.iter().fold(0, |acc, elem| acc + elem.length())
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<HeadersPayload, NetworkError> {
        let count = VarInt::deserialize(reader)?;
        let length = count.value();

        let mut headers = vec![];

        for _ in 0..length {
            let curr_header = BlockHeader::deserialize(reader)?;
            headers.push(curr_header);
        }

        Ok(HeadersPayload {
            headers,
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use hex::FromHex;
    use super::*;

    #[test]
    fn genesis_block_hash() {
        let genesis_block = BlockHeader::new_genesis();

        assert_eq!(genesis_block.hash(), Vec::from_hex("000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943").unwrap().iter().cloned().rev().collect::<Vec<u8>>());
    }
}