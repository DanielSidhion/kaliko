use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use network::NetworkError;
use network::varint::VarInt;
use std::io::{Read, Write};

pub struct BlockHeader {
    version: i32,
    prev_block: [u8; 32],
    merkle_root: [u8; 32],
    timestamp: u32,
    bits: u32,
    nonce: u32,
    txn_count: VarInt,
}

impl BlockHeader {
    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        writer.write_i32::<LittleEndian>(self.version)?;
        writer.write_all(&self.prev_block)?;
        writer.write_all(&self.merkle_root)?;
        writer.write_u32::<LittleEndian>(self.timestamp)?;
        writer.write_u32::<LittleEndian>(self.bits)?;
        writer.write_u32::<LittleEndian>(self.nonce)?;
        self.txn_count.serialize(writer)?;

        Ok(())
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

#[derive(Debug)]
pub struct HeadersPayload {
    count: VarInt,
    headers: Vec<BlockHeader>,
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
            count,
            headers,
        })
    }
}