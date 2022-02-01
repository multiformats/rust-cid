use std::convert::TryFrom;

use multihash::{Code, MultihashDigest, MultihashGeneric};
use quickcheck::{Arbitrary, Gen};
use rand::{
    distributions::{weighted::WeightedIndex, Distribution},
    Rng,
};

use crate::{CidGeneric, Version};

impl Arbitrary for Version {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let version = if g.gen_bool(0.7) { 1 } else { 0 };
        Version::try_from(version).unwrap()
    }
}

impl<const S: usize> Arbitrary for CidGeneric<S> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        if S >= 32 && <Version as Arbitrary>::arbitrary(g) == Version::V0 {
            let data: Vec<u8> = Arbitrary::arbitrary(g);
            let hash = Code::Sha2_256
                .digest(&data)
                .resize()
                .expect("digest too large");
            CidGeneric::new_v0(hash).expect("sha2_256 is a valid hash for cid v0")
        } else {
            // In real world lower IPLD Codec codes more likely to happen, hence distribute them
            // with bias towards smaller values.
            let weights = [128, 32, 4, 4, 2, 2, 1, 1];
            let dist = WeightedIndex::new(weights.iter()).unwrap();
            let codec = match dist.sample(g) {
                0 => g.gen_range(0, u64::pow(2, 7)),
                1 => g.gen_range(u64::pow(2, 7), u64::pow(2, 14)),
                2 => g.gen_range(u64::pow(2, 14), u64::pow(2, 21)),
                3 => g.gen_range(u64::pow(2, 21), u64::pow(2, 28)),
                4 => g.gen_range(u64::pow(2, 28), u64::pow(2, 35)),
                5 => g.gen_range(u64::pow(2, 35), u64::pow(2, 42)),
                6 => g.gen_range(u64::pow(2, 42), u64::pow(2, 49)),
                7 => g.gen_range(u64::pow(2, 56), u64::pow(2, 63)),
                _ => unreachable!(),
            };

            let hash: MultihashGeneric<S> = Arbitrary::arbitrary(g);
            CidGeneric::new_v1(codec, hash)
        }
    }
}
