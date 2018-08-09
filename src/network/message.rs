use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

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

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        writer.write_u32::<LittleEndian>(self.network.network_value())?;
        writer.write_all(&self.command.name_as_bytes())?;
        writer.write_u32::<LittleEndian>(self.command.length() as u32)?;
        // Internal byte order.
        writer.write_u32::<BigEndian>(self.checksum)?;
        self.command.serialize(writer)?;

        Ok(())
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<Message, NetworkError> {
        let network = Network::from_u32(reader.read_u32::<LittleEndian>()?)?;
        let command = Command::deserialize(reader)?;

        Ok(Message::new(network, command))
    }
}