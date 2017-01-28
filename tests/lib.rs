extern crate cid;
extern crate try_from;
extern crate multihash;

use cid::{Cid, Codec, Error};
use try_from::TryFrom;

#[test]
fn basic_marshalling() {
    let h = multihash::encode(multihash::Hash::SHA2256, "beep boop".as_bytes()).unwrap();

    let cid = Cid::new(Codec::DagProtobuf, 1, h.to_vec());

    let data = cid.as_bytes();
    let out = Cid::try_from(data).unwrap();

    println!("first {:?}", h);
    assert_eq!(cid, out);

    let s = cid.to_string();
    let out2 = Cid::try_from(&s[..]).unwrap();
    println!("second {:?} {:?} {:?}", cid, out2, &s);
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

    assert_eq!(cid.version, 0);
    assert_eq!(cid.to_string(), old);
}

#[test]
fn v0_error() {
    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII";
    assert_eq!(Cid::try_from(bad), Err(Error::ParsingError));
}
