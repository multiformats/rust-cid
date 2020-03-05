use std::convert::TryFrom;

use multibase::Base;
use multihash::{Code, Multihash, MultihashRef};
use unsigned_varint::{decode as varint_decode, encode as varint_encode};

use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::prefix::Prefix;
use crate::version::Version;

/// Representation of a CID.
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Cid {
    /// The version of CID.
    version: Version,
    /// The codec of CID.
    codec: Codec,
    /// The multihash of CID.
    hash: Multihash,
}

impl Cid {
    /// Create a new CIDv0.
    pub fn new_v0(hash: Multihash) -> Result<Cid> {
        if hash.algorithm() != Code::Sha2_256 {
            return Err(Error::InvalidCidV0Multihash);
        }
        Ok(Cid {
            version: Version::V0,
            codec: Codec::DagProtobuf,
            hash,
        })
    }

    /// Create a new CIDv1.
    pub fn new_v1(codec: Codec, hash: Multihash) -> Cid {
        Cid {
            version: Version::V1,
            codec,
            hash,
        }
    }

    /// Create a new CID.
    pub fn new(version: Version, codec: Codec, hash: Multihash) -> Result<Cid> {
        match version {
            Version::V0 => {
                if codec != Codec::DagProtobuf {
                    return Err(Error::InvalidCidV0Codec);
                }
                Self::new_v0(hash)
            }
            Version::V1 => Ok(Self::new_v1(codec, hash)),
        }
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

    /// Returns the cid version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the cid codec.
    pub fn codec(&self) -> Codec {
        self.codec
    }

    /// Returns the cid multihash.
    pub fn hash(&self) -> MultihashRef {
        self.hash.as_ref()
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

    fn from_str(cid_str: &str) -> Result<Self> {
        Cid::try_from(cid_str)
    }
}

impl TryFrom<String> for Cid {
    type Error = Error;

    fn try_from(cid_str: String) -> Result<Self> {
        Self::try_from(cid_str.as_str())
    }
}

impl TryFrom<&str> for Cid {
    type Error = Error;

    fn try_from(cid_str: &str) -> Result<Self> {
        static IPFS_DELIMETER: &str = "/ipfs/";

        let hash = match cid_str.find(IPFS_DELIMETER) {
            Some(index) => &cid_str[index + IPFS_DELIMETER.len()..],
            _ => cid_str,
        };

        if hash.len() < 2 {
            return Err(Error::InputTooShort);
        }

        let decoded = if Version::is_v0_str(hash) {
            Base::Base58Btc.decode(hash)?
        } else {
            let (_, decoded) = multibase::decode(hash)?;
            decoded
        };

        Self::try_from(decoded)
    }
}

impl TryFrom<Vec<u8>> for Cid {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for Cid {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if Version::is_v0_binary(bytes) {
            let mh = MultihashRef::from_slice(bytes)?.to_owned();
            Cid::new_v0(mh)
        } else {
            let (raw_version, remain) = varint_decode::u64(&bytes)?;
            let version = Version::try_from(raw_version)?;

            let (raw_codec, hash) = varint_decode::u64(&remain)?;
            let codec = Codec::from(raw_codec)?;

            let mh = MultihashRef::from_slice(hash)?.to_owned();

            Cid::new(version, codec, mh)
        }
    }
}

impl From<&Cid> for Cid {
    fn from(cid: &Cid) -> Self {
        cid.to_owned()
    }
}

impl From<Cid> for Vec<u8> {
    fn from(cid: Cid) -> Self {
        cid.to_bytes()
    }
}

impl From<Cid> for String {
    fn from(cid: Cid) -> Self {
        cid.to_string()
    }
}
