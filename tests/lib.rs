use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

use cid::{Cid, Codec, Error, Version};
use multibase::Base;
use multihash::Sha2_256;

#[test]
fn basic_marshalling() {
    let h = Sha2_256::digest(b"beep boop");

    let cid = Cid::new_v1(Codec::DagProtobuf, h);

    let data = cid.to_bytes();
    let out = Cid::try_from(data.clone()).unwrap();
    assert_eq!(cid, out);

    let out2: Cid = data.try_into().unwrap();
    assert_eq!(cid, out2);

    let s = cid.to_string();
    let out3 = Cid::try_from(&s[..]).unwrap();
    assert_eq!(cid, out3);

    let out4: Cid = (&s[..]).try_into().unwrap();
    assert_eq!(cid, out4);
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

#[test]
fn test_hash() {
    let data: Vec<u8> = vec![1, 2, 3];
    let hash = Sha2_256::digest(&data);
    let mut map = HashMap::new();
    let cid = Cid::new_v0(hash).unwrap();
    map.insert(cid.clone(), data.clone());
    assert_eq!(&data, map.get(&cid).unwrap());
}

#[test]
fn test_base32() {
    let cid = Cid::from_str("bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy").unwrap();
    assert_eq!(cid.version(), Version::V1);
    assert_eq!(cid.codec(), Codec::Raw);
    assert_eq!(cid.hash(), Sha2_256::digest(b"foo"));
}

#[test]
fn to_string() {
    let expected_cid = "bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy";
    let cid = Cid::new_v1(Codec::Raw, Sha2_256::digest(b"foo"));
    assert_eq!(cid.to_string(), expected_cid);
}

#[test]
fn to_string_of_base32() {
    let expected_cid = "bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy";
    let cid = Cid::new_v1(Codec::Raw, Sha2_256::digest(b"foo"));
    assert_eq!(
        cid.to_string_of_base(Base::Base32Lower).unwrap(),
        expected_cid
    );
}

#[test]
fn to_string_of_base64() {
    let expected_cid = "mAVUSICwmtGto/8aP+ZtFPB0wQTQTQi1wZIO/oPmKXohiZueu";
    let cid = Cid::new_v1(Codec::Raw, Sha2_256::digest(b"foo"));
    assert_eq!(cid.to_string_of_base(Base::Base64).unwrap(), expected_cid);
}

#[test]
fn to_string_of_base58_v0() {
    let expected_cid = "QmRJzsvyCQyizr73Gmms8ZRtvNxmgqumxc2KUp71dfEmoj";
    let cid = Cid::new_v0(Sha2_256::digest(b"foo")).unwrap();
    assert_eq!(
        cid.to_string_of_base(Base::Base58Btc).unwrap(),
        expected_cid
    );
}

#[test]
fn to_string_of_base_v0_error() {
    let cid = Cid::new_v0(Sha2_256::digest(b"foo")).unwrap();
    assert_eq!(
        cid.to_string_of_base(Base::Base16Upper),
        Err(Error::InvalidCidV0Base)
    );
}
