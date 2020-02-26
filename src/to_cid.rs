use multibase::Base;
use multihash::MultihashRef;
use unsigned_varint::decode as varint_decode;

use crate::cid::Cid;
use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::version::Version;

/// A trait for converting data into CID format.
pub trait ToCid {
    /// The only method for converting data into CID format in the `ToCid` trait.
    fn to_cid(&self) -> Result<Cid>;
}

impl ToCid for String {
    /// Create a Cid from an owned String.
    #[inline]
    fn to_cid(&self) -> Result<Cid> {
        self.as_str().to_cid()
    }
}

impl<'a> ToCid for &'a str {
    #[inline]
    fn to_cid(&self) -> Result<Cid> {
        ToCid::to_cid(*self)
    }
}

impl ToCid for str {
    fn to_cid(&self) -> Result<Cid> {
        static IPFS_DELIMETER: &str = "/ipfs/";

        let hash = match self.find(IPFS_DELIMETER) {
            Some(index) => &self[index + IPFS_DELIMETER.len()..],
            _ => self,
        };

        if hash.len() < 2 {
            return Err(Error::InputTooShort);
        }

        if Version::is_v0_str(hash) {
            let decoded = Base::Base58Btc.decode(hash)?;
            decoded.to_cid()
        } else {
            let (_, decoded) = multibase::decode(hash)?;
            decoded.to_cid()
        }
    }
}

impl ToCid for Vec<u8> {
    /// Create a Cid from a byte vector.
    #[inline]
    fn to_cid(&self) -> Result<Cid> {
        self.as_slice().to_cid()
    }
}

impl<'a> ToCid for &'a [u8] {
    #[inline]
    fn to_cid(&self) -> Result<Cid> {
        ToCid::to_cid(*self)
    }
}

impl ToCid for [u8] {
    /// Create a Cid from a byte slice.
    fn to_cid(&self) -> Result<Cid> {
        if Version::is_v0_binary(self) {
            // Verify that hash can be decoded, this is very cheap
            let _hash = MultihashRef::from_slice(self)?;

            Ok(Cid::new(Version::V0, Codec::DagProtobuf, self))
        } else {
            let (raw_version, remain) = varint_decode::u64(&self)?;
            let version = Version::from(raw_version)?;
            let (raw_codec, hash) = varint_decode::u64(&remain)?;
            let codec = Codec::from(raw_codec)?;

            // Verify that hash can be decoded, this is very cheap
            let _hash = MultihashRef::from_slice(hash)?;

            Ok(Cid::new(version, codec, hash))
        }
    }
}
