extern crate chalice;

use chalice::bip44::Wallet;
use std::env;

fn byte_slice_as_hex(slice: &[u8]) -> String {
    let mut result = String::new();

    for byte in slice {
        result.push_str(&format!("{:02x}", byte));
    }

    result
}

fn main() {
    // let secp_engine = Secp256k1::new();

    // let seed = Vec::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();

    // let mut mac = Hmac::<Sha512>::new_varkey(b"Bitcoin seed").unwrap();
    // mac.input(&seed);
    // let output = mac.result().code();

    // let priv_key = SecretKey::from_slice(&secp_engine, &output[..32]).unwrap();
    // let mut new_chain_code = [0u8; bip32::CHAIN_CODE_SIZE];
    // new_chain_code.copy_from_slice(&output[32..]);

    // let extended_key = bip32::ExtendedPrivateKey {
    //     network_type: bip32::ExtendedKeyNetworkType::Mainnet,
    //     depth: 0,
    //     parent_fingerprint: [0, 0, 0, 0],
    //     child_type: bip32::ExtendedKeyType::Normal(0),
    //     chain_code: new_chain_code,
    //     private_key: priv_key,
    // };

    // let first_data: &[u8] = &extended_key.serialize();

    // println!("Input data: {:#?}", byte_slice_as_hex(first_data));

    // let input_data = Sha256::digest(Sha256::digest(first_data).as_slice());

    // let input_data = first_data.iter().cloned().chain(input_data[0..4].iter().cloned());

    // println!("Extended key: {:#?}", base58::base58_encode(input_data));

    // let child_priv = extended_key.ckd_priv(&secp_engine, ExtendedKeyType::Hardened(0));

    // println!("Child priv: {:#?}", base58::base58_encode((&child_priv.serialize()).iter().cloned()));
}
