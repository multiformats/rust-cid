//! This module contains the main CID type.
//!
//! If you are an application developer you likely won't use the `Cid` which is generic over the
//! digest size. Intead you would use the concrete top-level `Cid` type.
//!
//! As a library author that works with CIDs that should support hashes of anysize, you would
//! import the `Cid` type from this module.
use core::convert::TryFrom;

#[cfg(feature = "alloc")]
use multibase::{encode as base_encode, Base};

use multihash::MultihashGeneric as Multihash;
use unsigned_varint::encode as varint_encode;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{
  borrow,
  string::{String, ToString},
  vec::Vec,
};

#[cfg(feature = "std")]
pub(crate) use unsigned_varint::io::read_u64 as varint_read_u64;

/// Reads 64 bits from a byte array into a u64
/// Adapted from unsigned-varint's generated read_u64 function at
/// https://github.com/paritytech/unsigned-varint/blob/master/src/io.rs
#[cfg(not(feature = "std"))]
pub(crate) fn varint_read_u64<R: io::Read>(mut r: R) -> Result<u64> {
  use unsigned_varint::decode;
  let mut b = varint_encode::u64_buffer();
  for i in 0..b.len() {
    let n = r.read(&mut (b[i..i + 1]))?;
    if n == 0 {
      return Err(Error::VarIntDecodeError);
    } else if decode::is_last(b[i]) {
      return Ok(decode::u64(&b[..=i]).unwrap().0);
    }
  }
  Err(Error::VarIntDecodeError)
}

#[cfg(feature = "std")]
use std::io;

#[cfg(not(feature = "std"))]
use core2::io;

use crate::error::{Error, Result};
use crate::version::Version;

/// DAG-PB multicodec code
const DAG_PB: u64 = 0x70;
/// The SHA_256 multicodec code
const SHA2_256: u64 = 0x12;

/// Representation of a CID.
///
/// The generic is about the allocated size of the multihash.
#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode))]
pub enum Cid<const S: usize, const M: usize> {
  /// A CidV0 is a SHA2_256 Multihash of DAG_PB data
  CidV0 {
    /// A SHA2_256 Multihash of DAG_PB data
    hash: Multihash<S>,
  },
  /// A CidV1 is a generic Multihash prepended with a Multicodec descriptor
  CidV1 {
    /// A data Multicodec format
    codec: u64,
    /// A data Multihash pointer
    hash: Multihash<S>,
  },
  /// A CidV2 is two Multicodec-Multihash pairs, with the first indicating the
  /// data hash, and the second indicating the metadata hash
  CidV2 {
    /// A data Multicodec format
    codec: u64,
    /// A data Multihash pointer
    hash: Multihash<S>,
    /// A metadata Multicodec format
    meta_codec: u64,
    /// A metadata Multicodec pointer
    meta_hash: Multihash<M>,
  },
}

impl<const S: usize, const M: usize> Cid<S, M> {
  /// Create a new CIDv0.
  pub const fn new_v0(hash: Multihash<S>) -> Result<Self> {
    if hash.code() != SHA2_256 {
      return Err(Error::InvalidCidV0Multihash);
    }
    Ok(Self::CidV0 { hash })
  }

  /// Create a new CIDv1.
  pub const fn new_v1(codec: u64, hash: Multihash<S>) -> Self {
    Self::CidV1 { codec, hash }
  }

  /// Create a new CIDv2.
  pub const fn new_v2(
    codec: u64,
    hash: Multihash<S>,
    meta_codec: u64,
    meta_hash: Multihash<M>,
  ) -> Self {
    Self::CidV2 { codec, hash, meta_codec, meta_hash }
  }

  /// Returns the cid version.
  pub const fn version(&self) -> Version {
    match self {
      Self::CidV0 { .. } => Version::V0,
      Self::CidV1 { .. } => Version::V1,
      Self::CidV2 { .. } => Version::V2,
    }
  }

  /// Returns the cid codec.
  pub const fn codec(&self) -> u64 {
    match self {
      Self::CidV0 { .. } => DAG_PB,
      Self::CidV1 { codec, .. } => *codec,
      Self::CidV2 { codec, .. } => *codec,
    }
  }

  /// Returns the cid multihash.
  pub const fn hash(&self) -> &Multihash<S> {
    match self {
      Self::CidV0 { hash, .. } => hash,
      Self::CidV1 { hash, .. } => hash,
      Self::CidV2 { hash, .. } => hash,
    }
  }

  /// Reads the bytes from a byte stream.
  pub fn read_bytes<R: io::Read>(mut r: R) -> Result<Self> {
    let version = varint_read_u64(&mut r)?;
    let codec = varint_read_u64(&mut r)?;
    match Version::try_from(version)? {
      Version::V0 => {
        if codec != 0x20 {
          return Err(Error::InvalidCidV0Codec);
        }
        let mut digest = [0u8; 32];
        r.read_exact(&mut digest)?;
        let mh = Multihash::wrap(version, &digest)
          .expect("Digest is always 32 bytes.");
        Ok(Cid::CidV0 { hash: mh })
      }
      Version::V1 => {
        let mh = Multihash::read(r)?;
        Ok(Self::new_v1(codec, mh))
      }
      Version::V2 => {
        let data_mh = Multihash::read(&mut r)?;
        let meta_mc = varint_read_u64(&mut r)?;
        let meta_mh = Multihash::read(r)?;
        Ok(Self::new_v2(codec, data_mh, meta_mc, meta_mh))
      }
    }
  }

  /// Writes the bytes to a byte stream.
  pub fn write_bytes<W: io::Write>(&self, mut w: W) -> Result<()> {
    match self {
      Cid::CidV0 { hash } => {
        hash.write(w)?;
        Ok(())
      }
      Cid::CidV1 { codec, hash } => {
        let mut version_buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(Version::V1.into(), &mut version_buf);

        let mut codec_buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(*codec, &mut codec_buf);

        w.write_all(version)?;
        w.write_all(codec)?;
        hash.write(&mut w)?;
        Ok(())
      }
      Cid::CidV2 { codec, hash, meta_codec, meta_hash } => {
        let mut version_buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(Version::V2.into(), &mut version_buf);

        let mut codec_buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(*codec, &mut codec_buf);

        let mut meta_codec_buf = varint_encode::u64_buffer();
        let meta_codec = varint_encode::u64(*meta_codec, &mut meta_codec_buf);

        w.write_all(version)?;
        w.write_all(codec)?;
        hash.write(&mut w)?;
        w.write_all(meta_codec)?;
        meta_hash.write(&mut w)?;
        Ok(())
      }
    }
  }

  /// Returns the encoded bytes of the `Cid`.
  #[cfg(feature = "alloc")]
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    self.write_bytes(&mut bytes).unwrap();
    bytes
  }

  #[cfg(feature = "alloc")]
  #[allow(clippy::wrong_self_convention)]
  fn to_string_v0(&self) -> String {
    Base::Base58Btc.encode(self.hash().to_bytes())
  }

  #[cfg(feature = "alloc")]
  #[allow(clippy::wrong_self_convention)]
  fn to_string_v1(&self) -> String {
    multibase::encode(Base::Base32Lower, self.to_bytes().as_slice())
  }

  #[allow(clippy::wrong_self_convention)]
  fn to_string_v2(&self) -> String {
    multibase::encode(Base::Base32Lower, self.to_bytes().as_slice())
  }

  /// Convert CID into a multibase encoded string
  ///
  /// # Example
  ///
  /// ```
  /// use cid::Cid;
  /// use multibase::Base;
  /// use multihash::{Code, MultihashDigest};
  ///
  /// const RAW: u64 = 0x55;
  ///
  /// let cid = Cid::new_v1(RAW, Code::Sha2_256.digest(b"foo"));
  /// let encoded = cid.to_string_of_base(Base::Base64).unwrap();
  /// assert_eq!(encoded, "mAVUSICwmtGto/8aP+ZtFPB0wQTQTQi1wZIO/oPmKXohiZueu");
  /// ```
  #[cfg(feature = "alloc")]
  pub fn to_string_of_base(&self, base: Base) -> Result<String> {
    match self.version() {
      Version::V0 => {
        if base == Base::Base58Btc {
          Ok(self.to_string_v0())
        } else {
          Err(Error::InvalidCidV0Base)
        }
      }
      Version::V1 => Ok(base_encode(base, self.to_bytes())),
      Version::V2 => Ok(base_encode(base, self.to_bytes())),
    }
  }
}

impl<const S: usize, const M: usize> Default for Cid<S, M> {
  fn default() -> Self {
    Cid::CidV1 { codec: 0, hash: Multihash::<S>::default() }
  }
}

// TODO: remove the dependency on alloc by fixing
// https://github.com/multiformats/rust-multibase/issues/33
#[cfg(feature = "alloc")]
impl<const S: usize, const M: usize> core::fmt::Display for Cid<S, M> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    let output = match self.version() {
      Version::V0 => self.to_string_v0(),
      Version::V1 => self.to_string_v1(),
      Version::V2 => self.to_string_v2(),
    };
    write!(f, "{}", output)
  }
}

#[cfg(feature = "alloc")]
impl<const S: usize, const M: usize> core::fmt::Debug for Cid<S, M> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    if f.alternate() {
      f.debug_struct("Cid")
        .field("version", &self.version())
        .field("codec", &self.codec())
        .field("hash", (*self).clone().hash())
        .finish()
    } else {
      let output = match self.version() {
        Version::V0 => self.to_string_v0(),
        Version::V1 => self.to_string_v1(),
        Version::V2 => self.to_string_v2(),
      };
      write!(f, "Cid({})", output)
    }
  }
}

#[cfg(feature = "alloc")]
impl<const S: usize, const M: usize> core::str::FromStr for Cid<S, M> {
  type Err = Error;

  fn from_str(cid_str: &str) -> Result<Self> {
    Self::try_from(cid_str)
  }
}

#[cfg(feature = "alloc")]
impl<const S: usize, const M: usize> TryFrom<String> for Cid<S, M> {
  type Error = Error;

  fn try_from(cid_str: String) -> Result<Self> {
    Self::try_from(cid_str.as_str())
  }
}

#[cfg(feature = "alloc")]
impl<const S: usize, const M: usize> TryFrom<&str> for Cid<S, M> {
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

#[cfg(feature = "alloc")]
impl<const S: usize, const M: usize> TryFrom<Vec<u8>> for Cid<S, M> {
  type Error = Error;

  fn try_from(bytes: Vec<u8>) -> Result<Self> {
    Self::try_from(bytes.as_slice())
  }
}

impl<const S: usize, const M: usize> TryFrom<&[u8]> for Cid<S, M> {
  type Error = Error;

  fn try_from(mut bytes: &[u8]) -> Result<Self> {
    Self::read_bytes(&mut bytes)
  }
}

impl<const S: usize, const M: usize> From<&Cid<S, M>> for Cid<S, M> {
  fn from(cid: &Cid<S, M>) -> Self {
    *cid
  }
}

#[cfg(feature = "alloc")]
impl<const S: usize, const M: usize> From<Cid<S, M>> for Vec<u8> {
  fn from(cid: Cid<S, M>) -> Self {
    cid.to_bytes()
  }
}

#[cfg(feature = "alloc")]
impl<const S: usize, const M: usize> From<Cid<S, M>> for String {
  fn from(cid: Cid<S, M>) -> Self {
    cid.to_string()
  }
}

#[cfg(feature = "alloc")]
impl<'a, const S: usize, const M: usize> From<Cid<S, M>>
  for borrow::Cow<'a, Cid<S, M>>
{
  fn from(from: Cid<S, M>) -> Self {
    borrow::Cow::Owned(from)
  }
}

#[cfg(feature = "alloc")]
impl<'a, const S: usize, const M: usize> From<&'a Cid<S, M>>
  for borrow::Cow<'a, Cid<S, M>>
{
  fn from(from: &'a Cid<S, M>) -> Self {
    borrow::Cow::Borrowed(from)
  }
}

#[cfg(test)]
mod tests {
  #[test]
  #[cfg(feature = "scale-codec")]
  fn test_cid_scale_codec() {
    use super::Cid;
    use parity_scale_codec::{Decode, Encode};

    let cid = Cid::<64, 0>::default();
    let bytes = cid.encode();
    let cid2 = Cid::decode(&mut &bytes[..]).unwrap();
    assert_eq!(cid, cid2);
  }

  #[test]
  #[cfg(feature = "std")]
  fn test_debug_instance() {
    use super::Cid;
    use std::str::FromStr;
    let cid = Cid::<64, 0>::from_str(
      "bafyreibjo4xmgaevkgud7mbifn3dzp4v4lyaui4yvqp3f2bqwtxcjrdqg4",
    )
    .unwrap();
    // short debug
    assert_eq!(
      &format!("{:?}", cid),
      "Cid(bafyreibjo4xmgaevkgud7mbifn3dzp4v4lyaui4yvqp3f2bqwtxcjrdqg4)"
    );
    // verbose debug
    let mut txt = format!("{:#?}", cid);
    txt.retain(|c| !c.is_whitespace());
    assert_eq!(&txt, "Cid{version:V1,codec:113,hash:Multihash{code:18,size:32,digest:[41,119,46,195,0,149,81,168,63,176,40,43,118,60,191,149,226,240,10,35,152,172,31,178,232,48,180,238,36,196,112,55,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],},}");
  }
}
