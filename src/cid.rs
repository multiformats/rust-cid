use std::convert::TryFrom;

use multibase::Base;
use multihash::{Code, MultihashGeneric, MultihashRefGeneric};
use unsigned_varint::{decode as varint_decode, encode as varint_encode};

use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::version::Version;

/// A CID with the default Multihash code table
pub type Cid = CidGeneric<Code>;

/// Representation of a CID.
///
/// Usually you would use `Cid` instead, unless you have a custom Multihash code table
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64>,
{
    /// The version of CID.
    version: Version,
    /// The codec of CID.
    codec: Codec,
    /// The multihash of CID.
    hash: MultihashGeneric<T>,
}

impl<T> CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    /// Create a new CIDv0.
    pub fn new_v0(hash: MultihashGeneric<T>) -> Result<CidGeneric<T>> {
        if hash.algorithm().into() != u64::from(Code::Sha2_256) {
            return Err(Error::InvalidCidV0Multihash);
        }
        Ok(Self {
            version: Version::V0,
            codec: Codec::DagProtobuf,
            hash,
        })
    }

    /// Create a new CIDv1.
    pub fn new_v1(codec: Codec, hash: MultihashGeneric<T>) -> CidGeneric<T> {
        Self {
            version: Version::V1,
            codec,
            hash,
        }
    }

    /// Create a new CID.
    pub fn new(version: Version, codec: Codec, hash: MultihashGeneric<T>) -> Result<CidGeneric<T>> {
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

    /// Returns the cid version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the cid codec.
    pub fn codec(&self) -> Codec {
        self.codec
    }

    /// Returns the cid multihash.
    pub fn hash(&self) -> MultihashRefGeneric<T> {
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
}

#[allow(clippy::derive_hash_xor_eq)]
impl<T> std::hash::Hash for CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}

impl<T> std::fmt::Display for CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output = match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(),
        };
        write!(f, "{}", output)
    }
}

impl<T> std::str::FromStr for CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    type Err = Error;

    fn from_str(cid_str: &str) -> Result<Self> {
        CidGeneric::try_from(cid_str)
    }
}

impl<T> TryFrom<String> for CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    type Error = Error;

    fn try_from(cid_str: String) -> Result<Self> {
        Self::try_from(cid_str.as_str())
    }
}

impl<T> TryFrom<&str> for CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
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

impl<T> TryFrom<Vec<u8>> for CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        Self::try_from(bytes.as_slice())
    }
}

impl<T> TryFrom<&[u8]> for CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if Version::is_v0_binary(bytes) {
            let mh = MultihashRefGeneric::from_slice(bytes)?.to_owned();
            CidGeneric::new_v0(mh)
        } else {
            let (raw_version, remain) = varint_decode::u64(&bytes)?;
            let version = Version::try_from(raw_version)?;

            let (raw_codec, hash) = varint_decode::u64(&remain)?;
            let codec = Codec::try_from(raw_codec)?;

            let mh = MultihashRefGeneric::from_slice(hash)?.to_owned();

            CidGeneric::new(version, codec, mh)
        }
    }
}

impl<T> From<&CidGeneric<T>> for CidGeneric<T>
where
    T: Into<u64> + TryFrom<u64> + Copy,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    fn from(cid: &CidGeneric<T>) -> Self {
        cid.to_owned()
    }
}

impl<T> From<CidGeneric<T>> for Vec<u8>
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    fn from(cid: CidGeneric<T>) -> Self {
        cid.to_bytes()
    }
}

impl<T> From<CidGeneric<T>> for String
where
    T: Into<u64> + TryFrom<u64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    fn from(cid: CidGeneric<T>) -> Self {
        cid.to_string()
    }
}
