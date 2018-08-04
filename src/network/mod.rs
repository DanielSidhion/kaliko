use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use sha2::{Sha256, Digest};

use bitcoin::Network;
use std::time::{SystemTime, UNIX_EPOCH};

trait NetworkValue {
    fn network_value(&self) -> u32;
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
}

pub enum Command {
    Version(VersionPayload),
}

impl Command {
    pub fn name_as_bytes(&self) -> [u8; 12] {
        match *self {
            Command::Version(_) => [b'v', b'e', b'r', b's', b'i', b'o', b'n', 0, 0, 0, 0, 0],
        }
    }

    pub fn payload_as_bytes(&self) -> Vec<u8> {
        match *self {
            Command::Version(ref p) => p.serialize(),
        }
    }

    pub fn length(&self) -> u32 {
        match *self {
            Command::Version(_) => VersionPayload::length(),
        }
    }
}

pub struct Message {
    network: Network,
    command: Command,
    checksum: u32,
}

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
        LittleEndian::write_i16(&mut result[24..26], self.port);

        result
    }
}

pub struct VersionPayload {
    version: i32,
    services: u64,
    timestamp: i64,
    addr_recv: NetworkAddress,
    addr_from: NetworkAddress,
    nonce: u64,
    user_agent: String,
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
        result.push(0);
        result.write_i32::<LittleEndian>(self.start_height);
        result.push(self.relay as u8);

        result
    }

    pub fn length() -> u32 {
        4 + 8 + 8 + (8 + 16 + 2) + (8 + 16 + 2) + 8 + 1 + 4 + 1
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
            user_agent: String::new(),
            start_height: 0,
            relay: false,
        }
    }
}

impl Message {
    pub fn new(network: Network, command: Command) -> Message {
        let dhash = Sha256::digest(Sha256::digest(&command.payload_as_bytes()).as_slice());

        Message {
            network,
            command,
            checksum: BigEndian::read_u32(&dhash[..4]),
        }
    }
}

impl IntoIterator for Message {
    type Item = u8;
    type IntoIter = ::std::iter::Chain<::std::vec::IntoIter<u8>, ::std::vec::IntoIter<u8>>;

    fn into_iter(self) -> Self::IntoIter {
        let mut result = vec![];

        result.write_u32::<LittleEndian>(self.network.network_value()).unwrap();
        result.extend_from_slice(&self.command.name_as_bytes());
        result.write_u32::<LittleEndian>(self.command.length()).unwrap();
        // Internal byte order.
        result.write_u32::<BigEndian>(self.checksum).unwrap();

        result.into_iter().chain(self.command.payload_as_bytes().into_iter())
    }
}