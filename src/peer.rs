use bitcoin;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use network::{Command, Message, NetworkError};
use network::cmpct::SendCmpctPayload;
use network::version::VersionPayload;
use rand;
use rand::Rng;
use std::io::prelude::*;
use std::net::{TcpStream};
use std::sync::mpsc::Sender;

pub struct PeerConnection {
    stream: TcpStream,
    message_receiver: Sender<Message>,
    protocol_version: i32,
    random_generator: rand::ThreadRng,
    fee_filter: u64,
}

impl PeerConnection {
    pub fn new(stream: TcpStream, message_receiver: Sender<Message>) -> PeerConnection {
        PeerConnection {
            stream,
            message_receiver,
            protocol_version: 0,
            random_generator: rand::thread_rng(),
            fee_filter: 0,
        }
    }

    fn version_handshake(&mut self) {
        let version = VersionPayload::new(self.random_generator.next_u64());
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

        let ping_nonce = self.random_generator.next_u64();
        println!("Ping nonce: {}", ping_nonce);
        let msg = Message::new(bitcoin::Network::Testnet3, Command::Ping(ping_nonce));
        msg.serialize(&mut self.stream).unwrap();

        let filter = 0x03e8;
        let msg = Message::new(bitcoin::Network::Testnet3, Command::Feefilter(filter));
        msg.serialize(&mut self.stream).unwrap();
    }

    pub fn handle_connection(&mut self) {
        self.version_handshake();
        println!("Version handshake complete! Remote's version is {}", self.protocol_version);

        // self.send_parameter_messages();
        // println!("Finished sending all parameter messages!");

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

            self.message_receiver.send(msg).unwrap();
        }
    }
}