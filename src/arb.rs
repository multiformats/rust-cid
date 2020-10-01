use std::convert::TryFrom;

use multihash::{Code, Multihash, MultihashDigest, U64};
use quickcheck::{Arbitrary, Gen};
use rand::Rng;

use crate::{Cid, Version};

impl Arbitrary for Version {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let version = if g.gen_bool(0.7) { 1 } else { 0 };
        Version::try_from(version).unwrap()
    }
}

impl Arbitrary for Cid {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let version: Version = Arbitrary::arbitrary(g);
        if version == Version::V0 {
            let data: Vec<u8> = Arbitrary::arbitrary(g);
            let hash = Code::Sha2_256.digest(&data);
            Cid::new_v0(hash).expect("sha2_256 is a valid hash for cid v0")
        } else {
            let codec = u64::arbitrary(g);
            let hash: Multihash<U64> = Arbitrary::arbitrary(g);
            Cid::new_v1(codec, hash)
        }
    }
}
