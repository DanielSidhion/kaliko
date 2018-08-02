use bip32::{ChildIndex, ExtendedKey};
use bip39::{MnemonicSeed, WordList};

pub enum BIP43Purpose {
    BIP44Wallet,
}

impl BIP43Purpose {
    pub fn value(&self) -> u32 {
        match *self {
            BIP43Purpose::BIP44Wallet => 44,
        }
    }
}

pub enum BIP44Error {
    ImpossibleToDeriveBranch,
}

pub enum CoinType {
    Bitcoin,
    Testnet,
    Litecoin,
    Namecoin,
}

impl CoinType {
    pub fn value(&self) -> u32 {
        match *self {
            CoinType::Bitcoin => 0x80000000,
            CoinType::Testnet => 0x80000001,
            CoinType::Litecoin => 0x80000002,
            CoinType::Namecoin => 0x80000007,
        }
    }
}

pub struct MasterWallet {
    key: ExtendedKey,
}

pub struct PurposeWallet {
    key: ExtendedKey,
    purpose: BIP43Purpose,
}

pub struct CoinWallet {
    key: ExtendedKey,
    coin_type: CoinType,
}

pub struct AccountWallet {
    key: ExtendedKey,
    account: u32,
}

pub struct ChainWallet {
    key: ExtendedKey,
    chain: u32,
}

pub struct AddressWallet {
    key: ExtendedKey,
    index: u32,
}

impl MasterWallet {
    pub fn from_mnemonic(mnemonic: &str, passphrase: &str, wordlist: WordList) -> MasterWallet {
        let seed = MnemonicSeed::from_mnemonic(mnemonic, wordlist).unwrap();
        let seed = seed.as_seed(passphrase);

        let key = ExtendedKey::from_seed(&seed).unwrap();

        MasterWallet {
            key
        }
    }

    pub fn for_purpose(&self, purpose: BIP43Purpose) -> PurposeWallet {
        PurposeWallet {
            // The unwrap here is guaranteed to always work because values from a BIP43Purpose won't cause indexing problems when generating a child.
            // Actually, there's a very unlikely scenario of the value being close to the max (2^31 - 1) and ckd_priv() not finding a suitable child per BIP32 specifications.
            // TODO: catch this scenario and in all other downlevel wallets.
            key: self.key.ckd_priv(ChildIndex::Hardened(purpose.value())).unwrap(),
            purpose,
        }
    }
}

impl PurposeWallet {
    pub fn for_coin(&self, coin_type: CoinType) -> CoinWallet {
        CoinWallet {
            key: self.key.ckd_priv(ChildIndex::Hardened(coin_type.value())).unwrap(),
            coin_type,
        }
    }
}

impl CoinWallet {
    pub fn for_account(&self, account: u32) -> AccountWallet {
        AccountWallet {
            key: self.key.ckd_priv(ChildIndex::Hardened(account)).unwrap(),
            account,
        }
    }
}

impl AccountWallet {
    pub fn for_chain(&self, chain: u32) -> ChainWallet {
        ChainWallet{
            key: self.key.ckd_priv(ChildIndex::Normal(chain)).unwrap(),
            chain,
        }
    }
}

impl ChainWallet {
    pub fn for_address(&self, index: u32) -> AddressWallet {
        AddressWallet {
            key: self.key.ckd_priv(ChildIndex::Normal(index)).unwrap(),
            index,
        }
    }
}