extern crate kaliko;

use kaliko::bitcoin;
use kaliko::network::{Command, Message, NetworkError};
use kaliko::network::version::VersionPayload;
use std::io::prelude::*;
use std::net::{TcpStream};

fn byte_slice_as_hex(slice: &[u8]) -> String {
    let mut result = String::new();

    for byte in slice {
        result.push_str(&format!("{:02x}", byte));
    }

    result
}

fn version_handshake(connection: &mut TcpStream) -> i32 {
    let remote_version;

    let version = VersionPayload::new();
    let cmd = Command::Version(version);
    let msg = Message::new(bitcoin::Network::Testnet3, cmd);

    let payload: Vec<u8> = msg.into_iter().collect();
    connection.write(&payload).unwrap();

    let result_msg = Message::deserialize(connection).unwrap();
    match result_msg.command {
        Command::Version(p) => {
            remote_version = p.version();
        },
        _ => panic!("Expected version command"),
    }

    let result_msg = Message::deserialize(connection).unwrap();
    match result_msg.command {
        Command::Verack => {},
        _ => panic!("Expected verack command"),
    }

    // Send our verack as well.
    let msg = Message::new(bitcoin::Network::Testnet3, Command::Verack);
    let payload: Vec<u8> = msg.into_iter().collect();
    connection.write(&payload).unwrap();

    remote_version
}

fn main() {
    if let Ok(ref mut connection) = TcpStream::connect("185.28.76.179:18333") {
        println!("Connected! Sending version");

        let version = version_handshake(connection);
        println!("Version handshake complete! Remote's version is {}", version);

        loop {
            let msg = Message::deserialize(connection);
            if let Err(NetworkError::InvalidCommand(name)) = msg {
                println!("Received invalid command: {}", name);
                continue;
            }

            let msg = msg.unwrap();
            println!("Received command: {}", msg.command.name());
        }
    } else {
        println!("Connection failed");
    }
}
