extern crate byteorder;
extern crate hex;
extern crate hmac;
#[macro_use] extern crate itertools;
#[macro_use] extern crate log;
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

use network::Message;
use network::headers::BlockHeader;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum KalikoControlMessage {
    NetworkMessage(SocketAddr, Message),
    StartPeerConnection(SocketAddr),
    PeerUnavailable(SocketAddr),
    PeerConnectionDestroyed(SocketAddr),
    PeerConnectionEstablished(SocketAddr, Sender<KalikoControlMessage>),
    PeerAnnouncedHeight(SocketAddr, i32),
    // TODO: likely wrap the message to be delivered under another enum/struct.
    RequestHeadersFromPeer(SocketAddr, Vec<Vec<u8>>),
    RequestHeaders(Vec<Vec<u8>>),
    NewHeadersAvailable(SocketAddr, Vec<BlockHeader>),
}