use ::KalikoControlMessage;
use bitcoin;
use byteorder::{ByteOrder, LittleEndian};
use network::{Command, Message, NetworkError};
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

        PeerConnection {
            network,
            stream,
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

    pub fn connect<A: ToSocketAddrs>(network: bitcoin::Network, peer: A, outgoing_control_sender: Sender<KalikoControlMessage>) -> Result<PeerConnection, ()> {
        if let Ok(connection) = TcpStream::connect(peer) {
            println!("Connection established");
            Ok(PeerConnection::new(network, connection, outgoing_control_sender))
        } else {
            Err(())
        }
    }

    pub fn incoming_channel(&self) -> Sender<KalikoControlMessage> {
        self.incoming_message_sender.clone()
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
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

    fn version_handshake(&mut self) -> bool {
        let version = VersionPayload::new(rand::thread_rng().next_u64());
        let cmd = Command::Version(version);
        let msg = Message::new(bitcoin::Network::Testnet3, cmd);
        msg.serialize(&mut self.stream).unwrap();

        // let result_msg = self.get_checked_message().unwrap();
        let result_msg = Message::deserialize(&mut self.stream).unwrap();
        match result_msg.command {
            Command::Version(p) => {
                self.protocol_version = p.version();
                self.peer_starting_height = p.start_height();
            },
            _ => panic!("Expected version command"),
        }

        // let result_msg = self.get_checked_message().unwrap();
        let result_msg = Message::deserialize(&mut self.stream).unwrap();
        match result_msg.command {
            Command::Verack => {},
            _ => panic!("Expected verack command"),
        }

        // Send our verack as well.
        let msg = Message::new(bitcoin::Network::Testnet3, Command::Verack);
        msg.serialize(&mut self.stream).unwrap();

        // TODO: remove this and instead make it support other versions.
        if self.protocol_version != 70015 {
            println!("Because our peer's version is not 70015, we're ending the connection with them");
            return false;
        }

        true
    }

    // To be used for sending certain meta commands to parameterize the communication between two peers only.
    fn send_parameter_messages(&mut self) {
        let msg = Message::new(bitcoin::Network::Testnet3, Command::SendHeaders);
        msg.serialize(&mut self.stream).unwrap();

        let cmpct = SendCmpctPayload::new();
        let msg = Message::new(bitcoin::Network::Testnet3, Command::SendCmpct(cmpct));
        msg.serialize(&mut self.stream).unwrap();

        let ping_nonce = rand::thread_rng().next_u64();
        println!("Ping nonce: {}", ping_nonce);
        let msg = Message::new(bitcoin::Network::Testnet3, Command::Ping(ping_nonce));
        msg.serialize(&mut self.stream).unwrap();

        let filter = 0x03e8;
        let msg = Message::new(bitcoin::Network::Testnet3, Command::Feefilter(filter));
        msg.serialize(&mut self.stream).unwrap();
    }

    fn handle_network_message(&mut self, msg: Message) {
        println!("Received command: {} with length {}", msg.command.name(), msg.command.length());

        // If it's something we can reply without sending to the receiver, do it here.
        match msg.command {
            Command::Ping(nonce) => {
                println!("Got ping: {}", nonce);
                let pong = Message::new(bitcoin::Network::Testnet3, Command::Pong(nonce));
                pong.serialize(&mut self.stream).unwrap();
                return;
            },
            Command::Pong(nonce) => {
                println!("Got pong: {}", nonce);
                return;
            },
            Command::Feefilter(fee_filter) => {
                self.fee_filter = fee_filter;
                return;
            },
            Command::SendCmpct(payload) => {
                // TODO: set cmpct parameters here.
                println!("Got cmpct: {:#?}", payload);
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

    }

    pub fn handle_connection(mut self) {
        thread::spawn(move || {
            if !self.version_handshake() {
                return;
            }

            println!("Version handshake complete! Remote's version is {}", self.protocol_version);
            self.outgoing_control_sender.send(KalikoControlMessage::PeerAnnouncedHeight(self.peer_starting_height)).unwrap();

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
                        println!("Received invalid command: {}", name);
                    },
                    Err(NetworkError::PeerClosedConnection) => {
                        println!("Peer has closed connection to us, breaking out of loop");
                        break;
                    },
                    Err(p) => {
                        println!("Got the following error: {:?}", p);
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
        });
    }
}