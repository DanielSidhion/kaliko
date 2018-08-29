use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use network::NetworkError;
use network::networkaddress::NetworkAddress;
use network::varstring::VarString;

#[derive(Clone, Debug)]
pub struct VersionPayload {
    version: i32,
    services: u64,
    timestamp: i64,
    addr_recv: NetworkAddress,
    addr_from: NetworkAddress,
    nonce: u64,
    user_agent: VarString,
    start_height: i32,
    relay: bool,
}

impl VersionPayload {
    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn start_height(&self) -> i32 {
        self.start_height
    }
    
    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        writer.write_i32::<LittleEndian>(self.version)?;
        writer.write_u64::<LittleEndian>(self.services)?;
        writer.write_i64::<LittleEndian>(self.timestamp)?;

        self.addr_recv.serialize_no_time(writer)?;
        self.addr_from.serialize_no_time(writer)?;

        writer.write_u64::<LittleEndian>(self.nonce)?;

        self.user_agent.serialize(writer)?;

        writer.write_i32::<LittleEndian>(self.start_height)?;
        writer.write_u8(self.relay as u8)?;

        Ok(())
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<VersionPayload, NetworkError> {
        let version = reader.read_i32::<LittleEndian>()?;
        let services = reader.read_u64::<LittleEndian>()?;
        let timestamp = reader.read_i64::<LittleEndian>()?;

        let addr_recv = NetworkAddress::deserialize_no_time(reader)?;
        let addr_from = NetworkAddress::deserialize_no_time(reader)?;
        let nonce = reader.read_u64::<LittleEndian>()?;
        let user_agent = VarString::deserialize(reader)?;
        let start_height = reader.read_i32::<LittleEndian>()?;
        let relay = reader.read_u8()? == 1;

        Ok(VersionPayload{
            version,
            services,
            timestamp,
            addr_recv,
            addr_from,
            nonce,
            user_agent,
            start_height,
            relay,
        })
    }

    pub fn length() -> usize {
        86
    }

    pub fn new(nonce: u64) -> VersionPayload {
        let time = SystemTime::now();
        let since_epoch = time.duration_since(UNIX_EPOCH).unwrap().as_secs();

        let recipient = NetworkAddress::new();
        let origin = NetworkAddress::new();

        VersionPayload {
            version: 70015,
            services: 0,
            timestamp: since_epoch as i64,
            addr_recv: recipient,
            addr_from: origin,
            nonce,
            user_agent: VarString::new(),
            start_height: 0,
            relay: false,
        }
    }
}