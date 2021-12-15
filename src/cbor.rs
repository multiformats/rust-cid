use serde::{de, ser};
use serde_cbor::tags::Tagged;
use std::convert::TryFrom;

use crate::Cid;

const MULTIBASE_IDENTITY: u8 = 0;
const CBOR_TAG_CID: u64 = 42;

impl ser::Serialize for Cid {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut cid_bytes = self.to_bytes();

        // or for all Cid bytes (byte is irrelevant and redundant)
        cid_bytes.insert(0, MULTIBASE_IDENTITY);

        let value = serde_bytes::Bytes::new(&cid_bytes);
        Tagged::new(Some(CBOR_TAG_CID), &value).serialize(s)
    }
}

impl<'de> de::Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let tagged = Tagged::<serde_bytes::ByteBuf>::deserialize(deserializer)?;
        match tagged.tag {
            Some(CBOR_TAG_CID) | None => {
                let mut bz = tagged.value.into_vec();

                if bz.first() == Some(&MULTIBASE_IDENTITY) {
                    bz.remove(0);
                }

                Ok(Cid::try_from(bz)
                    .map_err(|e| de::Error::custom(format!("Failed to deserialize Cid: {}", e)))?)
            }
            Some(_) => Err(de::Error::custom("unexpected tag")),
        }
    }
}

#[cfg(test)]
mod test {
    use multihash::{Code, MultihashDigest};
    #[test]
    #[cfg(feature = "cbor")]
    fn test_cid_serde() {
        use super::Cid;

        let cid = Cid::new_v1(0x55, Code::Sha2_256.digest(&*b"foobar"));
        let cid_bytes = cid.to_bytes();

        let bytes = serde_cbor::to_vec(&cid).unwrap();
        let cid2 = serde_cbor::from_slice(&bytes).unwrap();
        assert_eq!(cid, cid2);

        let mut expected: Vec<u8> = vec![
            // tag
            6 << 5 | 24,
            42,
            // array
            2 << 5 | 24,
            (cid_bytes.len() + 1) as u8,
            // leading 0
            0,
        ];
        expected.extend_from_slice(&cid_bytes);
        assert_eq!(expected, bytes);
    }
}
