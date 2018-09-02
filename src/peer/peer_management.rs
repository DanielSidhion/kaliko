use ::KalikoControlMessage;
use bitcoin;
use network::Command;
use network::Message;
use peer::PeerConnection;
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time;

pub struct PeerManager {
    network: bitcoin::Network,
    max_active_peers: usize,
    max_potential_peers: usize,
    potential_peers: VecDeque<SocketAddr>,
    active_peers: HashMap<SocketAddr, Sender<KalikoControlMessage>>,
    connecting_peers: HashSet<SocketAddr>,
    incoming_control_sender: Sender<KalikoControlMessage>,
    incoming_control_receiver: Receiver<KalikoControlMessage>,
    outgoing_control_sender: Sender<KalikoControlMessage>,
}

impl PeerManager {
    pub fn new(network: bitcoin::Network, max_active_peers: usize, max_potential_peers: usize, outgoing_control_sender: Sender<KalikoControlMessage>) -> PeerManager {
        let (incoming_control_sender, incoming_control_receiver) = channel();

        PeerManager {
            network,
            max_active_peers,
            max_potential_peers,
            potential_peers: VecDeque::with_capacity(max_potential_peers),
            active_peers: HashMap::new(),
            connecting_peers: HashSet::new(),
            incoming_control_sender,
            incoming_control_receiver,
            outgoing_control_sender,
        }
    }

    pub fn control_sender(&self) -> Sender<KalikoControlMessage> {
        self.incoming_control_sender.clone()
    }

    fn handle_control_message(&mut self, msg: KalikoControlMessage) {
        match msg {
            KalikoControlMessage::NetworkMessage(Message {command: Command::Addr(p), ..}) => {
                // Add all addresses as potential peers to connect to.
                for addr in p.addr_list {
                    let socket_addr = addr.socket_addr();

                    if !self.potential_peers.contains(&socket_addr) && !self.active_peers.contains_key(&socket_addr) && !self.connecting_peers.contains(&socket_addr) {
                        self.potential_peers.push_back(socket_addr);
                    }
                }
            },
            KalikoControlMessage::StartPeerConnection(peer) => {
                if self.active_peers.contains_key(&peer) || self.connecting_peers.contains(&peer){
                    return;
                }

                if self.active_peers.len() + self.connecting_peers.len() >= self.max_active_peers {
                    // Add as a potential peer if we're already full. If any peer drops out, we can try potential peers.
                    if !self.potential_peers.contains(&peer) {
                        // Kick oldest entry if our potential peer queue is already full.
                        if self.potential_peers.len() >= self.max_potential_peers {
                            self.potential_peers.pop_front();
                        }

                        self.potential_peers.push_back(peer);
                    }
                    return;
                }

                // If the peer we received is in `self.potential_peers`, but we have the space to connect to more peers, we can just try to connect now and remove the peer from `self.potential_peers`.
                if self.potential_peers.contains(&peer) {
                    // This is just an operation to remove `peer` from `self.potential_peers`.
                    self.potential_peers.retain(|p| p != &peer);
                }

                self.connecting_peers.insert(peer);
                self.try_start_connection(peer);
            },
            KalikoControlMessage::PeerUnavailable(p) => {
                if self.connecting_peers.contains(&p) {
                    self.connecting_peers.remove(&p);
                }
            },
            KalikoControlMessage::PeerConnectionDestroyed(p) => {
                self.active_peers.remove(&p);
            },
            KalikoControlMessage::PeerConnectionEstablished(p, chan) => {
                if self.connecting_peers.contains(&p) {
                    self.connecting_peers.remove(&p);
                }

                self.active_peers.insert(p, chan);
            },
            KalikoControlMessage::PeerAnnouncedHeight(peer, height) => {
                self.outgoing_control_sender.send(KalikoControlMessage::PeerAnnouncedHeight(peer, height)).unwrap();
            },
            _ => (),
        }
    }

    pub fn start(mut self) {
        thread::spawn(move || {
            loop {
                match self.incoming_control_receiver.try_recv() {
                    Ok(msg) => {
                        debug!("Got control message: {:?}", msg);
                        self.handle_control_message(msg);
                    },
                    _ => {
                        // Check if we're at full capacity of connected peers. If not, try to connect to some more from potential peers.
                        // Notice that we only do this if we have no other message to receive from the channel. When we have too many messages to handle, the last priority is connecting to more peers (which will cause us to receive even more messages to handle).
                        // Also notice that if we take the current code as-is out of this place, we will have a logic bug because we don't take into account the amount of `StartPeerConnection` messages when calculating `num_new_peers`. This will cause us to remove a lot of peers from `self.potential_peers` only to have them put back into `self.potential_peers` when the messages are processed.
                        let mut num_new_peers = self.max_active_peers - (self.active_peers.len() + self.connecting_peers.len());
                        num_new_peers = ::std::cmp::min(self.potential_peers.len(), num_new_peers);
                        if num_new_peers > 0 {
                            info!("We're not at capacity of connected peers, so trying to connect to some more");
                            let peers_to_try = self.potential_peers.drain(..num_new_peers).collect::<Vec<SocketAddr>>();

                            for peer in peers_to_try {
                                trace!("Trying to connect to {}", peer);
                                self.incoming_control_sender.send(KalikoControlMessage::StartPeerConnection(peer)).unwrap();
                            }
                        }
                    },
                }

                thread::sleep(time::Duration::from_millis(10));
            }
        });
    }

    fn try_start_connection(&mut self, addr: SocketAddr) {
        let control_sender = self.incoming_control_sender.clone();
        let network = self.network;

        // Attempt the connection in a separate thread - this avoids blocking the peer manager from dealing with other messages.
        thread::spawn(move || {
            match PeerConnection::connect(network, addr, control_sender) {
                Ok(mut connection) => {
                    connection.handle_connection();
                },
                _ => (),
            }
        });
    }
}