extern crate env_logger;
extern crate kaliko;
#[macro_use] extern crate log;
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
        trace!("Finish config file parsing");

        let (main_control_sender, main_control_receiver) = mpsc::channel();

        // Storage communication set up.
        let storage = BlockHeaderStorage::new(&config.storage_location, main_control_sender.clone());
        let storage_channel = storage.incoming_sender();
        storage.start();
        trace!("Finish storage communication set up");

        // Peer manager communication set up.
        let peer_manager = peer::PeerManager::new(bitcoin::Network::Testnet3, config.max_active_peers, config.max_active_peers, main_control_sender.clone());
        let peer_manager_channel = peer_manager.control_sender();
        peer_manager.start();
        trace!("Finish peer manager communication set up");

        Kaliko {
            config,
            main_control_sender,
            main_control_receiver,
            storage_channel,
            peer_manager_channel,
        }
    }

    pub fn process_message(&self, msg: Message) {
        match msg.command {
            Command::Addr(p) => {
                // We only take `max_extra_peers` addresses that we aren't currently connected to.
                for peer in p.addr_list.iter() {
                    self.peer_manager_channel.send(KalikoControlMessage::StartPeerConnection(peer.socket_addr())).unwrap();
                }
            },
            Command::Headers(p) => {
                // Confirming that the blocks are forming a chain.
                // TODO: Also confirm that their hash is below target.
                // TODO: move this inside storage.
                // let mut headers_in_chain = true;
                // let mut prev_hash = self.storage.latest_header.hash();

                // for header in &p.headers {
                //     if prev_hash != &header.prev_block {
                //         debug!("Message contains header which is not in the chain!\n");
                //         debug!("Latest hash: {}", byte_slice_as_hex(&prev_hash));
                //         headers_in_chain = false;
                //         break;
                //     }

                //     prev_hash = header.hash();
                // }

                // if headers_in_chain {
                //     debug!("All headers in chain. Writing them to storage!");
                //     self.storage_channel.send(KalikoControlMessage::NewHeadersAvailable(p.headers)).unwrap();
                // }
            },
            _ => (),
        }
    }

    pub fn process_control_message(&self, msg: KalikoControlMessage) {
        match msg {
            KalikoControlMessage::NetworkMessage(msg) => {
                self.process_message(msg);
            },
            KalikoControlMessage::RequestHeaders(peer, latest_hash) => {
                // TODO: find a way to just route the message?
                self.peer_manager_channel.send(KalikoControlMessage::RequestHeaders(peer, latest_hash)).unwrap();
            },
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();
    info!("Starting Kaliko");

    let kaliko = Kaliko::new();

    debug!("Reading peer seed list");
    let mut initial_peers = peer::read_peer_list(&kaliko.config.peer_seed_list);
    while let Some(addr) = initial_peers.pop() {
        let addrs = match addr.to_socket_addrs() {
            Ok(result) => result.collect::<Vec<SocketAddr>>(),
            Err(_) => continue,
        };

        // TODO: improve this to prioritize e.g. IPv6 or something like that.
        for addr in addrs {
            trace!("Sending message to connect to {}", addr);
            kaliko.peer_manager_channel.send(KalikoControlMessage::StartPeerConnection(addr)).unwrap();
        }
    }

    loop {
        // if let Ok(msg) = kaliko.main_control_receiver.try_recv() {
        if let Ok(msg) = kaliko.main_control_receiver.recv() {
            trace!("Got control message: {:?}", msg);
            kaliko.process_control_message(msg);
        }
    }
}
