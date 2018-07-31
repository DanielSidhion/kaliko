use base58;
use byteorder::{BigEndian, ByteOrder};
use secp256k1::Error;

pub use self::extended_key::{ExtendedKey, ExtendedPublicKey};

pub mod extended_key;

#[cfg(test)]
mod tests;

pub const CHILD_INDEX_SIZE: usize = 4;
pub const FINGERPRINT_SIZE: usize = 4;
pub const CHAIN_CODE_SIZE: usize = 32;
pub const EXTENDED_KEY_SIZE: usize = 78;

#[derive(Debug)]
pub enum BIP32Error {
    InvalidSliceSize,
    InvalidNetworkType,
    InvalidPrivateKey,
    InvalidPublicKey,
    InvalidChildIndex,
    SeedGeneratesInvalidMasterKey,
    ChildIndexOverflow,
    InvalidEncodedByte,
    WrongChecksumInEncoding,
    ImpossibleToDeriveFromHardenedKey,
}

impl From<base58::Error> for BIP32Error {
    fn from(value: base58::Error) -> BIP32Error {
        match value {
            base58::Error::InvalidByte => BIP32Error::InvalidEncodedByte,
            base58::Error::WrongChecksum => BIP32Error::WrongChecksumInEncoding,
        }
    }
}

// Secp256k1 error.
impl From<Error> for BIP32Error {
    fn from(value: Error) -> BIP32Error {
        match value {
            Error::InvalidSecretKey => BIP32Error::InvalidPrivateKey,
            Error::InvalidPublicKey => BIP32Error::InvalidPublicKey,
            _ => panic!("Please convert more errors")
        }
    }
}

#[derive(Copy, Clone)]
pub enum ChildIndex {
    Normal(u32),
    Hardened(u32),
}

impl ChildIndex {
    pub fn normalize_index(&self) -> u32 {
        match *self {
            ChildIndex::Normal(i) => i,
            ChildIndex::Hardened(i) => i + (1 << 31),
        }
    }

    pub fn from_slice(slice: &[u8]) -> Result<ChildIndex, BIP32Error> {
        if slice.len() != CHILD_INDEX_SIZE {
            Err(BIP32Error::InvalidSliceSize)
        } else {
            Ok(BigEndian::read_u32(slice).into())
        }
    }

    pub fn next_index(&self) -> Result<ChildIndex, BIP32Error> {
        match *self {
            ChildIndex::Normal(i) if i < (1 << 31) - 1 => Ok(ChildIndex::Normal(i + 1)),
            ChildIndex::Hardened(i) if i < (1 << 31) - 1 => Ok(ChildIndex::Hardened(i + 1)),
            _ => Err(BIP32Error::ChildIndexOverflow),
        }
    }
}

impl From<u32> for ChildIndex {
    fn from(value: u32) -> Self {
        if value >= (1 << 31) {
            ChildIndex::Hardened(value - (1 << 31))
        } else {
            ChildIndex::Normal(value)
        }
    }
}