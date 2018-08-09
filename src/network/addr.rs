use std::io::{Read, Write};

use network::NetworkError;
use network::networkaddress::NetworkAddress;
use network::varint::VarInt;

#[derive(Debug)]
pub struct AddrPayload {
    count: VarInt,
    addr_list: Vec<NetworkAddress>,
}

impl AddrPayload {
    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        self.count.serialize(writer)?;

        for addr in self.addr_list.iter() {
            addr.serialize(writer)?;
        }

        Ok(())
    }

    pub fn length(&self) -> usize {
        self.count.length() + self.addr_list.len() * NetworkAddress::length()
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<AddrPayload, NetworkError> {
        let count = VarInt::deserialize(reader)?;
        let total_addrs = count.value();

        let mut addr_list: Vec<NetworkAddress> = vec![];

        // TODO: do something against huge lengths.
        for _ in 0..total_addrs {
            let curr_addr = NetworkAddress::deserialize(reader)?;
            addr_list.push(curr_addr);
        }

        Ok(AddrPayload {
            count,
            addr_list,
        })
    }
}