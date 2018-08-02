use ring::{digest, pbkdf2};
use sha2::{Digest, Sha256};

#[cfg(test)]
mod tests;

pub const BIP39_SEED_SIZE: usize = 64;
pub const WORDLIST_SIZE: usize = 2048;
const ENGLISH_WORDLIST: &str = include_str!("english.txt");

#[derive(Debug)]
pub enum BIP39Error {
    InvalidEntropyLength,
    WrongChecksum,
    InvalidWordInMnemonic,
    InvalidMnemonicSize,
}

#[derive(PartialEq, Eq, Debug)]
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
    pub fn from_entropy(slice: &[u8]) -> Result<MnemonicSeed, BIP39Error> {
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

    pub fn from_mnemonic(mnemonic: &str, wordlist: WordList) -> Result<MnemonicSeed, BIP39Error> {
        let wordlist = wordlist.as_vec();

        let mut entropy = Vec::<u8>::new();
        let mut available_index = 0;
        let mut curr_val = 0;
        let mut remaining_bits = 8;
        let mut available_bits;
        let mut total_words = 0;

        for word in mnemonic.split_whitespace() {
            total_words += 1;

            let word_index = match wordlist.binary_search(&word) {
                Ok(i) => i,
                Err(_) => return Err(BIP39Error::InvalidWordInMnemonic),
            };

            available_index = word_index as u16;
            available_bits = 11;

            while available_bits != 0 {
                if available_bits >= remaining_bits {
                    let mask = ((!0u8 >> (8 - remaining_bits)) as u16) << (available_bits - remaining_bits);
                    let val = (available_index & mask) >> (available_bits - remaining_bits);
                    available_index &= !mask;
                    curr_val |= val as u8;
                    entropy.push(curr_val);
                    curr_val = 0;
                    available_bits -= remaining_bits;
                    remaining_bits = 8;
                } else {
                    curr_val |= (available_index << (8 - available_bits)) as u8;
                    remaining_bits -= available_bits;
                    available_bits = 0;
                }
            }
        }

        match total_words {
            12 | 15 | 18 | 21 => {},
            // 24 words uses 8 bits as checksum. Our algorithm above would have put that as a u8 inside `entropy`, so we need to take it back.
            24 => {
                available_index = entropy.pop().unwrap() as u16;
            },
            _ => {
                return Err(BIP39Error::InvalidMnemonicSize);
            }
        }

        let checksum = Sha256::digest(entropy.as_slice());
        
        let checksum_bits = total_words / 3;
        let checksum_val = (checksum[0] >> (8 - checksum_bits)) as u16;

        if checksum_val != available_index {
            return Err(BIP39Error::WrongChecksum);
        }

        Ok(MnemonicSeed {
            entropy,
        })
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

    pub fn as_seed(&self, passphrase: &str) -> [u8; BIP39_SEED_SIZE] {
        let mut result = [0u8; BIP39_SEED_SIZE];

        let mnemonic_phrase = self.as_words(WordList::English);
        let salt = format!("mnemonic{}", passphrase);
        pbkdf2::derive(&digest::SHA512, 2048, &salt.as_bytes(), &mnemonic_phrase.as_bytes(), &mut result);

        result
    }
}