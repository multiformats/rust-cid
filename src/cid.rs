use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::version::Version;
use core::convert::TryFrom;
use multihash::{MultihashCode, MultihashDigest};

/// Representation of a CID.
///
/// Usually you would use `Cid` instead, unless you have a custom Multihash code table
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    /// The version of CID.
    version: Version,
    /// The codec of CID.
    codec: C,
    /// The multihash of CID.
    hash: H::Multihash,
}

impl<C, H> Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    /// Create a new CIDv0.
    pub fn new_v0(hash: H::Multihash) -> Result<Self> {
        if hash.code().into() != 0x12 {
            return Err(Error::InvalidCidV0Multihash);
        }
        Ok(Self {
            version: Version::V0,
            // Convert the code of `DagProtobuf` into the given code table
            codec: C::try_from(Codec::DagProtobuf.into()).map_err(|_| Error::UnknownCodec)?,
            hash,
        })
    }

    /// Create a new CIDv1.
    pub fn new_v1(codec: C, hash: H::Multihash) -> Self {
        Self {
            version: Version::V1,
            codec,
            hash,
        }
    }

    /// Create a new CID.
    pub fn new(version: Version, codec: C, hash: H::Multihash) -> Result<Self> {
        match version {
            Version::V0 => {
                if codec.into() != u64::from(Codec::DagProtobuf) {
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
    pub fn codec(&self) -> C {
        self.codec
    }

    /// Returns the cid multihash.
    pub fn hash(&self) -> &H::Multihash {
        &self.hash
    }

    /// Reads the bytes from a byte stream.
    #[cfg(feature = "std")]
    pub fn read_bytes<R: std::io::Read>(mut r: R) -> Result<Self> {
        use unsigned_varint::io::read_u64;
        //if Version::is_v0_binary(bytes) {
        //    let mh = H::Multihash::read(r)?;
        //    Self::new_v0(mh)
        //} else {
        let version = Version::try_from(read_u64(&mut r)?)?;
        let codec = C::try_from(read_u64(&mut r)?).map_err(|_| Error::UnknownCodec)?;
        let mh = H::Multihash::read(r)?;
        Self::new(version, codec, mh)
        //}
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
    /// use cid::{Cid, Codec};
    /// use multibase::Base;
    /// use multihash::{Code, MultihashCode};
    ///
    /// let cid = Cid::<Codec, Code>::new_v1(Codec::Raw, Code::Sha2_256.digest(b"foo"));
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
impl<C, H> core::hash::Hash for Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    fn hash<T: core::hash::Hasher>(&self, state: &mut T) {
        self.to_bytes().hash(state);
    }
}

#[cfg(feature = "std")]
impl<C, H> core::fmt::Display for Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let output = match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(),
        };
        write!(f, "{}", output)
    }
}

#[cfg(feature = "std")]
impl<C, H> core::str::FromStr for Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    type Err = Error;

    fn from_str(cid_str: &str) -> Result<Self> {
        Self::try_from(cid_str)
    }
}

#[cfg(feature = "std")]
impl<C, H> TryFrom<String> for Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    type Error = Error;

    fn try_from(cid_str: String) -> Result<Self> {
        Self::try_from(cid_str.as_str())
    }
}

#[cfg(feature = "std")]
impl<C, H> TryFrom<&str> for Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
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
            multibase::Base::Base58Btc.decode(hash)?
        } else {
            let (_, decoded) = multibase::decode(hash)?;
            decoded
        };

        Self::try_from(decoded)
    }
}

#[cfg(feature = "std")]
impl<C, H> TryFrom<Vec<u8>> for Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        Self::try_from(bytes.as_slice())
    }
}

#[cfg(feature = "std")]
impl<C, H> TryFrom<&[u8]> for Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    type Error = Error;

    fn try_from(mut bytes: &[u8]) -> Result<Self> {
        Self::read_bytes(&mut bytes)
    }
}

impl<C, H> From<&Cid<C, H>> for Cid<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    fn from(cid: &Cid<C, H>) -> Self {
        cid.clone()
    }
}

#[cfg(feature = "std")]
impl<C, H> From<Cid<C, H>> for Vec<u8>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    fn from(cid: Cid<C, H>) -> Self {
        cid.to_bytes()
    }
}

#[cfg(feature = "std")]
impl<C, H> From<Cid<C, H>> for String
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    fn from(cid: Cid<C, H>) -> Self {
        cid.to_string()
    }
}
