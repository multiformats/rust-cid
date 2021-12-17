//! CID Serde (de)serialization for the IPLD Data Model.
//!
//! CIDs cannot directly be represented in any of the native Serde Data model types. In order to
//! work around that limitation. a newtype struct is introduced, that is used as a marker for Serde
//! (de)serialization.
use std::convert::TryFrom;
use std::fmt;

use serde::{de, ser};

use crate::Cid;

/// The newtype struct name that is used by Serde internally.
pub const CID_SERDE_NEWTYPE_STRUCT_NAME: &str = "$__serde__newtype__struct__for__cid";

impl ser::Serialize for Cid {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let value = serde_bytes::ByteBuf::from(self.to_bytes());
        s.serialize_newtype_struct(CID_SERDE_NEWTYPE_STRUCT_NAME, &value)
    }
}

/// Visitor to deserialize a CID that is wrapped in a new type struct named as defined at
/// [`CID_SERDE_NEWTYPE_STRUCT_NAME`].
pub struct CidVisitor;

impl<'de> de::Visitor<'de> for CidVisitor {
    type Value = Cid;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "a valid CID in bytes, wrapped in a new struct")
    }

    /// Define `visit_newtype_struct` so that we have an entry-point from the seserializer to pass
    /// in a custom deserializer just for CIDs.
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(self)
    }

    /// Some Serde data formats interpret a byte stream as a sequence of bytes (e.g. `serde_json`).
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut bytes = Vec::new();
        while let Some(byte) = seq.next_element()? {
            bytes.push(byte);
        }
        Cid::try_from(bytes)
            .map_err(|err| de::Error::custom(format!("Failed to deserialize CID: {}", err)))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Cid::try_from(value)
            .map_err(|err| de::Error::custom(format!("Failed to deserialize CID: {}", err)))
    }
}

impl<'de> de::Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(CID_SERDE_NEWTYPE_STRUCT_NAME, CidVisitor)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::Cid;

    #[test]
    fn test_cid_serde() {
        let cid =
            Cid::try_from("bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy").unwrap();
        let bytes = serde_json::to_string(&cid).unwrap();
        let cid2 = serde_json::from_str(&bytes).unwrap();
        assert_eq!(cid, cid2);
    }
}
