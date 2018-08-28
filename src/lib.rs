extern crate byteorder;
extern crate hex;
extern crate hmac;
#[macro_use] extern crate itertools;
extern crate rand;
extern crate ring;
extern crate ripemd160;
extern crate secp256k1;
extern crate sha2;

pub mod base58;
pub mod bip32;
pub mod bip39;
pub mod bip44;
pub mod bitcoin;
pub mod network;
pub mod peer;
pub mod storage;
pub mod util;

use network::headers::BlockHeader;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub enum KalikoControlMessage {
    StartPeerConnectionFromSocketAddr(SocketAddr),
    StartPeerConnectionFromString(String),
    PeerConnectionDestroyed(SocketAddr),
    PeerAnnouncedHeight(i32),
    RequestHeaders(Vec<u8>),
    NewHeadersAvailable(Vec<BlockHeader>),
}