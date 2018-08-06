use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt};
use std::io::{Read};

use network::NetworkError;

#[derive(Debug)]
pub struct NetworkAddress {
    time: u32,
    services: u64,
    ip: [u8; 16],
    port: i16,
}

impl NetworkAddress {
    pub fn new() -> NetworkAddress {
        NetworkAddress {
            time: 0,
            services: 0,
            ip: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 127, 0, 0, 1],
            port: 0,
        }
    }

    pub fn as_bytes_no_time(&self) -> [u8; 26] {
        let mut result = [0; 26];

        LittleEndian::write_u64(&mut result[0..8], self.services);
        result[8..24].copy_from_slice(&self.ip);
        BigEndian::write_i16(&mut result[24..26], self.port);

        result
    }

    pub fn deserialize_no_time<R: Read>(reader: &mut R) -> Result<NetworkAddress, NetworkError> {
        let services = reader.read_u64::<LittleEndian>()?;
        let mut ip = [0; 16];
        reader.read_exact(&mut ip)?;
        let port = reader.read_i16::<BigEndian>()?;

        Ok(NetworkAddress {
            time: 0,
            services,
            ip,
            port,
        })
    }
}