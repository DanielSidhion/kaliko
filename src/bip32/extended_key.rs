use base58;
use bip32::*;
use bitcoin::Network;

use byteorder::{BigEndian, ByteOrder};
use hmac::{Hmac, Mac};
use ripemd160::{Ripemd160};
use secp256k1::{ContextFlag, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256, Sha512};

use std::fmt;
use std::str;

trait BIP32Value {
    fn bip32_value(&self) -> [u8; 4];
    fn from_slice(slice: &[u8]) -> Result<Network, BIP32Error>;
}

impl BIP32Value for Network {
    fn bip32_value(&self) -> [u8; 4] {
        match *self {
            Network::Mainnet => [0x04, 0x88, 0xAD, 0xE4],
            Network::Testnet | Network::Testnet3 => [0x04, 0x35, 0x83, 0x94],
            Network::Namecoin => panic!("I don't know a BIP32 value for Namecoin!"),
        }
    }

    fn from_slice(slice: &[u8]) -> Result<Network, BIP32Error> {
        match slice {
            [0x04, 0x88, 0xAD, 0xE4] => Ok(Network::Mainnet),
            [0x04, 0x35, 0x83, 0x94] => Ok(Network::Testnet),
            _ => Err(BIP32Error::InvalidNetworkType),
        }
    }
}

pub struct ExtendedKey {
    network_type: Network,
    depth: u8,
    parent_fingerprint: [u8; FINGERPRINT_SIZE],
    child_index: ChildIndex,
    chain_code: [u8; CHAIN_CODE_SIZE],
    private_key: SecretKey,
    // Public key is also calculated and retained here for perf purposes. If you only need the extended public key chain, see ExtendedPublicKey.
    public_key: PublicKey,
}

pub struct ExtendedPublicKey {
    network_type: Network,
    depth: u8,
    parent_fingerprint: [u8; FINGERPRINT_SIZE],
    child_index: ChildIndex,
    chain_code: [u8; CHAIN_CODE_SIZE],
    public_key: PublicKey,
}

impl ExtendedKey {
    pub fn from_seed(seed: &[u8]) -> Result<ExtendedKey, BIP32Error> {
        let mut mac = Hmac::<Sha512>::new_varkey(b"Bitcoin seed").unwrap();
        mac.input(&seed);
        let output = mac.result().code();

        let secp = Secp256k1::with_caps(ContextFlag::SignOnly);

        let priv_key = match SecretKey::from_slice(&secp, &output[..32]) {
            Ok(val) => val,
            Err(_) => return Err(BIP32Error::SeedGeneratesInvalidMasterKey),
        };

        let mut new_chain_code = [0; CHAIN_CODE_SIZE];
        new_chain_code.copy_from_slice(&output[32..]);

        Ok(ExtendedKey {
            network_type: Network::Mainnet,
            depth: 0,
            parent_fingerprint: [0, 0, 0, 0],
            child_index: ChildIndex::Normal(0),
            chain_code: new_chain_code,
            private_key: priv_key,
            public_key: PublicKey::from_secret_key(&secp, &priv_key)?,
        })
    }

    pub fn serialize(&self) -> [u8; EXTENDED_KEY_SIZE] {
        let mut ret = [0; EXTENDED_KEY_SIZE];

        ret[0..4].copy_from_slice(&self.network_type.bip32_value());

        ret[4] = self.depth;

        ret[5..9].copy_from_slice(&self.parent_fingerprint);
        BigEndian::write_u32(&mut ret[9..13], self.child_index.normalize_index());
        ret[13..45].copy_from_slice(&self.chain_code);
        ret[45] = 0;
        ret[46..78].copy_from_slice(&self.private_key[..]);

        ret
    }

    pub fn from_slice(slice: &[u8]) -> Result<ExtendedKey, BIP32Error> {
        if slice.len() != EXTENDED_KEY_SIZE {
            return Err(BIP32Error::InvalidSliceSize)
        }

        // For private keys this value should always be 0.
        if slice[45] != 0 {
            return Err(BIP32Error::InvalidPrivateKey)
        }

        let network_type = Network::from_slice(&slice[0..4])?;

        let depth = slice[4];
        let mut parent_fingerprint = [0; FINGERPRINT_SIZE];
        parent_fingerprint.copy_from_slice(&slice[5..9]);

        // TODO: return error if depth == 0 and parent_fingerprint != 0.

        let child_index = ChildIndex::from_slice(&slice[9..13])?;
        let mut chain_code = [0; CHAIN_CODE_SIZE];
        chain_code.copy_from_slice(&slice[13..45]);

        let secp = Secp256k1::without_caps();
        let private_key = SecretKey::from_slice(&secp, &slice[46..78])?;
        let public_key = PublicKey::from_secret_key(&secp, &private_key)?;

        Ok(ExtendedKey {
            network_type,
            depth,
            parent_fingerprint,
            child_index,
            chain_code,
            private_key,
            public_key,
        })
    }

    pub fn fingerprint(&self) -> [u8; FINGERPRINT_SIZE] {
        let digest = Ripemd160::digest(Sha256::digest(&self.public_key.serialize()).as_slice());

        let mut result = [0; FINGERPRINT_SIZE];
        result.copy_from_slice(&digest[..4]);

        result
    }

    pub fn ckd_pub(&self, child_index: ChildIndex) -> Result<ExtendedPublicKey, BIP32Error> {
        if let ChildIndex::Hardened(_) = child_index {
            return Err(BIP32Error::ImpossibleToDeriveFromHardenedKey)
        }

        // The ckd_pub computation implicitly calculates the new child private key, so there's no harm doing this.
        let child_private = self.ckd_priv(child_index)?;

        Ok(ExtendedPublicKey {
            network_type: child_private.network_type,
            depth: child_private.depth,
            parent_fingerprint: child_private.parent_fingerprint,
            child_index,
            chain_code: child_private.chain_code,
            public_key: child_private.public_key,
        })
    }

    pub fn ckd_priv(&self, child_index: ChildIndex) -> Result<ExtendedKey, BIP32Error> {
        // TODO: remove copy trait from ChildIndex and fix the need for a copy in this function body.

        let mut result = self.ckd_priv_internal(child_index);

        while result.is_err() {
            let new_index = child_index.next_index()?;
            result = self.ckd_priv_internal(new_index);
        }

        result
    }

    fn ckd_priv_internal(&self, child_index: ChildIndex) -> Result<ExtendedKey, BIP32Error> {
        let mut mac = Hmac::<Sha512>::new_varkey(&self.chain_code).unwrap();

        match child_index {
            ChildIndex::Normal(i) if i < (1 << 31) => {
                mac.input(&self.public_key.serialize());
            },
            ChildIndex::Hardened(i) if i < (1 << 31) => {
                mac.input(&[0u8]);
                mac.input(&self.private_key[..]);
            },
            _ => {
                return Err(BIP32Error::InvalidChildIndex)
            },
        }

        let mut serialized_i = [0; CHILD_INDEX_SIZE];
        BigEndian::write_u32(&mut serialized_i, child_index.normalize_index());
        mac.input(&serialized_i);
        let output = mac.result().code();

        let secp = Secp256k1::with_caps(ContextFlag::SignOnly);
        let mut new_private_key = SecretKey::from_slice(&secp, &output[..32])?;
        new_private_key.add_assign(&secp, &self.private_key)?;

        let mut new_chain_code = [0u8; CHAIN_CODE_SIZE];
        new_chain_code.copy_from_slice(&output[32..]);

        Ok(ExtendedKey {
            network_type: Network::Mainnet,
            depth: self.depth + 1,
            parent_fingerprint: self.fingerprint(),
            child_index,
            chain_code: new_chain_code,
            private_key: new_private_key,
            public_key: PublicKey::from_secret_key(&secp, &new_private_key)?,
        })
    }
}

impl ExtendedPublicKey {

}

impl fmt::Display for ExtendedKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", base58::check_encode(&self.serialize()))
    }
}

impl str::FromStr for ExtendedKey {
    type Err = BIP32Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded_data = base58::decode_check(s)?;

        ExtendedKey::from_slice(&decoded_data)
    }
}