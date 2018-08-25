extern crate kaliko;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate toml;

use kaliko::network::{Command, Message};
use kaliko::peer;
use kaliko::peer::{PeerControlMessage, PeerConnection};
use kaliko::storage::BlockHeaderStorage;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::net::{SocketAddr, SocketAddrV6, TcpStream, ToSocketAddrs};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;

fn byte_slice_as_hex(slice: &[u8]) -> String {
    let mut result = String::new();
    for byte in slice {
        result.push_str(&format!("{:02x}", byte));
    }
    result
}

#[derive(Deserialize)]
struct Config {
    storage_location: String,
    peer_seed_list: String,
    max_active_peers: usize,
}

fn main() {
    let mut config_file = File::open("kaliko.toml").unwrap();
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).unwrap();

    let config: Config = toml::from_str(&contents).unwrap();
    println!("storage_location = {}", config.storage_location);

    let mut storage = BlockHeaderStorage::new(&config.storage_location);

    let mut initial_peers = peer::read_peer_list(&config.peer_seed_list);
    let (msg_tx, msg_rx) = mpsc::channel();
    let peer_manager = peer::PeerManager::new(msg_tx.clone(), config.max_active_peers);
    let peer_manager_channel = peer_manager.control_sender();
    peer_manager.start();

    while let Some(addr) = initial_peers.pop() {
        println!("Sending message to connect to {}", addr);
        peer_manager_channel.send(PeerControlMessage::StartPeerConnectionFromString(addr)).unwrap();
    }

    loop {
        println!("Checking for messages");
        let msg = msg_rx.recv().unwrap();
        println!("Got message back: {:?}", msg.command);

        match msg.command {
            Command::Addr(ref p) => {
                // We only take `max_extra_peers` addresses that we aren't currently connected to.
                for peer in p.addr_list.iter() {
                    peer_manager_channel.send(PeerControlMessage::StartPeerConnectionFromSocketAddr(peer.socket_addr())).unwrap();
                }
            },
            Command::Headers(ref p) => {
                // Confirming that the blocks are forming a chain.
                // TODO: Also confirm that their hash is below target.
                let mut headers_in_chain = true;
                let mut prev_hash = storage.latest_header.hash();

                for header in &p.headers {
                    if prev_hash != &header.prev_block {
                        println!("Message contains header which is not in the chain!\n");
                        println!("Latest hash: {}", byte_slice_as_hex(&prev_hash));
                        headers_in_chain = false;
                        break;
                    }

                    prev_hash = header.hash();
                }

                if headers_in_chain {
                    println!("All headers in chain. Writing them to storage!");
                    storage.write_headers(&p.headers).unwrap();
                }
            },
            _ => (),
        }
    }
}
