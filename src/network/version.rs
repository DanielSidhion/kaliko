use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read};
use std::time::{SystemTime, UNIX_EPOCH};

use network::NetworkError;
use network::networkaddress::NetworkAddress;
use network::varstring::VarString;

#[derive(Debug)]
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
    
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = vec![];

        result.write_i32::<LittleEndian>(self.version).unwrap();
        result.write_u64::<LittleEndian>(self.services).unwrap();
        result.write_i64::<LittleEndian>(self.timestamp).unwrap();
        result.extend_from_slice(&self.addr_recv.as_bytes_no_time());
        result.extend_from_slice(&self.addr_from.as_bytes_no_time());
        result.write_u64::<LittleEndian>(self.nonce).unwrap();
        result.extend_from_slice(&self.user_agent.as_bytes());
        result.write_i32::<LittleEndian>(self.start_height).unwrap();
        result.push(self.relay as u8);

        result
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

    pub fn new() -> VersionPayload {
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
            nonce: 0,
            user_agent: VarString::new(),
            start_height: 0,
            relay: false,
        }
    }
}