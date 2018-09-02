use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use network::NetworkError;

fn socket_v6_to_v4(socket: SocketAddrV6) -> Option<SocketAddr> {
    let ip = socket.ip();

    match ip.to_ipv4() {
        Some(ipv4) => {
            Some(SocketAddr::V4(SocketAddrV4::new(ipv4, socket.port())))
        },
        None => None,
    }
}

#[derive(Clone, Debug)]
pub struct NetworkAddress {
    time: u32,
    services: u64,
    socket_addr: SocketAddrV6,
}

impl NetworkAddress {
    pub fn new() -> NetworkAddress {
        NetworkAddress {
            time: 0,
            services: 0,
            socket_addr: SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 0, 0, 0),
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        // If we have an IPv4-mapped IPv6 address, convert it back to IPv4.
        match socket_v6_to_v4(self.socket_addr) {
            Some(s) => s,
            None => SocketAddr::V6(self.socket_addr),
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
        writer.write_all(&self.socket_addr.ip().octets())?;
        writer.write_u16::<BigEndian>(self.socket_addr.port())?;

        Ok(())
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<NetworkAddress, NetworkError> {
        let time = reader.read_u32::<LittleEndian>()?;

        let mut addr_no_time = NetworkAddress::deserialize_no_time(reader)?;
        addr_no_time.time = time;

        Ok(addr_no_time)
    }

    pub fn deserialize_no_time<R: Read>(reader: &mut R) -> Result<NetworkAddress, NetworkError> {
        let services = reader.read_u64::<LittleEndian>()?;
        let mut ip = [0; 16];
        reader.read_exact(&mut ip)?;
        let port = reader.read_u16::<BigEndian>()?;

        Ok(NetworkAddress {
            time: 0,
            services,
            socket_addr: SocketAddrV6::new(Ipv6Addr::from(ip), port, 0, 0),
        })
    }
}