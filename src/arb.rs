use crate::{Cid, Codec, Prefix, Version};
use multihash::Multihash;
use quickcheck::{Arbitrary, Gen};
use rand::seq::SliceRandom;
use rand::Rng;

const CODECS: [Codec; 18] = [
    Codec::Raw,
    Codec::DagProtobuf,
    Codec::DagCBOR,
    Codec::GitRaw,
    Codec::EthereumBlock,
    Codec::EthereumBlockList,
    Codec::EthereumTxTrie,
    Codec::EthereumTx,
    Codec::EthereumTxReceiptTrie,
    Codec::EthereumTxReceipt,
    Codec::EthereumStateTrie,
    Codec::EthereumAccountSnapshot,
    Codec::EthereumStorageTrie,
    Codec::BitcoinBlock,
    Codec::BitcoinTx,
    Codec::ZcashBlock,
    Codec::ZcashTx,
    Codec::DagJSON,
];

const POPULAR: [Codec; 4] = [
    Codec::Raw,
    Codec::DagProtobuf,
    Codec::DagCBOR,
    Codec::DagJSON,
];

impl Arbitrary for Codec {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        // chose the most frequently used codecs more often
        if g.gen_bool(0.7) {
            *POPULAR.choose(g).unwrap()
        } else {
            *CODECS.choose(g).unwrap()
        }
    }
}

impl Arbitrary for Version {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let version = if g.gen_bool(0.7) { 1 } else { 0 };
        Version::from(version).unwrap()
    }
}

impl Arbitrary for Cid {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let version: Version = Arbitrary::arbitrary(g);
        let v0 = version == Version::V0;
        let codec: Codec = if !v0 {
            // v1 supports arbitrary encodings
            Arbitrary::arbitrary(g)
        } else {
            // v0 does only support DagProtobuf encoding
            Codec::DagProtobuf
        };
        let hash: Multihash = if !v0 {
            Arbitrary::arbitrary(g)
        } else {
            let data: Vec<u8> = Arbitrary::arbitrary(g);
            multihash::Sha2_256::digest(&data)
        };
        Cid::new(codec, version, hash)
    }
}

impl Arbitrary for Prefix {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let cid: Cid = Arbitrary::arbitrary(g);
        Prefix {
            version: cid.version,
            codec: cid.codec,
            mh_type: cid.hash.algorithm(),
            mh_len: cid.hash.digest().len(),
        }
    }
}
