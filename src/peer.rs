use bitcoin;
use network::{Command, Message, NetworkError};
use network::version::VersionPayload;
use std::io::prelude::*;
use std::net::{TcpStream};
use std::sync::mpsc::Sender;

pub struct PeerConnection {
    stream: TcpStream,
    message_receiver: Sender<Message>,
    protocol_version: i32,
}

impl PeerConnection {
    pub fn new(stream: TcpStream, message_receiver: Sender<Message>) -> PeerConnection {
        PeerConnection {
            stream,
            message_receiver,
            protocol_version: 0,
        }
    }

    fn version_handshake(&mut self) {
        let version = VersionPayload::new();
        let cmd = Command::Version(version);
        let msg = Message::new(bitcoin::Network::Testnet3, cmd);

        let payload: Vec<u8> = msg.into_iter().collect();
        self.stream.write(&payload).unwrap();

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
        let payload: Vec<u8> = msg.into_iter().collect();
        self.stream.write(&payload).unwrap();
    }

    pub fn handle_connection(&mut self) -> ! {
        self.version_handshake();
        println!("Version handshake complete! Remote's version is {}", self.protocol_version);

        loop {
            let msg = Message::deserialize(&mut self.stream);
            if let Err(NetworkError::InvalidCommand(name)) = msg {
                println!("Received invalid command: {}", name);
                continue;
            }

            let msg = msg.unwrap();
            println!("Received command: {}", msg.command.name());
            self.message_receiver.send(msg).unwrap();
        }
    }
}