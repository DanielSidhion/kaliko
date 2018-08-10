extern crate kaliko;

use kaliko::peer::PeerConnection;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

// fn byte_slice_as_hex(slice: &[u8]) -> String {
//     let mut result = String::new();

//     for byte in slice {
//         result.push_str(&format!("{:02x}", byte));
//     }

//     result
// }

fn main() {
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
    }
}
