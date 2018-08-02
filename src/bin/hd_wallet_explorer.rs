extern crate chalice;

use chalice::bip39;
use std::env;

fn byte_slice_as_hex(slice: &[u8]) -> String {
    let mut result = String::new();

    for byte in slice {
        result.push_str(&format!("{:02x}", byte));
    }

    result
}

fn main() {
    let mut args = env::args();
    args.next();

    let num_words = args.next().unwrap().parse::<usize>().unwrap();
    let mnemonic_words: Vec<String> = args.take(num_words).collect();
    let mnemonic_words = mnemonic_words.join(" ");

    let seed = bip39::MnemonicSeed::from_mnemonic(&mnemonic_words, bip39::WordList::English);
}
