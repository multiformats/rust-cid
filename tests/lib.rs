use cid::{Cid, Codec, Error, Version};
use core::convert::TryFrom;
use multihash::{MultihashDigest, Sha2_256};

#[test]
fn basic_marshalling() {
    let hash = Sha2_256::digest(b"beep boop");

    let cid = Cid::new(Version::V1, Codec::DagProtobuf, hash).unwrap();

    let data = cid.to_bytes();
    let out = Cid::try_from(data).unwrap();

    assert_eq!(cid, out);

    let s = cid.to_string();
    let out2 = Cid::try_from(&s[..]).unwrap();

    assert_eq!(cid, out2);
}

#[test]
fn empty_string() {
    assert_eq!(Cid::try_from(""), Err(Error::InputTooShort));
}

#[test]
fn v0_handling() {
    let old = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";
    let cid = Cid::try_from(old).unwrap();

    assert_eq!(cid.version(), Version::V0);
    assert_eq!(cid.to_string(), old);
}

#[test]
fn from_str() {
    let cid: Cid = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
        .parse()
        .unwrap();
    assert_eq!(cid.version(), Version::V0);

    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII".parse::<Cid>();
    assert_eq!(bad, Err(Error::ParsingError));
}

#[test]
fn v0_error() {
    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII";
    assert_eq!(Cid::try_from(bad), Err(Error::ParsingError));
}

#[test]
fn from() {
    let the_hash = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";

    let cases = vec![
        format!("/ipfs/{:}", &the_hash),
        format!("https://ipfs.io/ipfs/{:}", &the_hash),
        format!("http://localhost:8080/ipfs/{:}", &the_hash),
    ];

    for case in cases {
        let cid = Cid::try_from(case).unwrap();
        assert_eq!(cid.version(), Version::V0);
        assert_eq!(cid.to_string(), the_hash);
    }
}

#[cfg(feature = "random")]
#[test]
fn test_random() {
    let cid = Cid::random();
    let cid2 = Cid::try_from(cid.to_bytes()).unwrap();
    assert_eq!(cid, cid2);
}
