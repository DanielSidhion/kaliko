extern crate kaliko;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate toml;

use kaliko::KalikoControlMessage;
use kaliko::bitcoin;
use kaliko::network::{Command, Message};
use kaliko::peer;
use kaliko::peer::PeerConnection;
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

pub struct Kaliko {
    config: Config,
    main_control_sender: mpsc::Sender<KalikoControlMessage>,
    main_control_receiver: mpsc::Receiver<KalikoControlMessage>,
    storage: BlockHeaderStorage,
    storage_channel: mpsc::Sender<KalikoControlMessage>,
    peer_manager_channel: mpsc::Sender<KalikoControlMessage>,
}

impl Kaliko {
    pub fn new() -> Kaliko {
        // Config file parsing.
        let mut config_file = File::open("kaliko.toml").unwrap();
        let mut contents = String::new();
        config_file.read_to_string(&mut contents).unwrap();
        let config: Config = toml::from_str(&contents).unwrap();

        let (main_control_sender, main_control_receiver) = mpsc::channel();

        // Storage communication set up.
        let storage = BlockHeaderStorage::new(&config.storage_location, main_control_sender.clone());
        let storage_channel = storage.incoming_sender();

        // Peer manager communication set up.
        let peer_manager = peer::PeerManager::new(bitcoin::Network::Testnet3, config.max_active_peers, main_control_sender.clone());
        let peer_manager_channel = peer_manager.control_sender();
        peer_manager.start();

        Kaliko {
            config,
            main_control_sender,
            main_control_receiver,
            storage,
            storage_channel,
            peer_manager_channel,
        }
    }

    pub fn process_message(&self, msg: Message) {
        match msg.command {
            Command::Addr(p) => {
                // We only take `max_extra_peers` addresses that we aren't currently connected to.
                for peer in p.addr_list.iter() {
                    self.peer_manager_channel.send(KalikoControlMessage::StartPeerConnectionFromSocketAddr(peer.socket_addr())).unwrap();
                }
            },
            Command::Headers(p) => {
                // Confirming that the blocks are forming a chain.
                // TODO: Also confirm that their hash is below target.
                // TODO: move this inside storage.
                let mut headers_in_chain = true;
                let mut prev_hash = self.storage.latest_header.hash();

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
                    self.storage_channel.send(KalikoControlMessage::NewHeadersAvailable(p.headers)).unwrap();
                }
            },
            _ => (),
        }
    }

    pub fn process_control_message(&self, msg: KalikoControlMessage) {
        match msg {
            KalikoControlMessage::NetworkMessage(msg) => {
                self.process_message(msg);
            },
            KalikoControlMessage::RequestHeaders(latest_hash) => {
                // TODO: find a way to just route the message?
                self.peer_manager_channel.send(KalikoControlMessage::RequestHeaders(latest_hash)).unwrap();
            },
            _ => (),
        }
    }
}

fn main() {
    let kaliko = Kaliko::new();

    let mut initial_peers = peer::read_peer_list(&kaliko.config.peer_seed_list);
    while let Some(addr) = initial_peers.pop() {
        println!("Sending message to connect to {}", addr);
        kaliko.peer_manager_channel.send(KalikoControlMessage::StartPeerConnectionFromString(addr)).unwrap();
    }

    loop {
        // if let Ok(msg) = kaliko.main_control_receiver.try_recv() {
        if let Ok(msg) = kaliko.main_control_receiver.recv() {
            println!("Got control message: {:?}", msg);
            kaliko.process_control_message(msg);
        }
    }
}
