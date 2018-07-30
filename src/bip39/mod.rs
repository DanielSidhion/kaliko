use sha2::{Digest, Sha256};

pub const WORDLIST_SIZE: usize = 2048;
const ENGLISH_WORDLIST: &str = include_str!("english.txt");

pub enum BIP39Error {
    InvalidEntropyLength,
}

pub enum BIP39Entropy {
    Size128Bits([u8; 16]),
    Size160Bits([u8; 20]),
    Size192Bits([u8; 24]),
    Size224Bits([u8; 28]),
    Size256Bits([u8; 32]),
}

pub enum WordList {
    English,
}

impl WordList {
    pub fn as_slice(list: WordList) -> Vec<&'static str> {
        match list {
            WordList::English => ENGLISH_WORDLIST.lines().collect()
        }
    }
}

impl BIP39Entropy {
    pub fn from_slice(slice: &[u8]) -> Result<BIP39Entropy, BIP39Error> {
        match slice.len() {
            16 => {
                let entropy = [0u8; 16];
                entropy.copy_from_slice(&slice);
                Ok(BIP39Entropy::Size128Bits(entropy))
            },
            20 => {
                let entropy = [0u8; 20];
                entropy.copy_from_slice(&slice);
                Ok(BIP39Entropy::Size160Bits(entropy))
            },
            24 => {
                let entropy = [0u8; 24];
                entropy.copy_from_slice(&slice);
                Ok(BIP39Entropy::Size192Bits(entropy))
            },
            28 => {
                let entropy = [0u8; 28];
                entropy.copy_from_slice(&slice);
                Ok(BIP39Entropy::Size224Bits(entropy))
            },
            32 => {
                let entropy = [0u8; 32];
                entropy.copy_from_slice(&slice);
                Ok(BIP39Entropy::Size256Bits(entropy))
            },
            _ => Err(BIP39Error::InvalidEntropyLength),
        }
    }

    pub fn as_mnemonic(&self, wordlist: WordList) -> String {
        match *self {
            BIP39Entropy::Size128Bits(e) => {
                let checksum = Sha256::digest(&e[..4]).as_slice();
                let indices = [];
            }
        }
    }
}