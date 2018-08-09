use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use network::NetworkError;

#[derive(Debug)]
pub struct NetworkAddress {
    time: u32,
    services: u64,
    ip: [u8; 16],
    port: i16,
}

// TODO: join normal implementations with the ones using _no_time() for better code reuse.
impl NetworkAddress {
    pub fn new() -> NetworkAddress {
        NetworkAddress {
            time: 0,
            services: 0,
            ip: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 127, 0, 0, 1],
            port: 0,
        }
    }

    pub fn length() -> usize {
        30
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        writer.write_u32::<LittleEndian>(self.time)?;
        self.serialize_no_time(writer)?;

        Ok(())
    }

    pub fn serialize_no_time<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        writer.write_u64::<LittleEndian>(self.services)?;
        writer.write_all(&self.ip)?;
        writer.write_i16::<BigEndian>(self.port)?;

        Ok(())
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

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<NetworkAddress, NetworkError> {
        let time = reader.read_u32::<LittleEndian>()?;
        let services = reader.read_u64::<LittleEndian>()?;
        let mut ip = [0; 16];
        reader.read_exact(&mut ip)?;
        let port = reader.read_i16::<BigEndian>()?;

        Ok(NetworkAddress {
            time,
            services,
            ip,
            port,
        })
    }
}