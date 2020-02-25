use integer_encoding::VarIntReader;
use multibase;
use multihash::{self, Multihash};
use std::io::Cursor;
use std::str::FromStr;

use crate::{Cid, Codec, Error, Result, Version};

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

        let (_, decoded) = if Version::is_v0_str(hash) {
            // TODO: could avoid the roundtrip here and just use underlying
            // base-x Base58Btc decoder here.
            let hash = multibase::Base::Base58Btc.code().to_string() + &hash;

            multibase::decode(hash)
        } else {
            let mb = multibase::decode(hash);

            mb
        }?;

        decoded.to_cid()
    }
}

impl FromStr for Cid {
    type Err = Error;
    fn from_str(src: &str) -> Result<Self> {
        let res = src.to_cid();
        res
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
            let _mh = Multihash::from_bytes(Vec::from(self))?;
            Ok(Cid::new(Codec::DagProtobuf, Version::V0, self))
        } else {
            let mut cur = Cursor::new(self);
            let raw_version = cur.read_varint()?;
            let raw_codec = cur.read_varint()?;

            let version = Version::from(raw_version)?;
            let codec = Codec::from(raw_codec)?;

            let hash = &self[cur.position() as usize..];

            // Verify that hash can be decoded, this is very cheap
            let _mh = Multihash::from_bytes(Vec::from(hash))?;

            Ok(Cid::new(codec, version, hash))
        }
    }
}
