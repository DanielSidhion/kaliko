use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use sha2::{Sha256, Digest};

use bitcoin::Network;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::prelude::*;

#[derive(Debug)]
pub enum NetworkError {
    NotEnoughData,
    MalformedUTF8String,
    UnknownNetworkIdentifier,
    InvalidCommand,
    InvalidChecksum,
}

// TODO: work on this.
impl From<::std::io::Error> for NetworkError {
    fn from(error: ::std::io::Error) -> NetworkError {
        match error.kind() {
            _ => NetworkError::NotEnoughData,
        }
    }
}

impl From<::std::string::FromUtf8Error> for NetworkError {
    fn from(_: ::std::string::FromUtf8Error) -> NetworkError {
        NetworkError::MalformedUTF8String
    }
}

trait NetworkValue {
    fn network_value(&self) -> u32;
    fn from_u32(value: u32) -> Result<Network, NetworkError>;
}

impl NetworkValue for Network {
    fn network_value(&self) -> u32 {
        match *self {
            Network::Mainnet => 0xD9B4BEF9,
            Network::Testnet => 0xDAB5BFFA,
            Network::Testnet3 => 0x0709110B,
            Network::Namecoin => 0xFEB4BEF9,
        }
    }

    fn from_u32(value: u32) -> Result<Network, NetworkError> {
        match value {
            0xD9B4BEF9 => Ok(Network::Mainnet),
            0xDAB5BFFA => Ok(Network::Testnet),
            0x0709110B => Ok(Network::Testnet3),
            0xFEB4BEF9 => Ok(Network::Namecoin),
            _ => Err(NetworkError::UnknownNetworkIdentifier)
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Version(VersionPayload),
    Verack,
    Ping(u64),
    Pong(u64),
}

const VERSION_COMMAND: [u8; 12] = [b'v', b'e', b'r', b's', b'i', b'o', b'n', 0, 0, 0, 0, 0];
const VERACK_COMMAND: [u8; 12] = [b'v', b'e', b'r', b'a', b'c', b'k', 0, 0, 0, 0, 0, 0];
const PING_COMMAND: [u8; 12] = [b'p', b'i', b'n', b'g', 0, 0, 0, 0, 0, 0, 0, 0];
const PONG_COMMAND: [u8; 12] = [b'p', b'o', b'n', b'g', 0, 0, 0, 0, 0, 0, 0, 0];

impl Command {
    pub fn name_as_bytes(&self) -> [u8; 12] {
        match *self {
            Command::Version(_) => VERSION_COMMAND,
            Command::Verack => VERACK_COMMAND,
            Command::Ping(_) => PING_COMMAND,
            Command::Pong(_) => PONG_COMMAND,
        }
    }

    pub fn payload_as_bytes(&self) -> Vec<u8> {
        match *self {
            Command::Version(ref p) => p.serialize(),
            Command::Verack => vec![],
            Command::Ping(p) | Command::Pong(p) => {
                let mut result = vec![];
                result.write_u64::<LittleEndian>(p).unwrap();
                result
            },
        }
    }

    pub fn length(&self) -> usize {
        match *self {
            Command::Version(_) => VersionPayload::length(),
            Command::Verack => 0,
            Command::Ping(_) => 8,
            Command::Pong(_) => 8,
        }
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<Command, NetworkError> {
        let mut command_bytes = [0u8; 12];
        reader.read_exact(&mut command_bytes)?;

        println!("Got the following command bytes: {:x?}", command_bytes);

        // To get the command payload, we need to read length and checksum first.
        let length = reader.read_u32::<LittleEndian>()?;
        let checksum = reader.read_u32::<BigEndian>()?;

        // Constraining reader to read at most `length` bytes.
        // Should probably do some validation of length here to prevent huge lengths.
        let mut constrained_reader = reader.take(length as u64);

        let result = match command_bytes {
            VERSION_COMMAND => Command::Version(VersionPayload::deserialize(&mut constrained_reader)?),
            VERACK_COMMAND => Command::Verack,
            PING_COMMAND => {
                let result = constrained_reader.read_u64::<LittleEndian>()?;
                Command::Ping(result)
            },
            PONG_COMMAND => {
                let result = constrained_reader.read_u64::<LittleEndian>()?;
                Command::Pong(result)
            },
            _ => return Err(NetworkError::InvalidCommand),
        };

        let actual_checksum = result.checksum();
        if checksum != actual_checksum {
            return Err(NetworkError::InvalidChecksum);
        }

        Ok(result)
    }

    pub fn checksum(&self) -> u32 {
        let bytes = self.payload_as_bytes();
        let dhash = Sha256::digest(Sha256::digest(&bytes[..]).as_slice());

        BigEndian::read_u32(&dhash[..4])
    }
}

// TODO: this whole thing doesn't need to exist if we have a NetworkDeserializer trait or something like this and implement it for string.
#[derive(Debug)]
pub struct VarString {
    data: String,
}

impl VarString {
    pub fn new() -> VarString {
        VarString{
            data: String::new(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut result = vec![];

        match self.data.len() {
            0 ... 0xFC => {
                result.write_u8(self.data.len() as u8).unwrap();
                result.extend_from_slice(self.data.as_bytes());
            },
            0xFD ... 0xFFFF => {
                result.write_u8(0xFD).unwrap();
                result.write_u16::<LittleEndian>(self.data.len() as u16).unwrap();
            },
            0x1_0000 ... 0xFFFF_FFFF => {
                result.write_u8(0xFE).unwrap();
                result.write_u32::<LittleEndian>(self.data.len() as u32).unwrap();
            },
            _ => {
                result.write_u8(0xFF).unwrap();
                result.write_u64::<LittleEndian>(self.data.len() as u64).unwrap();
            }
        }

        result
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<VarString, NetworkError> {
        let length_type = reader.read_u8()?;
        let length;

        match length_type {
            0xFF => {
                length = reader.read_u64::<LittleEndian>()?;
            },
            0xFE => {
                length = reader.read_u32::<LittleEndian>()? as u64;
            },
            0xFD => {
                length = reader.read_u16::<LittleEndian>()? as u64;
            },
            _ => {
                length = length_type as u64;
            }
        }

        // TODO: this can be a problem if length is actually too big.
        let mut data = vec![0; length as usize];
        reader.read_exact(&mut data)?;
        let data = String::from_utf8(data)?;

        Ok(VarString {
            data,
        })
    }
}

#[derive(Debug)]
pub struct NetworkAddress {
    time: u32,
    services: u64,
    ip: [u8; 16],
    port: i16,
}

impl NetworkAddress {
    pub fn as_bytes_no_time(&self) -> [u8; 26] {
        let mut result = [0; 26];

        LittleEndian::write_u64(&mut result[0..8], self.services);
        result[8..24].copy_from_slice(&self.ip);
        BigEndian::write_i16(&mut result[24..26], self.port);

        result
    }

    pub fn length() -> usize {
        30
    }

    pub fn from_slice_no_time(slice: &[u8]) -> Result<NetworkAddress, NetworkError> {
        match slice.len() {
            26 => {
                let services = LittleEndian::read_u64(&slice[..8]);
                let mut ip = [0; 16];
                ip.copy_from_slice(&slice[8..24]);
                let port = BigEndian::read_i16(&slice[24..]);

                Ok(NetworkAddress {
                    time: 0,
                    services,
                    ip,
                    port,
                })
            },
            _ => Err(NetworkError::NotEnoughData)
        }
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
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = vec![];

        result.write_i32::<LittleEndian>(self.version);
        result.write_u64::<LittleEndian>(self.services);
        result.write_i64::<LittleEndian>(self.timestamp);
        result.extend_from_slice(&self.addr_recv.as_bytes_no_time());
        result.extend_from_slice(&self.addr_from.as_bytes_no_time());
        result.write_u64::<LittleEndian>(self.nonce);
        result.extend_from_slice(&self.user_agent.as_bytes());
        result.write_i32::<LittleEndian>(self.start_height);
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

        let recipient = NetworkAddress {
            time: since_epoch as u32,
            services: 0,
            ip: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 185, 28, 76, 179],
            port: 18333,
        };

        let origin = NetworkAddress {
            time: since_epoch as u32,
            services: 0,
            ip: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 127, 0, 0, 1],
            port: 18333,
        };

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