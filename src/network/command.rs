use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

use network::NetworkError;
use network::addr::AddrPayload;
use network::blocks::GetBlocksOrHeadersPayload;
use network::cmpct::SendCmpctPayload;
use network::headers::HeadersPayload;
use network::inv::InvPayload;
use network::version::VersionPayload;

#[derive(Debug)]
pub enum Command {
    Version(VersionPayload),
    Verack,
    SendHeaders,
    SendCmpct(SendCmpctPayload),
    Addr(AddrPayload),
    Feefilter(u64),
    Inv(InvPayload),
    GetBlocks(GetBlocksOrHeadersPayload),
    GetHeaders(GetBlocksOrHeadersPayload),
    Headers(HeadersPayload),
    Ping(u64),
    Pong(u64),
}

const VERSION_COMMAND: [u8; 12] = [b'v', b'e', b'r', b's', b'i', b'o', b'n', 0, 0, 0, 0, 0];
const VERACK_COMMAND: [u8; 12] = [b'v', b'e', b'r', b'a', b'c', b'k', 0, 0, 0, 0, 0, 0];
const SENDHEADERS_COMMAND: [u8; 12] = [b's', b'e', b'n', b'd', b'h', b'e', b'a', b'd', b'e', b'r', b's', 0];
const SENDCMPCT_COMMAND: [u8; 12] = [b's', b'e', b'n', b'd', b'c', b'm', b'p', b'c', b't', 0, 0, 0];
const ADDR_COMMAND: [u8; 12] = [b'a', b'd', b'd', b'r', 0, 0, 0, 0, 0, 0, 0, 0];
const FEEFILTER_COMMAND: [u8; 12] = [b'f', b'e', b'e', b'f', b'i', b'l', b't', b'e', b'r', 0, 0, 0];
const INV_COMMAND: [u8; 12] = [b'i', b'n', b'v', 0, 0, 0, 0, 0, 0, 0, 0, 0];
const GETBLOCKS_COMMAND: [u8; 12] = [b'g', b'e', b't', b'b', b'l', b'o', b'c', b'k', b's', 0, 0, 0];
const GETHEADERS_COMMAND: [u8; 12] = [b'g', b'e', b't', b'h', b'e', b'a', b'd', b'e', b'r', b's', 0, 0];
const HEADERS_COMMAND: [u8; 12] = [b'h', b'e', b'a', b'd', b'e', b'r', b's', 0, 0, 0, 0, 0];
const PING_COMMAND: [u8; 12] = [b'p', b'i', b'n', b'g', 0, 0, 0, 0, 0, 0, 0, 0];
const PONG_COMMAND: [u8; 12] = [b'p', b'o', b'n', b'g', 0, 0, 0, 0, 0, 0, 0, 0];

impl Command {
    pub fn name(&self) -> &str {
        match *self {
            Command::Version(_) => "version",
            Command::Verack => "verack",
            Command::SendHeaders => "sendheaders",
            Command::SendCmpct(_) => "sendcmpct",
            Command::Addr(_) => "addr",
            Command::Feefilter(_) => "feefilter",
            Command::Inv(_) => "inv",
            Command::GetBlocks(_) => "getblocks",
            Command::GetHeaders(_) => "getheaders",
            Command::Headers(_) => "headers",
            Command::Ping(_) => "ping",
            Command::Pong(_) => "pong",
        }
    }

    pub fn name_as_bytes(&self) -> [u8; 12] {
        match *self {
            Command::Version(_) => VERSION_COMMAND,
            Command::Verack => VERACK_COMMAND,
            Command::SendHeaders => SENDHEADERS_COMMAND,
            Command::SendCmpct(_) => SENDCMPCT_COMMAND,
            Command::Addr(_) => ADDR_COMMAND,
            Command::Feefilter(_) => FEEFILTER_COMMAND,
            Command::Inv(_) => INV_COMMAND,
            Command::GetBlocks(_) => GETBLOCKS_COMMAND,
            Command::GetHeaders(_) => GETHEADERS_COMMAND,
            Command::Headers(_) => HEADERS_COMMAND,
            Command::Ping(_) => PING_COMMAND,
            Command::Pong(_) => PONG_COMMAND,
        }
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), NetworkError> {
        match *self {
            Command::Version(ref p) => p.serialize(writer)?,
            Command::SendCmpct(ref p) => p.serialize(writer)?,
            Command::Addr(ref p) => p.serialize(writer)?,
            Command::Feefilter(p) | Command::Ping(p) | Command::Pong(p) => writer.write_u64::<LittleEndian>(p)?,
            Command::Inv(ref p) => p.serialize(writer)?,
            Command::GetBlocks(ref p) | Command::GetHeaders(ref p) => p.serialize(writer)?,
            Command::Headers(ref p) => p.serialize(writer)?,
            Command::Verack | Command::SendHeaders => (),
        }

        Ok(())
    }

    pub fn length(&self) -> usize {
        match *self {
            Command::Version(_) => VersionPayload::length(),
            Command::Verack => 0,
            Command::SendHeaders => 0,
            Command::SendCmpct(_) => SendCmpctPayload::length(),
            Command::Addr(ref p) => p.length(),
            Command::Feefilter(_) => 8,
            Command::Inv(ref p) => p.length(),
            Command::GetBlocks(ref p) => p.length(),
            Command::GetHeaders(ref p) => p.length(),
            Command::Headers(ref p) => p.length(),
            Command::Ping(_) => 8,
            Command::Pong(_) => 8,
        }
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> Result<Command, NetworkError> {
        let mut command_bytes = [0u8; 12];
        reader.read_exact(&mut command_bytes)?;

        // To get the command payload, we need to read length and checksum first.
        let length = reader.read_u32::<LittleEndian>()?;
        let checksum = reader.read_u32::<BigEndian>()?;

        // Constraining reader to read at most `length` bytes.
        // Should probably do some validation of length here to prevent huge lengths.
        let mut constrained_reader = reader.take(length as u64);

        let result = match command_bytes {
            VERSION_COMMAND => Command::Version(VersionPayload::deserialize(&mut constrained_reader)?),
            VERACK_COMMAND => Command::Verack,
            SENDHEADERS_COMMAND => Command::SendHeaders,
            SENDCMPCT_COMMAND => Command::SendCmpct(SendCmpctPayload::deserialize(&mut constrained_reader)?),
            ADDR_COMMAND => Command::Addr(AddrPayload::deserialize(&mut constrained_reader)?),
            FEEFILTER_COMMAND => {
                let result = constrained_reader.read_u64::<LittleEndian>()?;
                Command::Feefilter(result)
            },
            INV_COMMAND => Command::Inv(InvPayload::deserialize(&mut constrained_reader)?),
            GETBLOCKS_COMMAND => Command::GetBlocks(GetBlocksOrHeadersPayload::deserialize(&mut constrained_reader)?),
            GETHEADERS_COMMAND => Command::GetHeaders(GetBlocksOrHeadersPayload::deserialize(&mut constrained_reader)?),
            HEADERS_COMMAND => Command::Headers(HeadersPayload::deserialize(&mut constrained_reader)?),
            PING_COMMAND => {
                let result = constrained_reader.read_u64::<LittleEndian>()?;
                Command::Ping(result)
            },
            PONG_COMMAND => {
                let result = constrained_reader.read_u64::<LittleEndian>()?;
                Command::Pong(result)
            },
            _ => {
                let mut vec = vec![];
                vec.extend_from_slice(&command_bytes);
                let command_name = String::from_utf8(vec).unwrap();
                return Err(NetworkError::InvalidCommand(command_name))
            },
        };

        let actual_checksum = result.checksum();
        if checksum != actual_checksum {
            return Err(NetworkError::InvalidChecksum);
        }

        Ok(result)
    }

    pub fn checksum(&self) -> u32 {
        let mut bytes = vec![];
        self.serialize(&mut bytes).unwrap();
        let dhash = Sha256::digest(Sha256::digest(&bytes[..]).as_slice());

        BigEndian::read_u32(&dhash[..4])
    }
}