use base58::*;
use hex::{FromHex};

#[test]
fn decode_works() {
    assert_eq!(decode("1").ok(), Some(vec![0u8]));
    assert_eq!(decode("2").ok(), Some(vec![1u8]));
    assert_eq!(decode_check("1PfJpZsjreyVrqeoAfabrRwwjQyoSQMmHH").ok(), Some(Vec::from_hex("00f8917303bfa8ef24f292e8fa1419b20460ba064d").unwrap()));
}