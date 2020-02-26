use std::{fmt, hash, str};

use multibase::Base;
use multihash::MultihashRef;
use unsigned_varint::encode as varint_encode;

use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::prefix::Prefix;
use crate::to_cid::ToCid;
use crate::version::Version;

/// Representation of a CID.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Cid {
    /// The version of CID.
    pub version: Version,
    /// The codec of CID.
    pub codec: Codec,
    /// The hash of CID.
    pub hash: Vec<u8>,
}

impl Cid {
    /// Create a new CID.
    pub fn new(version: Version, codec: Codec, hash: &[u8]) -> Cid {
        Cid {
            version,
            codec,
            hash: hash.into(),
        }
    }

    /// Create a new CID from raw data (binary or multibase encoded string)
    pub fn from<T: ToCid>(data: T) -> Result<Cid> {
        data.to_cid()
    }

    /// Create a new CID from a prefix and some data.
    pub fn new_from_prefix(prefix: &Prefix, data: &[u8]) -> Cid {
        let mut hash = prefix.mh_type.hasher().unwrap().digest(data).into_bytes();
        hash.truncate(prefix.mh_len + 2);
        Cid {
            version: prefix.version,
            codec: prefix.codec,
            hash,
        }
    }

    fn to_string_v0(&self) -> String {
        Base::Base58Btc.encode(&self.hash)
    }

    fn to_string_v1(&self) -> String {
        multibase::encode(Base::Base58Btc, self.to_bytes())
    }

    fn to_bytes_v0(&self) -> Vec<u8> {
        self.hash.clone()
    }

    fn to_bytes_v1(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(16);

        let mut buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(self.version.into(), &mut buf);
        res.extend_from_slice(version);
        let mut buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(self.codec.into(), &mut buf);
        res.extend_from_slice(codec);
        res.extend_from_slice(&self.hash);

        res
    }

    /// Convert CID to encoded bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self.version {
            Version::V0 => self.to_bytes_v0(),
            Version::V1 => self.to_bytes_v1(),
        }
    }

    /// Return the prefix of the CID.
    pub fn prefix(&self) -> Prefix {
        // Unwrap is safe, as this should have been validated on creation
        let mh = MultihashRef::from_slice(&self.hash).unwrap();

        Prefix {
            version: self.version,
            codec: self.codec,
            mh_type: mh.algorithm(),
            mh_len: mh.digest().len(),
        }
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.version {
            Version::V0 => write!(f, "{}", self.to_string_v0()),
            Version::V1 => write!(f, "{}", self.to_string_v1()),
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl hash::Hash for Cid {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}

impl str::FromStr for Cid {
    type Err = Error;
    fn from_str(src: &str) -> Result<Self> {
        src.to_cid()
    }
}
