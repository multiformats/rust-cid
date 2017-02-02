/// ! # cid
/// !
/// ! Implementation of [cid](https://github.com/ipld/cid) in Rust.

extern crate multihash;
extern crate multibase;
extern crate try_from;
extern crate varmint;

use std::io;

pub mod error;
pub mod codec;
pub mod version;

pub use codec::Codec;
pub use version::Version;
pub use error::{Error, Result};

// No idea why these can't be in error.rs, will look later
impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error::ParsingError
    }
}

impl From<multibase::Error> for Error {
    fn from(_: multibase::Error) -> Error {
        Error::ParsingError
    }
}

impl From<multihash::Error> for Error {
    fn from(_: multihash::Error) -> Error {
        Error::ParsingError
    }
}

use try_from::TryFrom;
use varmint::{WriteVarInt, ReadVarInt};
use std::fmt;

/// Representation of a CID.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cid {
    pub version: Version,
    pub codec: Codec,
    pub hash: Vec<u8>,
}

/// Prefix represents all metadata of a CID, without the actual content.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Prefix {
    pub version: Version,
    pub codec: Codec,
    pub mh_type: multihash::Hash,
    pub mh_len: usize,
}

impl TryFrom<Vec<u8>> for Cid {
    type Err = Error;

    /// Create a Cid from a u8 vector.
    fn try_from(data: Vec<u8>) -> Result<Self> {
        Cid::try_from(data.as_slice())
    }
}

impl TryFrom<String> for Cid {
    type Err = Error;

    /// Create a Cid from a string.
    fn try_from(data: String) -> Result<Self> {
        Cid::try_from(data.as_str())
    }
}

impl<'a> TryFrom<&'a str> for Cid {
    type Err = Error;

    /// Create a Cid from a string slice.
    fn try_from(data: &str) -> Result<Self> {
        let hash = if data.contains("/ipfs/") {
            let matches: Vec<&str> = data.split("/ipfs/").collect();
            matches[1]
        } else {
            data
        };

        if hash.len() < 2 {
            return Err(Error::InputTooShort);
        }

        let (_, decoded) = if Version::is_v0_str(hash) {
            // TODO: could avoid the roundtrip here and just use underlying
            // base-x base58btc decoder here.
            let hash = multibase::Base::Base58btc.code().to_string() + &hash;

            multibase::decode(hash)
        } else {
            multibase::decode(hash)
        }?;

        Cid::try_from(decoded)
    }
}

impl<'a> TryFrom<&'a [u8]> for Cid {
    type Err = Error;

    /// Create a Cid from a byte slice.
    fn try_from(data: &[u8]) -> Result<Self> {
        if Version::is_v0_binary(data) {
            multihash::decode(data)?;

            Ok(Cid::new(Codec::DagProtobuf, Version::V0, data))
        } else {
            let mut data = data;
            let raw_version = data.read_u64_varint()?;
            let raw_codec = data.read_u64_varint()?;

            let version = Version::from(raw_version)?;
            let codec = Codec::from(raw_codec)?;

            multihash::decode(data)?;

            Ok(Cid::new(codec, version, data))
        }
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.version {
            Version::V0 => self.to_string_v0(f),
            Version::V1 => self.to_string_v1(f),
        }
    }
}

impl Cid {
    /// Create a new CID.
    pub fn new(codec: Codec, version: Version, hash: &[u8]) -> Cid {
        Cid {
            version: version,
            codec: codec,
            hash: hash.into(),
        }
    }

    /// Create a new CID from a prefix and some data.
    pub fn new_from_prefix(prefix: &Prefix, data: &[u8]) -> Cid {
        let mut hash = multihash::encode(prefix.mh_type.to_owned(), data).unwrap();
        hash.truncate(prefix.mh_len + 2);
        Cid {
            version: prefix.version,
            codec: prefix.codec.to_owned(),
            hash: hash,
        }
    }

    fn to_string_v0(&self, f: &mut fmt::Formatter) -> fmt::Result {
        multibase::encode(multibase::Base::Base58btc, self.hash.as_slice())
            .map_err(|_| fmt::Error {})
            .and_then(|enc| {
                // Drop the first character as v0 does not know                 // about multibase
                f.write_str(&enc[1..])
            })
    }

    fn to_string_v1(&self, f: &mut fmt::Formatter) -> fmt::Result {
        multibase::encode(multibase::Base::Base58btc, self.as_bytes().as_slice())
            .map_err(|_| fmt::Error {})
            .and_then(|enc| f.write_str(&enc))
    }

    fn as_bytes_v0(&self) -> Vec<u8> {
        self.hash.clone()
    }

    fn as_bytes_v1(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(16);
        res.write_u64_varint(self.version.into()).unwrap();
        res.write_u64_varint(self.codec.into()).unwrap();
        res.extend_from_slice(&self.hash);

        res
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self.version {
            Version::V0 => self.as_bytes_v0(),
            Version::V1 => self.as_bytes_v1(),
        }
    }

    pub fn prefix(&self) -> Prefix {
        // Unwrap is safe, as this should have been validated on creation
        let mh = multihash::decode(self.hash.as_slice()).unwrap();

        Prefix {
            version: self.version,
            codec: self.codec.to_owned(),
            mh_type: mh.alg,
            mh_len: mh.digest.len(),
        }
    }
}

impl Prefix {
    pub fn new_from_bytes(data: &[u8]) -> Result<Prefix> {
        let mut data = data;
        let raw_version = data.read_u64_varint()?;
        let raw_codec = data.read_u64_varint()?;
        let raw_mh_type = data.read_u64_varint()?;

        let version = Version::from(raw_version)?;
        let codec = Codec::from(raw_codec)?;
        let mh_type = multihash::Hash::from_code(raw_mh_type as u8)?;

        let mh_len = data.read_u64_varint()?;

        Ok(Prefix {
            version: version,
            codec: codec,
            mh_type: mh_type,
            mh_len: mh_len as usize,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(4);

        // io can't fail on Vec
        res.write_u64_varint(self.version.into()).unwrap();
        res.write_u64_varint(self.codec.into()).unwrap();
        res.write_u64_varint(self.mh_type.code() as u64).unwrap();
        res.write_u64_varint(self.mh_len as u64).unwrap();

        res
    }
}
