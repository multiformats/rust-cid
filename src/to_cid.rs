use core::convert::TryFrom;

use multibase::Base;
use multihash::{self, MultihashRef};
use unsigned_varint::decode as varint_decode;

use crate::cid::Cid;
use crate::codec::Codec;
use crate::error::{Error, Result};
use crate::version::Version;

impl TryFrom<String> for Cid {
    type Error = Error;

    fn try_from(cid_str: String) -> Result<Self> {
        Self::try_from(cid_str.as_str())
    }
}

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
            Base::Base58Btc.decode(hash)?
        } else {
            let (_, decoded) = multibase::decode(hash)?;
            decoded
        };

        Self::try_from(decoded)
    }
}

impl TryFrom<Vec<u8>> for Cid {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for Cid {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if Version::is_v0_binary(bytes) {
            let mh = MultihashRef::from_slice(bytes)?.to_owned();
            Ok(Cid::new(Version::V0, Codec::DagProtobuf, mh))
        } else {
            let (raw_version, remain) = varint_decode::u64(&bytes)?;
            let version = Version::from(raw_version)?;

            let (raw_codec, hash) = varint_decode::u64(&remain)?;
            let codec = Codec::from(raw_codec)?;

            let mh = MultihashRef::from_slice(hash)?.to_owned();

            Ok(Cid::new(version, codec, mh))
        }
    }
}
