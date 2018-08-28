use std::fs::File;
use std::io::Read;

pub mod peer_connection;
pub mod peer_management;

pub use self::peer_connection::PeerConnection;
pub use self::peer_management::PeerManager;

pub fn read_peer_list(peer_list_location: &str) -> Vec<String> {
    let mut peer_list = File::open(peer_list_location).unwrap();
    let mut peers = String::new();
    peer_list.read_to_string(&mut peers).unwrap();

    let mut result = vec![];

    for peer in peers.lines() {
        result.push(String::from(peer));
    }

    result
}