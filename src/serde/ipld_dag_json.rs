use std::collections::BTreeMap;

use serde::{de, ser, Deserialize, Serialize};
use serde_json::json;

use crate::cid::Cid;

/// IPLD DagJSON serialization.
pub fn serialize<S>(cid: &Cid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    let value = json!({ "/": cid.to_string() });
    value.serialize(serializer)
}

/// IPLD DagJSON deserialization.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Cid, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = BTreeMap::<String, String>::deserialize(deserializer)?;
    if let Some(cid) = value.get("/") {
        Ok(Cid::from(cid.as_str()).map_err(|e| de::Error::custom(e.to_string()))?)
    } else {
        Err(de::Error::custom("unexpected JSON object key"))
    }
}

#[cfg(test)]
mod tests {
    use serde_derive::{Deserialize, Serialize};

    use crate::cid::Cid;
    use crate::codec::Codec;
    use crate::version::Version;

    #[derive(Serialize, Deserialize)]
    struct TestJsonCid(#[serde(with = "super")] Cid);

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

        let json_cid = TestJsonCid(cid.clone());
        let json = serde_json::to_string(&json_cid).unwrap();
        assert_eq!(
            json,
            "{\"/\":\"Qmf5Qzp6nGBku7CEn2UQx4mgN8TW69YUok36DrGa6NN893\"}"
        );

        let out: TestJsonCid = serde_json::from_str(&json).unwrap();
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

        let json_cid = TestJsonCid(cid.clone());
        let json = serde_json::to_string(&json_cid).unwrap();
        assert_eq!(
            json,
            "{\"/\":\"bafkreie5qrjvaw64n4tjm6hbnm7fnqvcssfed4whsjqxzslbd3jwhsk3mm\"}"
        );

        let out: TestJsonCid = serde_json::from_str(&json).unwrap();
        assert_eq!(out.0, cid);
    }
}
