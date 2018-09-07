use ::KalikoControlMessage;
use bitcoin;
use byteorder::{ByteOrder, LittleEndian};
use network::{Command, Message, NetworkError};
use network::blocks::GetBlocksOrHeadersPayload;
use network::cmpct::SendCmpctPayload;
use network::version::VersionPayload;
use rand;
use rand::Rng;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};

pub struct PeerConnection {
    network: bitcoin::Network,
    stream: TcpStream,
    peer_addr: SocketAddr,
    protocol_version: i32,
    fee_filter: u64,
    peer_starting_height: i32,
    message_buffer: Vec<u8>,
    outgoing_control_sender: Sender<KalikoControlMessage>,
    incoming_message_sender: Sender<KalikoControlMessage>,
    incoming_message_receiver: Receiver<KalikoControlMessage>,
}

impl PartialEq for PeerConnection {
    fn eq(&self, other: &PeerConnection) -> bool {
        self.stream.peer_addr().unwrap() == other.stream.peer_addr().unwrap()
    }
}

impl Eq for PeerConnection {}

impl Hash for PeerConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stream.peer_addr().unwrap().hash(state);
    }
}

impl PeerConnection {
    pub fn new(network: bitcoin::Network, stream: TcpStream, outgoing_control_sender: Sender<KalikoControlMessage>) -> PeerConnection {
        let (incoming_message_sender, incoming_message_receiver) = mpsc::channel();
        let peer_addr = stream.peer_addr().unwrap();

        PeerConnection {
            network,
            stream,
            peer_addr,
            protocol_version: 0,
            fee_filter: 0,
            peer_starting_height: 0,
            // TODO: possibly make this size configurable.
            message_buffer: Vec::with_capacity(4096),
            outgoing_control_sender,
            incoming_message_sender,
            incoming_message_receiver,
        }
    }

    pub fn connect(network: bitcoin::Network, peer: SocketAddr, outgoing_control_sender: Sender<KalikoControlMessage>) -> Result<PeerConnection, ()> {
        debug!("[{}] Attempting connection", peer);

        if let Ok(connection) = TcpStream::connect(peer) {
            debug!("[{}] Connection established", peer);
            Ok(PeerConnection::new(network, connection, outgoing_control_sender))
        } else {
            outgoing_control_sender.send(KalikoControlMessage::PeerUnavailable(peer)).unwrap();
            Err(())
        }
    }

    pub fn incoming_channel(&self) -> Sender<KalikoControlMessage> {
        self.incoming_message_sender.clone()
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    fn get_checked_message(&mut self) -> Result<Message, NetworkError> {
        // TODO: don't ignore error here.
        self.stream.read_to_end(&mut self.message_buffer);

        if self.message_buffer.len() < 24 {
            // We don't even have the message header received.
            return Err(NetworkError::NotEnoughData);
        }

        // Check if we have a complete message in the buffer.
        let message_length = 24 + LittleEndian::read_u32(&self.message_buffer[16..20]) as usize;

        if self.message_buffer.len() < message_length {
            // We haven't yet received the full payload.
            return Err(NetworkError::NotEnoughData);
        }

        let full_message_bytes = self.message_buffer.drain(0..message_length).collect::<Vec<u8>>();
        let msg = Message::deserialize(&mut &full_message_bytes[..]).unwrap();
        if msg.network != self.network {
            // TODO: shutdown this connection or do something else.
            return Err(NetworkError::WrongNetwork)
        }

        Ok(msg)
    }

    fn version_handshake(&mut self) -> Result<bool, NetworkError> {
        let version = VersionPayload::new(rand::thread_rng().next_u64());
        let cmd = Command::Version(version);
        let msg = Message::new(bitcoin::Network::Testnet3, cmd);
        msg.serialize(&mut self.stream)?;

        let result_msg = Message::deserialize(&mut self.stream)?;
        match result_msg.command {
            Command::Version(p) => {
                self.protocol_version = p.version();
                self.peer_starting_height = p.start_height();
            },
            _ => panic!("Expected version command"),
        }

        let result_msg = Message::deserialize(&mut self.stream)?;
        match result_msg.command {
            Command::Verack => {},
            _ => panic!("Expected verack command"),
        }

        // Send our verack as well.
        let msg = Message::new(bitcoin::Network::Testnet3, Command::Verack);
        msg.serialize(&mut self.stream)?;

        // TODO: remove this and instead make it support other versions.
        if self.protocol_version != 70015 {
            info!("[{}] Because our peer's version is not 70015, we're ending the connection with them", self.peer_addr());
            Ok(false)
        } else {
            Ok(true)
        }
    }

    // To be used for sending certain meta commands to parameterize the communication between two peers only.
    fn send_parameter_messages(&mut self) {
        let msg = Message::new(bitcoin::Network::Testnet3, Command::SendHeaders);
        msg.serialize(&mut self.stream).unwrap();

        let cmpct = SendCmpctPayload::new();
        let msg = Message::new(bitcoin::Network::Testnet3, Command::SendCmpct(cmpct));
        msg.serialize(&mut self.stream).unwrap();

        let ping_nonce = rand::thread_rng().next_u64();
        let msg = Message::new(bitcoin::Network::Testnet3, Command::Ping(ping_nonce));
        msg.serialize(&mut self.stream).unwrap();

        let filter = 0x03e8;
        let msg = Message::new(bitcoin::Network::Testnet3, Command::Feefilter(filter));
        msg.serialize(&mut self.stream).unwrap();
    }

    fn handle_network_message(&mut self, msg: Message) {
        debug!("[{}] Received command: {} with length {}", self.peer_addr(), msg.command.name(), msg.command.length());
        // TODO: validate any message received. If it's not valid, either ignore or close connection.

        // If it's something we can reply without sending to the receiver, do it here.
        match msg.command {
            Command::Ping(nonce) => {
                let pong = Message::new(bitcoin::Network::Testnet3, Command::Pong(nonce));
                pong.serialize(&mut self.stream).unwrap();
                return;
            },
            Command::Pong(nonce) => {
                return;
            },
            Command::Feefilter(fee_filter) => {
                self.fee_filter = fee_filter;
                return;
            },
            Command::SendCmpct(payload) => {
                // TODO: set cmpct parameters here.
                return;
            },
            Command::SendHeaders => {
                // TODO: Set headers parameters here.
                return;
            },
            _ => (),
        }

        self.outgoing_control_sender.send(KalikoControlMessage::NetworkMessage(msg)).unwrap();
    }

    fn handle_control_message(&mut self, msg: KalikoControlMessage) {
        debug!("[{}] Received control command: {:?}", self.peer_addr(), msg);

        match msg {
            KalikoControlMessage::RequestHeaders(locator) => {
                let msg = Message::new(bitcoin::Network::Testnet3, Command::GetHeaders(GetBlocksOrHeadersPayload::new()));
                msg.serialize(&mut self.stream).unwrap();
            },
            _ => (),
        }
    }

    pub fn handle_connection(&mut self) {
        match self.version_handshake() {
            Ok(false) | Err(_) => {
                self.outgoing_control_sender.send(KalikoControlMessage::PeerUnavailable(self.peer_addr())).unwrap();
                return;
            },
            _ => (),
        }

        info!("[{}] Version handshake complete! Remote's version is {}", self.peer_addr(), self.protocol_version);
        self.outgoing_control_sender.send(KalikoControlMessage::PeerConnectionEstablished(self.peer_addr(), self.incoming_channel())).unwrap();
        self.outgoing_control_sender.send(KalikoControlMessage::PeerAnnouncedHeight(self.peer_addr(), self.peer_starting_height)).unwrap();

        // self.send_parameter_messages();
        // println!("Finished sending all parameter messages!");

        // let msg = Message::new(bitcoin::Network::Testnet3, Command::GetBlocks(GetBlocksOrHeadersPayload::new()));
        // msg.serialize(&mut self.stream).unwrap();
        // println!("Sent getblocks command");

        // Sending fake getheaders message for first 4 blocks of the testnet3 blockchain.
        // let msg = Message::new(bitcoin::Network::Testnet3, Command::GetHeaders(GetBlocksOrHeadersPayload::new()));
        // msg.serialize(&mut self.stream).unwrap();
        // println!("Sent getblocks command");

        // Set the stream as nonblocking since we'll enter a loop to check for messages from it and from another channel.
        self.stream.set_nonblocking(true).unwrap();

        // TODO: match on connection closed errors and send a PeerConnectionDestroyed message.
        loop {
            match self.get_checked_message() {
                Err(NetworkError::NotEnoughData) => (),
                Err(NetworkError::InvalidCommand(name)) => {
                    debug!("[{}] Received invalid command: {}", self.peer_addr(), name);
                },
                Err(NetworkError::PeerClosedConnection) => {
                    debug!("[{}] Peer has closed connection to us, breaking out of loop", self.peer_addr());
                    break;
                },
                Err(p) => {
                    debug!("[{}] Got the following error: {:?}", self.peer_addr(), p);
                    panic!("Got unexpected error");
                }
                Ok(msg) => {
                    self.handle_network_message(msg);
                },
            };

            match self.incoming_message_receiver.try_recv() {
                Ok(msg) => self.handle_control_message(msg),
                _ => (),
            };

            thread::sleep(time::Duration::from_millis(10));
        }

        self.outgoing_control_sender.send(KalikoControlMessage::PeerConnectionDestroyed(self.peer_addr())).unwrap();
    }
}