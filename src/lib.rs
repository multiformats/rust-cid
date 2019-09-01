/// ! # cid
/// !
/// ! Implementation of [cid](https://github.com/ipld/cid) in Rust.
use core::{convert::TryFrom, fmt, str::FromStr};
use integer_encoding::{VarIntReader, VarIntWriter};
use multibase::Base;
use multihash::Multihash;
use std::io::Cursor;

mod codec;
mod error;
mod version;

pub use codec::Codec;
pub use error::Error;
pub use version::Version;

/// Representation of a CID.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cid {
    pub version: Version,
    pub codec: Codec,
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

    /// Create a new CID from a prefix and a multihash.
    pub fn new_from_prefix(prefix: &Prefix, hash: Multihash) -> Cid {
        Cid {
            version: prefix.version,
            codec: prefix.codec,
            hash: hash,
        }
    }

    fn to_string_v0(&self) -> String {
        let mut string = multibase::encode(Base::Base58btc, &self.hash.as_ref());

        // Drop the first character as v0 does not know
        // about multibase
        string.remove(0);

        string
    }

    fn to_string_v1(&self) -> String {
        multibase::encode(Base::Base58btc, self.to_bytes().as_slice())
    }

    pub fn to_string(&self) -> String {
        match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(),
        }
    }

    fn to_bytes_v0(&self) -> Vec<u8> {
        self.hash.to_bytes()
    }

    fn to_bytes_v1(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(16);
        res.write_varint(u64::from(self.version)).unwrap();
        res.write_varint(u64::from(self.codec)).unwrap();
        res.extend_from_slice(&self.hash.as_ref());
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
            codec: self.codec,
        }
    }
}

impl TryFrom<&[u8]> for Cid {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if Version::is_v0_binary(bytes) {
            // Verify that hash can be decoded, this is very cheap
            let hash = multihash::decode(bytes)?;

            Ok(Cid::new(Codec::DagProtobuf, Version::V0, hash))
        } else {
            let mut cur = Cursor::new(bytes);
            let raw_version = cur.read_varint()?;
            let raw_codec = cur.read_varint()?;

            let version = Version::from(raw_version)?;
            let codec = Codec::from(raw_codec)?;

            let hash = &bytes[cur.position() as usize..];

            // Verify that hash can be decoded, this is very cheap
            let hash = multihash::decode(hash)?;

            Ok(Self::new(codec, version, hash))
        }
    }
}

impl TryFrom<Vec<u8>> for Cid {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&str> for Cid {
    type Error = Error;

    fn try_from(cid_str: &str) -> Result<Self, Self::Error> {
        static IPFS_DELIMETER: &'static str = "/ipfs/";

        let hash = match cid_str.find(IPFS_DELIMETER) {
            Some(index) => &cid_str[index + IPFS_DELIMETER.len()..],
            _ => cid_str,
        };

        if hash.len() < 2 {
            return Err(Error::InputTooShort);
        }

        let (_, bytes) = if Version::is_v0_str(hash) {
            // TODO: could avoid the roundtrip here and just use underlying
            // base-x base58btc decoder here.
            let hash = Base::Base58btc.code().to_string() + &hash;

            multibase::decode(hash)
        } else {
            multibase::decode(hash)
        }?;

        Self::try_from(bytes)
    }
}

impl TryFrom<String> for Cid {
    type Error = Error;

    fn try_from(cid_str: String) -> Result<Self, Self::Error> {
        Self::try_from(cid_str.as_str())
    }
}

impl FromStr for Cid {
    type Err = Error;

    fn from_str(cid_str: &str) -> Result<Self, Self::Err> {
        Cid::try_from(cid_str)
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Self::to_string(self))
    }
}

/// Prefix represents all metadata of a CID, without the actual content.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Prefix {
    pub version: Version,
    pub codec: Codec,
}

impl Prefix {
    pub fn new_from_bytes(data: &[u8]) -> Result<Prefix, Error> {
        let mut cur = Cursor::new(data);

        let raw_version = cur.read_varint()?;
        let raw_codec = cur.read_varint()?;

        let version = Version::from(raw_version)?;
        let codec = Codec::from(raw_codec)?;

        Ok(Prefix { version, codec })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(4);

        // io can't fail on Vec
        res.write_varint(u64::from(self.version)).unwrap();
        res.write_varint(u64::from(self.codec)).unwrap();

        res
    }
}
