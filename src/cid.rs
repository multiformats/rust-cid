use multibase::Base;
use multihash::Multihash;
use unsigned_varint::encode as varint_encode;

use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::prefix::Prefix;
use crate::to_cid::ToCid;
use crate::version::Version;

/// Representation of a CID.
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Cid {
    /// The version of CID.
    pub version: Version,
    /// The codec of CID.
    pub codec: Codec,
    /// The multihash of CID.
    pub hash: Multihash,
}

impl Cid {
    /// Create a new CID.
    pub fn new(codec: Codec, version: Version, hash: Multihash) -> Cid {
        Cid {
            version,
            codec,
            hash,
        }
    }

    /// Create a new CID from raw data (binary or multibase encoded string)
    pub fn from<T: ToCid>(data: T) -> Result<Cid> {
        data.to_cid()
    }

    /// Create a new CID from a prefix and some data.
    pub fn new_from_prefix(prefix: &Prefix, data: &[u8]) -> Cid {
        let mut hash = prefix.mh_type.hasher().unwrap().digest(data);
        if prefix.mh_len < hash.digest().len() {
            hash = multihash::wrap(hash.algorithm(), &hash.digest()[..prefix.mh_len]);
        }
        Cid {
            version: prefix.version,
            codec: prefix.codec,
            hash,
        }
    }

    fn to_string_v0(&self) -> String {
        Base::Base58Btc.encode(self.hash.as_bytes())
    }

    fn to_string_v1(&self) -> String {
        multibase::encode(Base::Base32Lower, self.to_bytes().as_slice())
    }

    fn to_bytes_v0(&self) -> Vec<u8> {
        self.hash.to_vec()
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
        Prefix {
            version: self.version,
            codec: self.codec,
            mh_type: self.hash.algorithm(),
            mh_len: self.hash.digest().len(),
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for Cid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}

impl std::fmt::Display for Cid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output = match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(),
        };
        write!(f, "{}", output)
    }
}

impl std::str::FromStr for Cid {
    type Err = Error;
    fn from_str(src: &str) -> Result<Self> {
        src.to_cid()
    }
}
