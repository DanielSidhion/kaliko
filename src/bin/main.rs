extern crate kaliko;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate toml;

use kaliko::network::Command;
use kaliko::network::headers::BlockHeader;
use kaliko::peer::PeerConnection;
use kaliko::storage::BlockHeaderStorage;
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

fn byte_slice_as_hex(slice: &[u8]) -> String {
    let mut result = String::new();
    for byte in slice {
        result.push_str(&format!("{:02x}", byte));
    }
    result
}

#[derive(Deserialize)]
struct Config {
    storage_location: String,
}

fn main() {
    let mut config_file = File::open("kaliko.toml").unwrap();
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).unwrap();

    let config: Config = toml::from_str(&contents).unwrap();
    println!("storage_location = {}", config.storage_location);

    // TODO: initialize storage with the location provided here, and start downloading block headers into it.
    let mut storage = BlockHeaderStorage::new(&config.storage_location);

    let (tx, rx) = mpsc::channel();

    //if let Ok(connection) = TcpStream::connect("94.130.14.223:18333") {
    //if let Ok(connection) = TcpStream::connect("54.71.51.214:18333") {
    //if let Ok(connection) = TcpStream::connect("13.250.203.1:18333") {
    //if let Ok(connection) = TcpStream::connect("13.125.190.124:18333") {
    // if let Ok(connection) = TcpStream::connect("82.94.216.148:18333") {
    // if let Ok(connection) = TcpStream::connect("94.130.14.223:18333") {
    // if let Ok(connection) = TcpStream::connect("13.57.48.134:18333") {
    if let Ok(connection) = TcpStream::connect("185.28.76.179:18333") {
        println!("Connected!");

        let tx = tx.clone();
        thread::spawn(move || {
            let mut peer_connection = PeerConnection::new(connection, tx);
            peer_connection.handle_connection();
        });
    } else {
        println!("Connection failed");
    }

    loop {
        let msg = rx.recv().unwrap();
        println!("Got message back: {:?}", msg.command);

        match msg.command {
            Command::Headers(ref p) => {
                // Confirming that the blocks are forming a chain.
                // TODO: Also confirm that their hash is below target.
                let mut headers_in_chain = true;
                let mut prev_hash = storage.latest_header.hash();

                for header in &p.headers {
                    if prev_hash != &header.prev_block {
                        println!("Message contains header which is not in the chain!\n");
                        println!("Latest hash: {}", byte_slice_as_hex(&prev_hash));
                        headers_in_chain = false;
                        break;
                    }

                    prev_hash = header.hash();
                }

                if headers_in_chain {
                    println!("All headers in chain. Writing them to storage!");
                    storage.write_headers(&p.headers).unwrap();
                }
            },
            _ => (),
        }
    }
}
