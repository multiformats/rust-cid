use std::io::Cursor;
use std::str::FromStr;

use integer_encoding::VarIntReader;
use multibase::Base;
use multihash::{self, MultihashRef};

use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::version::Version;
use crate::Cid;

pub trait ToCid {
    fn to_cid(&self) -> Result<Cid>;
}

impl ToCid for Vec<u8> {
    /// Create a Cid from a byte vector.
    #[inline]
    fn to_cid(&self) -> Result<Cid> {
        self.as_slice().to_cid()
    }
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

impl FromStr for Cid {
    type Err = Error;
    fn from_str(src: &str) -> Result<Self> {
        src.to_cid()
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
            let mh = MultihashRef::from_slice(self)?.to_owned();
            Ok(Cid::new(Codec::DagProtobuf, Version::V0, mh))
        } else {
            let mut cur = Cursor::new(self);
            let raw_version = cur.read_varint()?;
            let raw_codec = cur.read_varint()?;

            let version = Version::from(raw_version)?;
            let codec = Codec::from(raw_codec)?;

            let hash = &self[cur.position() as usize..];

            let mh = MultihashRef::from_slice(hash)?.to_owned();

            Ok(Cid::new(codec, version, mh))
        }
    }
}
