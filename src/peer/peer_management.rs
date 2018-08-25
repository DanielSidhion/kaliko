use network::Message;
use peer::{PeerConnection, PeerControlMessage};
use std::collections::{HashSet, VecDeque};
use std::net::{SocketAddr, SocketAddrV6, ToSocketAddrs};
use std::sync::{mpsc};
use std::thread;

fn socket_addr_to_v6(socket: SocketAddr) -> SocketAddrV6 {
    match socket {
        SocketAddr::V6(p) => p,
        SocketAddr::V4(p) => {
            SocketAddrV6::new(p.ip().to_ipv6_mapped(), p.port(), 0, 0)
        },
    }
}

pub struct PeerManager {
    max_active_peers: usize,
    active_peers: HashSet<SocketAddrV6>,
    potential_peers: VecDeque<SocketAddrV6>,
    control_sender: mpsc::Sender<PeerControlMessage>,
    control_receiver: mpsc::Receiver<PeerControlMessage>,
    message_sender: mpsc::Sender<Message>,
}

impl PeerManager {
    pub fn new(message_sender: mpsc::Sender<Message>, max_active_peers: usize) -> PeerManager {
        let (control_sender, control_receiver) = mpsc::channel();

        PeerManager {
            max_active_peers,
            active_peers: HashSet::new(),
            potential_peers: VecDeque::new(),
            control_sender,
            control_receiver,
            message_sender,
        }
    }

    pub fn control_sender(&self) -> mpsc::Sender<PeerControlMessage> {
        self.control_sender.clone()
    }

    pub fn start(mut self) {
        thread::spawn(move || {
            loop {
                let msg = self.control_receiver.recv().unwrap();

                match msg {
                    PeerControlMessage::StartPeerConnectionFromString(p) => {
                        println!("Trying to connect to {}", p);

                        let mut addrs = match p.to_socket_addrs() {
                            Ok(result) => result.collect::<Vec<SocketAddr>>(),
                            Err(_) => continue,
                        };

                        println!("All the addresses that it maps to: {:?}", addrs);

                        // After resolving the peer address, if any of the resolved addresses are already active peers, just ignore everything.
                        if addrs.iter().any(|addr| self.active_peers.contains(&socket_addr_to_v6(*addr))) {
                            continue;
                        }

                        // TODO: deal with potential_peers here.

                        for addr in addrs {
                            if self.try_start_connection(addr) {
                                break;
                            }
                        }
                    },
                    PeerControlMessage::StartPeerConnectionFromSocketAddr(p) => {
                        if self.active_peers.contains(&socket_addr_to_v6(p)) {
                            continue;
                        }

                        self.try_start_connection(p);
                    },
                    PeerControlMessage::PeerConnectionDestroyed(p) => {
                        self.active_peers.remove(&socket_addr_to_v6(p));
                    },
                }
            }
        });
    }

    fn try_start_connection(&mut self, addr: SocketAddr) -> bool {
        let message_sender = self.message_sender.clone();
        let control_sender = self.control_sender.clone();

        match PeerConnection::connect(addr, message_sender, control_sender) {
            Ok(connection) => {
                println!("Connected!");
                self.active_peers.insert(socket_addr_to_v6(connection.peer_addr()));
                connection.handle_connection();
                true
            },
            Err(_) => {
                // TODO: increase retry count.
                false
            },
        }
    }
}