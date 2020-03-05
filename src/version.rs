use crate::error::{Error, Result};

/// The version of the CID.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Version {
    /// CID version 0.
    V0,
    /// CID version 1.
    V1,
}

impl Version {
    /// Convert a number to the matching version, or `Error` if no valid version is matching.
    pub fn from(raw: u64) -> Result<Version> {
        match raw {
            0 => Ok(Self::V0),
            1 => Ok(Self::V1),
            _ => Err(Error::InvalidCidVersion),
        }
    }

    /// Check if the version of `data` string is CIDv0.
    pub fn is_v0_str(data: &str) -> bool {
        // v0 is a Base58Btc encoded sha hash, so it has
        // fixed length and always begins with "Qm"
        data.len() == 46 && data.starts_with("Qm")
    }

    /// Check if the version of `data` bytes is CIDv0.
    pub fn is_v0_binary(data: &[u8]) -> bool {
        data.len() == 34 && data.starts_with(&[0x12, 0x20])
    }
}

impl From<Version> for u64 {
    fn from(ver: Version) -> u64 {
        match ver {
            Version::V0 => 0,
            Version::V1 => 1,
        }
    }
}
