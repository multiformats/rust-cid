use std::convert::TryFrom;

use multihash::{Multihash, MultihashDigest, SHA2_256};
use quickcheck::{Arbitrary, Gen};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::codec::*;
use crate::{Cid, Version};

const CODECS: [u64; 18] = [
    RAW,
    DAG_PROTOBUF,
    DAG_CBOR,
    DAG_JSON,
    GIT_RAW,
    ETHEREUM_BLOCK,
    ETHEREUM_BLOCK_LIST,
    ETHEREUM_TX_TRIE,
    ETHEREUM_TX,
    ETHEREUM_TX_RECEIPT_TRIE,
    ETHEREUM_RECEIPT,
    ETHEREUM_STATE_TRIE,
    ETHEREUM_ACCOUNT_SNAPSHOT,
    ETHEREUM_STORAGE_TRIE,
    BITCOIN_BLOCK,
    BITCOIN_TX,
    ZCASH_BLOCK,
    ZCASH_TX,
];

const POPULAR: [u64; 4] = [RAW, DAG_PROTOBUF, DAG_CBOR, DAG_JSON];

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
            let hash = Multihash::new(SHA2_256, &data).unwrap().to_raw().unwrap();
            Cid::new_v0(hash).expect("sha2_256 is a valid hash for cid v0")
        } else {
            loop {
                // chose the most frequently used codecs more often
                let codec = if g.gen_bool(0.7) {
                    *POPULAR.choose(g).unwrap()
                } else {
                    *CODECS.choose(g).unwrap()
                };
                let hash: Multihash = Arbitrary::arbitrary(g);
                if hash.size() > 32 {
                    continue;
                }
                return Cid::new_v1(codec, hash.to_raw().unwrap());
            }
        }
    }
}
