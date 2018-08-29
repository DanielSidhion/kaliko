use ::KalikoControlMessage;
use bitcoin;
use network::Message;
use peer::PeerConnection;
use std::collections::{HashMap, VecDeque};
use std::net::{SocketAddr, SocketAddrV6, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};
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
    network: bitcoin::Network,
    max_active_peers: usize,
    potential_peers: VecDeque<SocketAddrV6>,
    active_peers: HashMap<SocketAddrV6, Sender<KalikoControlMessage>>,
    incoming_control_sender: Sender<KalikoControlMessage>,
    incoming_control_receiver: Receiver<KalikoControlMessage>,
    outgoing_control_sender: Sender<KalikoControlMessage>,
}

impl PeerManager {
    pub fn new(network: bitcoin::Network, max_active_peers: usize, outgoing_control_sender: Sender<KalikoControlMessage>) -> PeerManager {
        let (incoming_control_sender, incoming_control_receiver) = channel();

        PeerManager {
            network,
            max_active_peers,
            active_peers: HashMap::new(),
            potential_peers: VecDeque::new(),
            incoming_control_sender,
            incoming_control_receiver,
            outgoing_control_sender,
        }
    }

    pub fn control_sender(&self) -> Sender<KalikoControlMessage> {
        self.incoming_control_sender.clone()
    }

    pub fn start(mut self) {
        thread::spawn(move || {
            loop {
                let msg = self.incoming_control_receiver.recv().unwrap();

                match msg {
                    KalikoControlMessage::StartPeerConnectionFromString(p) => {
                        if self.active_peers.len() >= self.max_active_peers {
                            continue;
                        }

                        let mut addrs = match p.to_socket_addrs() {
                            Ok(result) => result.collect::<Vec<SocketAddr>>(),
                            Err(_) => continue,
                        };

                        // After resolving the peer address, if any of the resolved addresses are already active peers, just ignore everything.
                        if addrs.iter().any(|addr| self.active_peers.contains_key(&socket_addr_to_v6(*addr))) {
                            continue;
                        }

                        // TODO: deal with potential_peers here.

                        for addr in addrs {
                            if self.try_start_connection(addr) {
                                break;
                            }
                        }
                    },
                    KalikoControlMessage::StartPeerConnectionFromSocketAddr(p) => {
                        if self.active_peers.contains_key(&socket_addr_to_v6(p)) {
                            continue;
                        }

                        self.try_start_connection(p);
                    },
                    KalikoControlMessage::PeerConnectionDestroyed(p) => {
                        self.active_peers.remove(&socket_addr_to_v6(p));
                    },
                    KalikoControlMessage::PeerAnnouncedHeight(height) => {
                        self.outgoing_control_sender.send(KalikoControlMessage::PeerAnnouncedHeight(height)).unwrap();
                    },
                    _ => (),
                }
            }
        });
    }

    fn try_start_connection(&mut self, addr: SocketAddr) -> bool {
        let control_sender = self.incoming_control_sender.clone();

        match PeerConnection::connect(self.network, addr, control_sender) {
            Ok(connection) => {
                self.active_peers.insert(socket_addr_to_v6(connection.peer_addr()), connection.incoming_channel());
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