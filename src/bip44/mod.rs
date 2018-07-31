use bip32::ExtendedKey;
use bip39::MnemonicSeed;

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

pub struct Wallet {
    master: ExtendedKey,
}

impl Wallet {
    pub fn from_mnemonic(mnemonic: &str) -> Wallet {
        panic!("not implemented");
    }
}