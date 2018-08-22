extern crate kaliko;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate toml;

use kaliko::network::{Command, Message};
use kaliko::peer;
use kaliko::peer::{ControlMessage, PeerConnection};
use kaliko::storage::BlockHeaderStorage;
use std::collections::HashSet;
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

fn socket_addr_to_v6(socket: SocketAddr) -> SocketAddrV6 {
    match socket {
        SocketAddr::V6(p) => p,
        SocketAddr::V4(p) => {
            SocketAddrV6::new(p.ip().to_ipv6_mapped(), p.port(), 0, 0)
        },
    }
}

#[derive(Deserialize)]
struct Config {
    storage_location: String,
    peer_list: String,
    max_active_peers: u16,
}

fn try_peer_connection<A: ToSocketAddrs + Copy + Display>(address: A, msg_tx: Sender<Message>, control_tx: Sender<ControlMessage>) {
    // TODO: look into making this async.
    if let Ok(connection) = TcpStream::connect(address) {
        println!("Connected to {}!", address);

        thread::spawn(move || {
            let mut peer_connection = PeerConnection::new(connection, msg_tx, control_tx);
            peer_connection.handle_connection();
        });
    } else {
        println!("Connection failed to {}", address);
    }
}

fn main() {
    let mut config_file = File::open("kaliko.toml").unwrap();
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).unwrap();

    let config: Config = toml::from_str(&contents).unwrap();
    println!("storage_location = {}", config.storage_location);

    let mut storage = BlockHeaderStorage::new(&config.storage_location);

    let initial_peers = peer::read_peer_list(&config.peer_list);
    let current_peers = Arc::new(Mutex::new(HashSet::new()));
    let (control_tx, control_rx) = mpsc::channel();
    let (msg_tx, msg_rx) = mpsc::channel();

    for peer in initial_peers.iter() {
        let msg_tx = msg_tx.clone();
        let control_tx = control_tx.clone();
        try_peer_connection(peer, msg_tx, control_tx);
    }

    // Thread that listens for control messages from other threads.
    {
        let current_peers = Arc::clone(&current_peers);
        thread::spawn(move || {
            loop {
                let msg = control_rx.recv().unwrap();

                match msg {
                    ControlMessage::PeerConnectionEstablished(p) => {
                        let mut set = current_peers.lock().unwrap();
                        set.insert(socket_addr_to_v6(p));
                    },
                    ControlMessage::PeerConnectionDestroyed(p) => {
                        let mut set = current_peers.lock().unwrap();
                        set.remove(&socket_addr_to_v6(p));
                    },
                }
            }
        });
    }

    loop {
        let msg = msg_rx.recv().unwrap();
        println!("Got message back: {:?}", msg.command);

        match msg.command {
            Command::Addr(ref p) => {
                let mut set = current_peers.lock().unwrap();

                // Here we're calculating how many extra peers we can try connecting to, and then try those.
                // It's possible in the future we may need to just store the remaining peers in a list, that way we always have a backlog of peers to connect in case we lose connection to an active peer.
                let max_extra_peers = (config.max_active_peers as usize) - set.len();

                // We only take `max_extra_peers` addresses that we aren't currently connected to.
                for peer in p.addr_list.iter().filter(|addr| !set.contains(&addr.socket_addr())).take(max_extra_peers) {
                    let msg_tx = msg_tx.clone();
                    let control_tx = control_tx.clone();
                    try_peer_connection(peer.socket_addr(), msg_tx, control_tx);
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
