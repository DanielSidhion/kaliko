use bitcoin;
use network::{Command, Message, NetworkError};
use network::cmpct::SendCmpctPayload;
use network::version::VersionPayload;
use peer::PeerControlMessage;
use rand;
use rand::Rng;
use std::hash::{Hash, Hasher};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc::Sender;
use std::thread;

pub struct PeerConnection {
    stream: TcpStream,
    message_sender: Sender<Message>,
    control_sender: Sender<PeerControlMessage>,
    protocol_version: i32,
    fee_filter: u64,
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
    pub fn new(stream: TcpStream, message_sender: Sender<Message>, control_sender: Sender<PeerControlMessage>) -> PeerConnection {
        PeerConnection {
            stream,
            message_sender,
            control_sender,
            protocol_version: 0,
            fee_filter: 0,
        }
    }

    pub fn connect<A: ToSocketAddrs>(peer: A, message_sender: Sender<Message>, control_sender: Sender<PeerControlMessage>) -> Result<PeerConnection, ()> {
        if let Ok(connection) = TcpStream::connect(peer) {
            Ok(PeerConnection::new(connection, message_sender, control_sender))
        } else {
            Err(())
        }
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.stream.peer_addr().unwrap()
    }

    fn version_handshake(&mut self) {
        let version = VersionPayload::new(rand::thread_rng().next_u64());
        let cmd = Command::Version(version);
        let msg = Message::new(bitcoin::Network::Testnet3, cmd);
        msg.serialize(&mut self.stream).unwrap();

        let result_msg = Message::deserialize(&mut self.stream).unwrap();
        match result_msg.command {
            Command::Version(p) => {
                self.protocol_version = p.version();
            },
            _ => panic!("Expected version command"),
        }

        let result_msg = Message::deserialize(&mut self.stream).unwrap();
        match result_msg.command {
            Command::Verack => {},
            _ => panic!("Expected verack command"),
        }

        // Send our verack as well.
        let msg = Message::new(bitcoin::Network::Testnet3, Command::Verack);
        msg.serialize(&mut self.stream).unwrap();
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

    pub fn handle_connection(mut self) {
        thread::spawn(move || {
            self.version_handshake();
            println!("Version handshake complete! Remote's version is {}", self.protocol_version);

            // TODO: remove this and instead make it support other versions.
            if self.protocol_version != 70015 {
                println!("Because our peer's version is not 70015, we're ending the connection with them");
                return;
            }

            // TODO: match on connection closed errors and send a PeerConnectionDestroyed message.

            // self.send_parameter_messages();
            // println!("Finished sending all parameter messages!");

            // let msg = Message::new(bitcoin::Network::Testnet3, Command::GetBlocks(GetBlocksOrHeadersPayload::new()));
            // msg.serialize(&mut self.stream).unwrap();
            // println!("Sent getblocks command");

            // Sending fake getheaders message for first 4 blocks of the testnet3 blockchain.
            // let msg = Message::new(bitcoin::Network::Testnet3, Command::GetHeaders(GetBlocksOrHeadersPayload::new()));
            // msg.serialize(&mut self.stream).unwrap();
            // println!("Sent getblocks command");

            loop {
                let msg = Message::deserialize(&mut self.stream);
                if let Err(NetworkError::InvalidCommand(name)) = msg {
                    println!("Received invalid command: {}", name);
                    continue;
                }

                if let Err(NetworkError::PeerClosedConnection) = msg {
                    println!("Peer has closed connection to us, breaking out of loop");
                    break;
                }

                let msg = msg.unwrap();
                println!("Received command: {} with length {}", msg.command.name(), msg.command.length());

                // If it's something we can reply without sending to the receiver, do it here.
                match msg.command {
                    Command::Ping(nonce) => {
                        println!("Got ping: {}", nonce);
                        let pong = Message::new(bitcoin::Network::Testnet3, Command::Pong(nonce));
                        pong.serialize(&mut self.stream).unwrap();
                        continue;
                    },
                    Command::Pong(nonce) => {
                        println!("Got pong: {}", nonce);
                        continue;
                    },
                    Command::Feefilter(fee_filter) => {
                        self.fee_filter = fee_filter;
                        continue;
                    },
                    Command::SendCmpct(payload) => {
                        // TODO: set cmpct parameters here.
                        println!("Got cmpct: {:#?}", payload);
                        continue;
                    },
                    Command::SendHeaders => {
                        // TODO: Set headers parameters here.
                        continue;
                    },
                    _ => (),
                }

                self.message_sender.send(msg).unwrap();
            }
        });
    }
}