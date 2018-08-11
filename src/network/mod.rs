use bitcoin::Network;

mod addr;
pub mod blocks;
pub mod cmpct;
pub mod command;
mod inv;
pub mod message;
mod networkaddress;
mod varint;
mod varstring;
pub mod version;

pub use self::command::Command;
pub use self::message::Message;

#[derive(Debug)]
pub enum NetworkError {
    NotEnoughData,
    MalformedUTF8String,
    UnknownNetworkIdentifier,
    InvalidCommand(String),
    InvalidChecksum,
    InvalidValue,
    PeerClosedConnection,
}

// TODO: work on this.
impl From<::std::io::Error> for NetworkError {
    fn from(error: ::std::io::Error) -> NetworkError {
        println!("Got std::io::Error! Need to convert to NetworkError");
        println!("Error: {:#?}", error);
        match error.kind() {
            ::std::io::ErrorKind::UnexpectedEof => NetworkError::PeerClosedConnection,
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