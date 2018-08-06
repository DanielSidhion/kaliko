use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read};

use bitcoin::Network;
use network::{Command, NetworkError, NetworkValue};

#[derive(Debug)]
pub struct Message {
    pub network: Network,
    pub command: Command,
    checksum: u32,
}

impl Message {
    pub fn new(network: Network, command: Command) -> Message {
        let checksum = command.checksum();

        Message {
            network,
            command,
            checksum,
        }
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<Message, NetworkError> {
        let network = Network::from_u32(reader.read_u32::<LittleEndian>()?)?;
        let command = Command::deserialize(reader)?;

        Ok(Message::new(network, command))
    }
}

impl IntoIterator for Message {
    type Item = u8;
    type IntoIter = ::std::iter::Chain<::std::vec::IntoIter<u8>, ::std::vec::IntoIter<u8>>;

    fn into_iter(self) -> Self::IntoIter {
        let mut result = vec![];

        result.write_u32::<LittleEndian>(self.network.network_value()).unwrap();
        result.extend_from_slice(&self.command.name_as_bytes());
        result.write_u32::<LittleEndian>(self.command.length() as u32).unwrap();
        // Internal byte order.
        result.write_u32::<BigEndian>(self.checksum).unwrap();

        result.into_iter().chain(self.command.payload_as_bytes().into_iter())
    }
}