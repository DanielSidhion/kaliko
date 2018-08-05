extern crate chalice;

use chalice::bip39;
use chalice::bitcoin;
use chalice::network;
use std::io::prelude::*;
use std::env;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

fn byte_slice_as_hex(slice: &[u8]) -> String {
    let mut result = String::new();

    for byte in slice {
        result.push_str(&format!("{:02x}", byte));
    }

    result
}

fn main() {
    if let Ok(ref mut connection) = TcpStream::connect("185.28.76.179:18333") {
        println!("Connected! Sending version");

        let payload = network::VersionPayload::new();
        let cmd = network::Command::Version(payload);
        let msg = network::Message::new(bitcoin::Network::Testnet3, cmd);

        let payload: Vec<u8> = msg.into_iter().collect();
        println!("Payload: {:?}", byte_slice_as_hex(&payload[..]));

        connection.write(&payload);
        let result_msg = network::Message::deserialize(connection).unwrap();
        println!("Got message back: {:#?}", result_msg);

        let result_msg = network::Message::deserialize(connection).unwrap();
        if let network::Command::Verack = result_msg.command {
            let msg = network::Message::new(bitcoin::Network::Testnet3, network::Command::Verack);
            let payload: Vec<u8> = msg.into_iter().collect();
            connection.write(&payload);

            let msg = network::Message::new(bitcoin::Network::Testnet3, network::Command::Ping(0x0123456789ABCDEF));
            let payload: Vec<u8> = msg.into_iter().collect();
            connection.write(&payload);

            let mut result_msg = network::Message::deserialize(connection);

            while result_msg.is_err() {
                println!("Got some invalid command, trying to receive something else");
                result_msg = network::Message::deserialize(connection);
            }

            let result_msg = result_msg.unwrap();

            match result_msg.command {
                network::Command::Pong(n) => {
                    match n {
                        0x0123456789ABCDEF => println!("It works!"),
                        _ => println!("It almost works! Nonce is different"),
                    }
                },
                _ => {
                    println!("Boo");
                    println!("Received: {:#?}", result_msg);
                }
            }
        }
    } else {
        println!("Connection failed");
    }
}
