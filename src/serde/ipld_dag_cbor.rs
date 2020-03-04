use serde::{de, ser, Deserialize, Serialize};
use serde_bytes::{ByteBuf, Bytes};
use serde_cbor::tags::Tagged;

use crate::cid::Cid;

/// Raw binary multibase identity
const RAW_BINARY_MULTIBASE_IDENTITY: u8 = 0;
/// The specific CBOR tag for IPLD DagCBOR serialization/deserialization
const CBOR_TAG_CID: u64 = 42;

/// IPLD DagCBOR serialization.
pub fn serialize<S>(cid: &Cid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    let mut bytes = cid.to_bytes();
    bytes.insert(0, RAW_BINARY_MULTIBASE_IDENTITY);

    let value = Bytes::new(&bytes);
    Tagged::new(Some(CBOR_TAG_CID), value).serialize(serializer)
}

/// IPLD DagCBOR deserialization.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Cid, D::Error>
where
    D: de::Deserializer<'de>,
{
    let tagged = Tagged::<ByteBuf>::deserialize(deserializer)?;
    match tagged.tag {
        Some(CBOR_TAG_CID) | None => {
            let bytes = tagged.value.into_vec();

            if bytes.is_empty() || bytes[0] != RAW_BINARY_MULTIBASE_IDENTITY {
                return Err(de::Error::custom(
                    "raw binary multibase identity 0x00 must not be omitted",
                ));
            }

            Ok(Cid::from(&bytes[1..]).map_err(|e| de::Error::custom(e.to_string()))?)
        }
        Some(_) => Err(de::Error::custom("unexpected CBOR tag")),
    }
}

#[cfg(test)]
mod tests {
    use serde_derive::{Deserialize, Serialize};

    use crate::cid::Cid;
    use crate::codec::Codec;
    use crate::version::Version;

    #[derive(Serialize, Deserialize)]
    struct TestCborCid(#[serde(with = "super")] Cid);

    #[test]
    fn serde_for_cid_v0() {
        let cid = "Qmf5Qzp6nGBku7CEn2UQx4mgN8TW69YUok36DrGa6NN893"
            .parse::<Cid>()
            .unwrap();
        assert_eq!(cid.version, Version::V0);
        assert_eq!(cid.codec, Codec::DagProtobuf);
        assert_eq!(
            cid.hash.to_vec(),
            vec![
                18, 32, 248, 175, 118, 33, 111, 145, 175, 205, 162, 241, 159, 194, 73, 247, 191,
                123, 200, 8, 195, 247, 188, 251, 25, 128, 235, 202, 135, 150, 161, 75, 202, 70
            ]
        );

        let cbor_cid = TestCborCid(cid.clone());
        let cbor = serde_cbor::to_vec(&cbor_cid).unwrap();
        assert_eq!(
            cbor,
            vec![
                216, 42, 88, 35, 0, 18, 32, 248, 175, 118, 33, 111, 145, 175, 205, 162, 241, 159,
                194, 73, 247, 191, 123, 200, 8, 195, 247, 188, 251, 25, 128, 235, 202, 135, 150,
                161, 75, 202, 70
            ]
        );

        let out: TestCborCid = serde_cbor::from_slice(&cbor).unwrap();
        assert_eq!(out.0, cid);
    }

    #[test]
    fn serde_for_cid_v1() {
        let cid = "bafkreie5qrjvaw64n4tjm6hbnm7fnqvcssfed4whsjqxzslbd3jwhsk3mm"
            .parse::<Cid>()
            .unwrap();
        assert_eq!(cid.version, Version::V1);
        assert_eq!(cid.codec, Codec::Raw);
        assert_eq!(
            cid.hash.to_vec(),
            vec![
                18, 32, 157, 132, 83, 80, 91, 220, 111, 38, 150, 120, 225, 107, 62, 86, 194, 162,
                148, 138, 65, 242, 199, 146, 97, 124, 201, 97, 30, 211, 99, 201, 91, 99
            ]
        );

        let cbor_cid = TestCborCid(cid.clone());
        let cbor = serde_cbor::to_vec(&cbor_cid).unwrap();
        assert_eq!(
            cbor,
            vec![
                216, 42, 88, 37, 0, 1, 85, 18, 32, 157, 132, 83, 80, 91, 220, 111, 38, 150, 120,
                225, 107, 62, 86, 194, 162, 148, 138, 65, 242, 199, 146, 97, 124, 201, 97, 30, 211,
                99, 201, 91, 99
            ]
        );

        let out: TestCborCid = serde_cbor::from_slice(&cbor).unwrap();
        assert_eq!(out.0, cid);
    }
}
