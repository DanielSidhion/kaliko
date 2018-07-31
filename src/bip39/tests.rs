use bip39::*;
use hex::FromHex;

const TEST_SEEDS: &[&'static str] = &[
    "00000000000000000000000000000000",
    "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
    "80808080808080808080808080808080",
    "ffffffffffffffffffffffffffffffff",
    "000000000000000000000000000000000000000000000000",
    "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
    "808080808080808080808080808080808080808080808080",
    "ffffffffffffffffffffffffffffffffffffffffffffffff",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
    "8080808080808080808080808080808080808080808080808080808080808080",
    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    "9e885d952ad362caeb4efe34a8e91bd2",
    "6610b25967cdcca9d59875f5cb50b0ea75433311869e930b",
    "68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
    "c0ba5a8e914111210f2bd131f3d5e08d",
    "6d9be1ee6ebd27a258115aad99b7317b9c8d28b6d76431c3",
    "9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863",
    "23db8160a31d3e0dca3688ed941adbf3",
    "8197a4a47f0425faeaa69deebc05ca29c0a5b5cc76ceacc0",
    "066dca1a2bb7e8a1db2832148ce9933eea0f3ac9548d793112d9a95c9407efad",
    "f30f8c1da665478f49b001d94c5fc452",
    "c10ec20dc3cd9f652c7fac2f1230f7a3c828389a14392f05",
    "f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f",
];

const EXPECTED_MNEMONICS: &[&'static str] = &[
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    "legal winner thank year wave sausage worth useful legal winner thank yellow",
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
    "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon agent",
    "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal will",
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter always",
    "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo when",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
    "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title",
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic bless",
    "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote",
    "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic",
    "gravity machine north sort system female filter attitude volume fold club stay feature office ecology stable narrow fog",
    "hamster diagram private dutch cause delay private meat slide toddler razor book happy fancy gospel tennis maple dilemma loan word shrug inflict delay length",
    "scheme spot photo card baby mountain device kick cradle pact join borrow",
    "horn tenant knee talent sponsor spell gate clip pulse soap slush warm silver nephew swap uncle crack brave",
    "panda eyebrow bullet gorilla call smoke muffin taste mesh discover soft ostrich alcohol speed nation flash devote level hobby quick inner drive ghost inside",
    "cat swing flag economy stadium alone churn speed unique patch report train",
    "light rule cinnamon wrap drastic word pride squirrel upgrade then income fatal apart sustain crack supply proud access",
    "all hour make first leader extend hole alien behind guard gospel lava path output census museum junior mass reopen famous sing advance salt reform",
    "vessel ladder alter error federal sibling chat ability sun glass valve picture",
    "scissors invite lock maple supreme raw rapid void congress muscle digital elegant little brisk hair mango congress clump",
    "void come effort suffer camp survey warrior heavy shoot primary clutch crush open amazing screen patrol group space point ten exist slush involve unfold",
];

#[test]
fn trezor_seeds() {
    for (seed, mnemonic) in TEST_SEEDS.iter().zip(EXPECTED_MNEMONICS.iter()) {
        let data = Vec::from_hex(seed).unwrap();
        let mnemonic_seed = MnemonicSeed::from_slice(&data).unwrap();

        assert_eq!(mnemonic_seed.as_words(WordList::English), *mnemonic);
    }
}