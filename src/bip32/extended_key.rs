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

        ret[0..4].copy_from_slice(&match self.network_type {
            Network::Mainnet => [0x04, 0x88, 0xAD, 0xE4],
            Network::Testnet => [0x04, 0x35, 0x83, 0x94],
        });

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

        let network_type = match slice[0..4] {
            [0x04, 0x88, 0xAD, 0xE4] => Network::Mainnet,
            [0x04, 0x35, 0x83, 0x94] => Network::Testnet,
            _ => return Err(BIP32Error::InvalidNetworkType),
        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use hex::FromHex;

    #[test]
    fn test_vector1() {
        let seed = Vec::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();

        let key = ExtendedKey::from_seed(&seed).unwrap();

        assert_eq!(key.to_string(),
            "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi");

        let hardened0 = key.ckd_priv(ChildIndex::Hardened(0)).unwrap();
        assert_eq!(hardened0.to_string(),
            "xprv9uHRZZhk6KAJC1avXpDAp4MDc3sQKNxDiPvvkX8Br5ngLNv1TxvUxt4cV1rGL5hj6KCesnDYUhd7oWgT11eZG7XnxHrnYeSvkzY7d2bhkJ7");

        let normal1 = hardened0.ckd_priv(ChildIndex::Normal(1)).unwrap();
        assert_eq!(normal1.to_string(),
            "xprv9wTYmMFdV23N2TdNG573QoEsfRrWKQgWeibmLntzniatZvR9BmLnvSxqu53Kw1UmYPxLgboyZQaXwTCg8MSY3H2EU4pWcQDnRnrVA1xe8fs");

        let hardened2 = normal1.ckd_priv(ChildIndex::Hardened(2)).unwrap();
        assert_eq!(hardened2.to_string(),
            "xprv9z4pot5VBttmtdRTWfWQmoH1taj2axGVzFqSb8C9xaxKymcFzXBDptWmT7FwuEzG3ryjH4ktypQSAewRiNMjANTtpgP4mLTj34bhnZX7UiM");

        let normal2 = hardened2.ckd_priv(ChildIndex::Normal(2)).unwrap();
        assert_eq!(normal2.to_string(),
            "xprvA2JDeKCSNNZky6uBCviVfJSKyQ1mDYahRjijr5idH2WwLsEd4Hsb2Tyh8RfQMuPh7f7RtyzTtdrbdqqsunu5Mm3wDvUAKRHSC34sJ7in334");

        let normal1000000000 = normal2.ckd_priv(ChildIndex::Normal(1000000000)).unwrap();
        assert_eq!(normal1000000000.to_string(),
            "xprvA41z7zogVVwxVSgdKUHDy1SKmdb533PjDz7J6N6mV6uS3ze1ai8FHa8kmHScGpWmj4WggLyQjgPie1rFSruoUihUZREPSL39UNdE3BBDu76");
    }

    #[test]
    fn test_vector2() {
        let seed = Vec::from_hex("fffcf9f6f3f0edeae7e4e1dedbd8d5d2cfccc9c6c3c0bdbab7b4b1aeaba8a5a29f9c999693908d8a8784817e7b7875726f6c696663605d5a5754514e4b484542").unwrap();

        let key = ExtendedKey::from_seed(&seed).unwrap();

        assert_eq!(key.to_string(),
            "xprv9s21ZrQH143K31xYSDQpPDxsXRTUcvj2iNHm5NUtrGiGG5e2DtALGdso3pGz6ssrdK4PFmM8NSpSBHNqPqm55Qn3LqFtT2emdEXVYsCzC2U");

        let normal0 = key.ckd_priv(ChildIndex::Normal(0)).unwrap();
        assert_eq!(normal0.to_string(),
            "xprv9vHkqa6EV4sPZHYqZznhT2NPtPCjKuDKGY38FBWLvgaDx45zo9WQRUT3dKYnjwih2yJD9mkrocEZXo1ex8G81dwSM1fwqWpWkeS3v86pgKt");

        let hardened2147483647 = normal0.ckd_priv(ChildIndex::Hardened(2147483647)).unwrap();
        assert_eq!(hardened2147483647.to_string(),
            "xprv9wSp6B7kry3Vj9m1zSnLvN3xH8RdsPP1Mh7fAaR7aRLcQMKTR2vidYEeEg2mUCTAwCd6vnxVrcjfy2kRgVsFawNzmjuHc2YmYRmagcEPdU9");

        let normal1 = hardened2147483647.ckd_priv(ChildIndex::Normal(1)).unwrap();
        assert_eq!(normal1.to_string(),
            "xprv9zFnWC6h2cLgpmSA46vutJzBcfJ8yaJGg8cX1e5StJh45BBciYTRXSd25UEPVuesF9yog62tGAQtHjXajPPdbRCHuWS6T8XA2ECKADdw4Ef");

        let hardened2147483646 = normal1.ckd_priv(ChildIndex::Hardened(2147483646)).unwrap();
        assert_eq!(hardened2147483646.to_string(),
            "xprvA1RpRA33e1JQ7ifknakTFpgNXPmW2YvmhqLQYMmrj4xJXXWYpDPS3xz7iAxn8L39njGVyuoseXzU6rcxFLJ8HFsTjSyQbLYnMpCqE2VbFWc");

        let normal2 = hardened2147483646.ckd_priv(ChildIndex::Normal(2)).unwrap();
        assert_eq!(normal2.to_string(),
            "xprvA2nrNbFZABcdryreWet9Ea4LvTJcGsqrMzxHx98MMrotbir7yrKCEXw7nadnHM8Dq38EGfSh6dqA9QWTyefMLEcBYJUuekgW4BYPJcr9E7j");
    }

    #[test]
    fn test_vector3() {
        let seed = Vec::from_hex("4b381541583be4423346c643850da4b320e46a87ae3d2a4e6da11eba819cd4acba45d239319ac14f863b8d5ab5a0d0c64d2e8a1e7d1457df2e5a3c51c73235be").unwrap();

        let key = ExtendedKey::from_seed(&seed).unwrap();

        assert_eq!(key.to_string(),
            "xprv9s21ZrQH143K25QhxbucbDDuQ4naNntJRi4KUfWT7xo4EKsHt2QJDu7KXp1A3u7Bi1j8ph3EGsZ9Xvz9dGuVrtHHs7pXeTzjuxBrCmmhgC6");

        let hardened0 = key.ckd_priv(ChildIndex::Hardened(0)).unwrap();
        assert_eq!(hardened0.to_string(),
            "xprv9uPDJpEQgRQfDcW7BkF7eTya6RPxXeJCqCJGHuCJ4GiRVLzkTXBAJMu2qaMWPrS7AANYqdq6vcBcBUdJCVVFceUvJFjaPdGZ2y9WACViL4L");
    }
}