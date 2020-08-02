use crate::codec::DAG_PROTOBUF;
use crate::error::{Error, Result};
use crate::version::Version;
use core::convert::TryFrom;
use multihash::RawMultihash;

/// Representation of a CID.
///
/// Usually you would use `Cid` instead, unless you have a custom Multihash code table
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cid {
    /// The version of CID.
    version: Version,
    /// The codec of CID.
    codec: u64,
    /// The multihash of CID.
    hash: RawMultihash,
}

impl Cid {
    /// Create a new CIDv0.
    pub fn new_v0(hash: RawMultihash) -> Result<Self> {
        if hash.code() != 0x12 && hash.size() != 0x20 {
            return Err(Error::InvalidCidV0Multihash);
        }
        Ok(Self {
            version: Version::V0,
            // Convert the code of `DagProtobuf` into the given code table
            codec: DAG_PROTOBUF,
            hash,
        })
    }

    /// Create a new CIDv1.
    pub fn new_v1(codec: u64, hash: RawMultihash) -> Self {
        Self {
            version: Version::V1,
            codec: codec.into(),
            hash,
        }
    }

    /// Create a new CID.
    pub fn new(version: Version, codec: u64, hash: RawMultihash) -> Result<Self> {
        match version {
            Version::V0 => {
                if codec != DAG_PROTOBUF {
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
    pub fn codec(&self) -> u64 {
        self.codec
    }

    /// Returns the cid multihash.
    pub fn hash(&self) -> &RawMultihash {
        &self.hash
    }

    /// Reads the bytes from a byte stream.
    #[cfg(feature = "std")]
    pub fn read_bytes<R: std::io::Read>(mut r: R) -> Result<Self> {
        use unsigned_varint::io::read_u64;
        let version = read_u64(&mut r)?;
        let codec = read_u64(&mut r)?;
        if &[version, codec] == &[0x12, 0x20] {
            let mut digest = [0u8; 32];
            r.read_exact(&mut digest)?;
            let mh = RawMultihash::wrap(version, &digest).unwrap();
            Self::new_v0(mh)
        } else {
            let version = Version::try_from(version)?;
            let mh = RawMultihash::read(r)?;
            Self::new(version, codec, mh)
        }
    }

    #[cfg(feature = "std")]
    fn write_bytes_v1<W: std::io::Write>(&self, mut w: W) -> Result<()> {
        use unsigned_varint::encode as varint_encode;

        let mut version_buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(self.version.into(), &mut version_buf);

        let mut codec_buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(self.codec.into(), &mut codec_buf);

        w.write_all(version)?;
        w.write_all(codec)?;
        self.hash.write(&mut w)?;
        Ok(())
    }

    /// Writes the bytes to a byte stream.
    #[cfg(feature = "std")]
    pub fn write_bytes<W: std::io::Write>(&self, w: W) -> Result<()> {
        match self.version {
            Version::V0 => self.hash.write(w)?,
            Version::V1 => self.write_bytes_v1(w)?,
        }
        Ok(())
    }

    /// Returns the encoded bytes of the `Cid`.
    #[cfg(feature = "std")]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        self.write_bytes(&mut bytes).unwrap();
        bytes
    }

    #[cfg(feature = "std")]
    fn to_string_v0(&self) -> String {
        multibase::Base::Base58Btc.encode(self.hash.to_bytes())
    }

    #[cfg(feature = "std")]
    fn to_string_v1(&self) -> String {
        multibase::encode(multibase::Base::Base32Lower, self.to_bytes().as_slice())
    }

    /// Convert CID into a multibase encoded string
    ///
    /// # Example
    ///
    /// ```
    /// use cid::{RAW, Cid};
    /// use multibase::Base;
    /// use multihash::{SHA2_256, Multihash, MultihashDigest};
    ///
    /// let mh = Multihash::new(SHA2_256, b"foo").unwrap();
    /// let cid = Cid::new_v1(RAW, mh.to_raw().unwrap());
    /// let encoded = cid.to_string_of_base(Base::Base64).unwrap();
    /// assert_eq!(encoded, "mAVUSICwmtGto/8aP+ZtFPB0wQTQTQi1wZIO/oPmKXohiZueu");
    /// ```
    #[cfg(feature = "std")]
    pub fn to_string_of_base(&self, base: multibase::Base) -> Result<String> {
        match self.version {
            Version::V0 => {
                if base == multibase::Base::Base58Btc {
                    Ok(self.to_string_v0())
                } else {
                    Err(Error::InvalidCidV0Base)
                }
            }
            Version::V1 => Ok(multibase::encode(base, self.to_bytes())),
        }
    }
}

#[cfg(feature = "std")]
#[allow(clippy::derive_hash_xor_eq)]
impl core::hash::Hash for Cid {
    fn hash<T: core::hash::Hasher>(&self, state: &mut T) {
        self.to_bytes().hash(state);
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for Cid {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let output = match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(),
        };
        write!(f, "{}", output)
    }
}

#[cfg(feature = "std")]
impl core::str::FromStr for Cid {
    type Err = Error;

    fn from_str(cid_str: &str) -> Result<Self> {
        Self::try_from(cid_str)
    }
}

#[cfg(feature = "std")]
impl TryFrom<String> for Cid {
    type Error = Error;

    fn try_from(cid_str: String) -> Result<Self> {
        Self::try_from(cid_str.as_str())
    }
}

#[cfg(feature = "std")]
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
            multibase::Base::Base58Btc.decode(hash)?
        } else {
            let (_, decoded) = multibase::decode(hash)?;
            decoded
        };

        Self::try_from(decoded)
    }
}

#[cfg(feature = "std")]
impl TryFrom<Vec<u8>> for Cid {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        Self::try_from(bytes.as_slice())
    }
}

#[cfg(feature = "std")]
impl TryFrom<&[u8]> for Cid {
    type Error = Error;

    fn try_from(mut bytes: &[u8]) -> Result<Self> {
        Self::read_bytes(&mut bytes)
    }
}

impl From<&Cid> for Cid {
    fn from(cid: &Cid) -> Self {
        cid.clone()
    }
}

#[cfg(feature = "std")]
impl From<Cid> for Vec<u8> {
    fn from(cid: Cid) -> Self {
        cid.to_bytes()
    }
}

#[cfg(feature = "std")]
impl From<Cid> for String {
    fn from(cid: Cid) -> Self {
        cid.to_string()
    }
}
