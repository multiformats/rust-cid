/// ! # cid
/// !
/// ! Implementation of [cid](https://github.com/ipld/cid) in Rust.

extern crate multihash;
extern crate multibase;
extern crate try_from;
extern crate varmint;

use try_from::TryFrom;
use varmint::{WriteVarInt, ReadVarInt};
use std::fmt;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Codec {
    Raw,
    DagProtobuf,
    DagCBOR,
    EthereumBlock,
    EthereumTx,
    BitcoinBlock,
    BitcoinTx,
    ZcashBlock,
    ZcashTx,
}

impl Codec {
    /// Convert to the matching integer code
    pub fn to_code(&self) -> u64 {
        use Codec::*;

        match *self {
            Raw => 0x55,
            DagProtobuf => 0x70,
            DagCBOR => 0x71,
            EthereumBlock => 0x90,
            EthereumTx => 0x91,
            BitcoinBlock => 0xb0,
            BitcoinTx => 0xb1,
            ZcashBlock => 0xc0,
            ZcashTx => 0xc1,
        }
    }

    /// Convert a number to the matching codec
    pub fn from(raw: u64) -> Result<Codec, Error> {
        use Codec::*;

        match raw {
            0x55 => Ok(Raw),
            0x70 => Ok(DagProtobuf),
            0x71 => Ok(DagCBOR),
            0x90 => Ok(EthereumBlock),
            0x91 => Ok(EthereumTx),
            0xb0 => Ok(BitcoinBlock),
            0xb1 => Ok(BitcoinTx),
            0xc0 => Ok(ZcashBlock),
            0xc1 => Ok(ZcashTx),
            _ => Err(Error::UnkownCodec),
        }
    }
}

/// Representation of a CID.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cid {
    pub version: u64,
    pub codec: Codec,
    pub hash: Vec<u8>,
}

/// Prefix represents all metadata of a CID, without the actual content.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Prefix {
    pub version: u64,
    pub codec: Codec,
    pub mh_type: multihash::Hash,
    pub mh_len: usize,
}

/// Error types
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Error {
    UnkownCodec,
    InputTooShort,
    ParsingError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match *self {
            UnkownCodec => write!(f, "Unkown codec"),
            InputTooShort => write!(f, "Input too short"),
            ParsingError => write!(f, "Failed to parse multihash"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        use Error::*;

        match *self {
            UnkownCodec => "Unkown codec",
            InputTooShort => "Input too short",
            ParsingError => "Failed to parse multihash",
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Error {
        Error::ParsingError
    }
}

impl From<multibase::Error> for Error {
    fn from(_: multibase::Error) -> Error {
        Error::ParsingError
    }
}


impl TryFrom<Vec<u8>> for Cid {
    type Err = Error;

    /// Create a Cid from a u8 vector.
    fn try_from(data: Vec<u8>) -> Result<Self, Self::Err> {
        Cid::try_from(data.as_slice())
    }
}

impl TryFrom<String> for Cid {
    type Err = Error;

    /// Create a Cid from a string.
    fn try_from(data: String) -> Result<Self, Self::Err> {
        Cid::try_from(data.as_str())
    }
}

impl<'a> TryFrom<&'a str> for Cid {
    type Err = Error;

    /// Create a Cid from a string slice.
    fn try_from(data: &str) -> Result<Self, Self::Err> {
        let hash = if data.contains("/ipfs/") {
            let matches: Vec<&str> = data.split("/ipfs/").collect();
            matches[1]
        } else {
            data
        };

        if hash.len() < 2 {
            return Err(Error::InputTooShort);
        }

        let mut hash = hash.to_string();

        if hash.len() == 46 && &hash[0..2] == "Qm" {
            hash = multibase::Base::Base58btc.code().to_string() + &hash;
        }

        let decoded = try!(multibase::decode(hash));
        Cid::try_from(decoded.1)
    }
}

impl<'a> TryFrom<&'a [u8]> for Cid {
    type Err = Error;

    /// Create a Cid from a byte slice.
    fn try_from(data: &[u8]) -> Result<Self, Self::Err> {
        // legacy multihash
        let is_legacy = data.len() == 34 && data[0] == 18 && data[1] == 32;

        if is_legacy {
            try!(multihash::decode(data));
            Ok(Cid::new(Codec::DagProtobuf, 0, data.to_vec()))
        } else {
            let mut data = data;
            let version = try!(data.read_u64_varint());
            let raw_codec = try!(data.read_u64_varint());
            let codec = try!(Codec::from(raw_codec));

            try!(multihash::decode(&data));
            Ok(Cid::new(codec, version, (*data).to_vec()))
        }
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.version {
            0 => self.to_string_v0(f),
            1 => self.to_string_v1(f),
            _ => panic!("should not happen"),
        }
    }
}

impl Cid {
    /// Create a new CID.
    pub fn new(codec: Codec, version: u64, hash: Vec<u8>) -> Cid {
        Cid {
            version: version,
            codec: codec,
            hash: hash,
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
        self.hash.iter().cloned().collect()
    }

    fn as_bytes_v1(&self) -> Vec<u8> {
        let mut res = vec![];
        res.write_u64_varint(self.version).unwrap();
        res.write_u64_varint(self.codec.to_code()).unwrap();
        res.extend(self.hash.iter().cloned().collect::<Vec<u8>>());

        res
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self.version {
            0 => self.as_bytes_v0(),
            1 => self.as_bytes_v1(),
            _ => panic!("should not happen"),
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
    pub fn new_from_bytes(data: &[u8]) -> Result<Prefix, Error> {
        let mut data = data;
        let version = try!(data.read_u64_varint());
        let raw_codec = try!(data.read_u64_varint());
        let codec = try!(Codec::from(raw_codec));
        let raw_mh_type = try!(data.read_u64_varint());
        let mh_type = try!(multihash::Hash::from_code(raw_mh_type as u8)
            .ok_or(Error::ParsingError));

        let mh_len = try!(data.read_u64_varint());

        Ok(Prefix {
            version: version,
            codec: codec,
            mh_type: mh_type,
            mh_len: mh_len as usize,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res = vec![];
        res.write_u64_varint(self.version).unwrap();
        res.write_u64_varint(self.codec.to_code()).unwrap();
        res.write_u64_varint(self.mh_type.code() as u64).unwrap();
        res.write_u64_varint(self.mh_len as u64).unwrap();

        res
    }
}
