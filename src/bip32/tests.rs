use bip32::*;
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