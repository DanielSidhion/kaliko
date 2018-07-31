use sha2::{Digest, Sha256};

#[cfg(test)]
mod tests;

pub const WORDLIST_SIZE: usize = 2048;
const ENGLISH_WORDLIST: &str = include_str!("english.txt");

#[derive(Debug)]
pub enum BIP39Error {
    InvalidEntropyLength,
    WrongChecksum,
}

pub struct MnemonicSeed {
    entropy: Vec<u8>,
}

pub enum WordList {
    English,
}

impl WordList {
    pub fn as_vec(&self) -> Vec<&'static str> {
        match *self {
            WordList::English => ENGLISH_WORDLIST.lines().collect()
        }
    }
}

impl MnemonicSeed {
    pub fn from_slice(slice: &[u8]) -> Result<MnemonicSeed, BIP39Error> {
        match slice.len() {
            16 | 20 | 24 | 28 | 32 => {
                let mut entropy = Vec::with_capacity(slice.len());
                entropy.extend(slice);

                Ok(MnemonicSeed {
                    entropy,
                })
            },
            _ => Err(BIP39Error::InvalidEntropyLength),
        }
    }

    pub fn from_mnemonic(mnemonic: &str) -> Result<MnemonicSeed, BIP39Error> {
        Err(BIP39Error::WrongChecksum)
    }

    pub fn as_words(&self, wordlist: WordList) -> String {
        let mut result = String::new();
        let wordlist = wordlist.as_vec();
        let wordlist_slice = wordlist.as_slice();
        let checksum = Sha256::digest(self.entropy.as_slice());
        let mut num_words = self.entropy.len() * 8 * 3 / 32;
        let mut entropy_it = self.entropy.iter();

        let mut curr_index = 0u16;
        let mut remaining_bits = 11;
        let mut curr_val;

        while num_words > 0 {
            while remaining_bits > 0 {
                curr_val = entropy_it.next();

                if curr_val.is_none() {
                    // Reached the end of the actual value. We need to add the remaining X bits from checksum now.
                    curr_index |= (checksum[0] as u16) >> (8 - remaining_bits);

                    result.push_str(wordlist_slice[curr_index as usize]);
                    num_words -= 1;

                    break;
                }

                let curr_val = curr_val.unwrap();

                if remaining_bits > 8 {
                    curr_index |= (*curr_val as u16) << (remaining_bits - 8);
                    remaining_bits -= 8;
                } else {
                    curr_index |= (*curr_val as u16) >> (8 - remaining_bits);

                    result.push_str(wordlist_slice[curr_index as usize]);
                    result.push(' ');
                    num_words -= 1;

                    if remaining_bits != 8 {
                        // Bootstraping next number.
                        curr_index = ((*curr_val << remaining_bits) as u16) << 3;
                        remaining_bits = 11 - (8 - remaining_bits);
                    } else {
                        curr_index = 0;
                        remaining_bits = 11;
                    }
                }
            }
        }

        result
    }
}