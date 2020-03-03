//! # cid
//!
//! Implementation of [cid](https://github.com/ipld/cid) in Rust.

mod codec;
mod error;
mod to_cid;
mod version;

pub use self::codec::Codec;
pub use self::error::{Error, Result};
pub use self::to_cid::ToCid;
pub use self::version::Version;

use std::fmt;
use std::str::FromStr;

use multibase::Base;
use multihash::Multihash;
use unsigned_varint::{decode as varint_decode, encode as varint_encode};

/// Representation of a CID.
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Cid {
    pub version: Version,
    pub codec: Codec,
    pub hash: Multihash,
}

/// Prefix represents all metadata of a CID, without the actual content.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Prefix {
    pub version: Version,
    pub codec: Codec,
    pub mh_type: multihash::Code,
    pub mh_len: usize,
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
            codec: prefix.codec.to_owned(),
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

    pub fn to_bytes(&self) -> Vec<u8> {
        match self.version {
            Version::V0 => self.to_bytes_v0(),
            Version::V1 => self.to_bytes_v1(),
        }
    }

    pub fn prefix(&self) -> Prefix {
        Prefix {
            version: self.version,
            codec: self.codec.to_owned(),
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

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(),
        };
        write!(f, "{}", output)
    }
}

impl FromStr for Cid {
    type Err = Error;
    fn from_str(src: &str) -> Result<Self> {
        src.to_cid()
    }
}

impl Prefix {
    pub fn new_from_bytes(data: &[u8]) -> Result<Prefix> {
        let (raw_version, remain) = varint_decode::u64(data)?;
        let version = Version::from(raw_version)?;

        let (raw_codec, remain) = varint_decode::u64(remain)?;
        let codec = Codec::from(raw_codec)?;

        let (raw_mh_type, remain) = varint_decode::u64(remain)?;
        let mh_type = match multihash::Code::from_u64(raw_mh_type) {
            multihash::Code::Custom(_) => return Err(Error::UnknownCodec),
            code => code,
        };

        let (mh_len, _remain) = varint_decode::usize(remain)?;

        Ok(Prefix {
            version,
            codec,
            mh_type,
            mh_len,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(4);

        let mut buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(self.version.into(), &mut buf);
        res.extend_from_slice(version);
        let mut buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(self.codec.into(), &mut buf);
        res.extend_from_slice(codec);
        let mut buf = varint_encode::u64_buffer();
        let mh_type = varint_encode::u64(self.mh_type.to_u64(), &mut buf);
        res.extend_from_slice(mh_type);
        let mut buf = varint_encode::u64_buffer();
        let mh_len = varint_encode::u64(self.mh_len as u64, &mut buf);
        res.extend_from_slice(mh_len);

        res
    }
}
