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

        connection.write(&payload[..]);
        let mut result = [0u8; 2048];
        let mut size = connection.read(&mut result).unwrap();

        while (size == 0) {
            size = connection.read(&mut result).unwrap();
        }

        println!("Result: {:?}", byte_slice_as_hex(&result[0..size]));
    } else {
        println!("Connection failed");
    }
}
