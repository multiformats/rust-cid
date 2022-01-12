//! CID Serde (de)serialization for the IPLD Data Model.
//!
//! CIDs cannot directly be represented in any of the native Serde Data model types. In order to
//! work around that limitation. a newtype struct is introduced, that is used as a marker for Serde
//! (de)serialization.
use std::convert::TryFrom;
use std::fmt;

use serde::{de, ser};

use crate::Cid;

/// An identifier that is used internally by Serde implementations that support [`Cid`]s.
pub const CID_SERDE_PRIVATE_IDENTIFIER: &str = "$__private__serde__identifier__for__cid";
///// dfsdf
//pub const CID_SERDE_PRIVATE_IDENTIFIER_VARIANT: &str = "$__private__variant";

/// Serialize a CID into the Serde data model as enum.
///
/// Custom types are not supported by Serde, hence we map a CID into an enum that can be identified
/// as a CID by implementations that support CIDs. The corresponding Rust type would be:
///
/// ```
/// enum $__private__serde__identifier__for__cid {
///     $__private__serde__identifier__for__cid(serde_bytes::BytesBuf)
/// }
/// ```
impl ser::Serialize for Cid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let value = serde_bytes::ByteBuf::from(self.to_bytes());
        //serializer.serialize_newtype_struct(CID_SERDE_PRIVATE_IDENTIFIER, &value)
        serializer.serialize_newtype_variant(CID_SERDE_PRIVATE_IDENTIFIER, 0, CID_SERDE_PRIVATE_IDENTIFIER, &value)
    }
}

//struct InternalVisitor;
//
//impl<'de> de::Visitor<'de> for InternalVisitor {
//    type Value = Cid;
//
//    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
//        write!(fmt, "a valid CID in bytes")
//    }
//
//    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
//    where
//        E: de::Error,
//    {
//        println!("vmx: rust cid: serde: de: internal visitor: bytes");
//        Cid::try_from(value)
//            .map_err(|err| de::Error::custom(format!("Failed to deserialize CID: {}", err)))
//    }
//}

/// Visitor to deserialize a CID that is wrapped in a new type struct named as defined at
/// [`CID_SERDE_NEWTYPE_STRUCT_NAME`].
pub struct CidVisitor;

impl<'de> de::Visitor<'de> for CidVisitor {
    type Value = Cid;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "a valid CID in bytes, wrapped in an enum")
    }

    ///// Define `visit_newtype_struct` so that we have an entry-point from the seserializer to pass
    ///// in a custom deserializer just for CIDs.
    //fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    //where
    //    D: de::Deserializer<'de>,
    //{
    //    deserializer.deserialize_bytes(self)
    //    //deserializer.deserialize_bytes(InternalVisitor)
    //}
    
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error> where
        A: de::EnumAccess<'de> {
            //println!("vmx: rust cid: de visit enum");
            //let (variant , _): (Self::Value, _) = data.variant()?;
            //println!("vmx: rust cid: de visit enum variant: {:?}", variant);
            ////if let Ok((variant, whatisthis)) = data.variant() {
            ////println!("vmx: rust cid: de visit enum: variant: {:#?}", variant);
            ////    Ok(variant)
            ////} else {
            ////println!("vmx: rust cid: de visit enum: error");
            //   Err(de::Error::custom("invalid enum TODO vmx 2022-01-12: better error message"))
            ////}


        println!("vmx: rust cid: de visit enum");
        match data.variant() {
            Ok((CID_SERDE_PRIVATE_IDENTIFIER, variant)) => {
                println!("vmx: that works?");
                let bytes = de::VariantAccess::newtype_variant::<Vec<u8>>(variant).unwrap();
                Ok(Cid::try_from(bytes).unwrap())
                //de::VariantAccess::newtype_variant(variant)
            },
            _ => Err(de::Error::custom("Cannot deserialize CID")),
        }

    }

    ///// Some Serde data formats interpret a byte stream as a sequence of bytes (e.g. `serde_json`).
    //fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    //where
    //    A: de::SeqAccess<'de>,
    //{
    //    let mut bytes = Vec::new();
    //    while let Some(byte) = seq.next_element()? {
    //        bytes.push(byte);
    //    }
    //    Cid::try_from(bytes)
    //        .map_err(|err| de::Error::custom(format!("Failed to deserialize CID: {}", err)))
    //}
    //
    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
       E: de::Error,
    {
       println!("vmx: rust cid: serde: de: visitor: bytes");
       Cid::try_from(value)
           .map_err(|err| de::Error::custom(format!("Failed to deserialize CID: {}", err)))
    }
}

//#[derive(Debug)]
//enum Error {
//    Cid
//}
//
//impl fmt::Display for Error {
//    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//        match self {
//            Error::Cid => formatter.write_str("not a CID"),
//        }
//    }
//}
//
//impl std::error::Error for Error {}
//
//impl ser::Error for Error {
//    fn custom<T: fmt::Display>(msg: T) -> Self {
//        //Error::Message(msg.to_string())
//        Self::Cid
//    }
//}
//
//impl de::Error for Error {
//    fn custom<T: fmt::Display>(msg: T) -> Self {
//        //Error::Message(msg.to_string())
//        Self::Cid
//    }
//}
//
//struct Enum<'a, 'de: 'a>(&'a mut Deserializer<'de>);
//
//
//impl<'de> de::EnumAccess<'de> for Enum {
//    type Error = Error;
//    //type Variant = VariantDeserializer;
//    type Variant = Self;
//
//    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
//    where
//        V: de::DeserializeSeed<'de>,
//    {
//        //let variant = self.variant.into_deserializer();
//        //let visitor = VariantDeserializer { value: self.value };
//        //seed.deserialize(variant).map(|v| (v, visitor))
//
//
//        let val = seed.deserialize(Enum)?;
//        // Parse the colon separating map key from value.
//        if self.de.next_char()? == ':' {
//            Ok((val, self))
//        } else {
//            Err(Error::ExpectedMapColon)
//        }
//    }
//}
//
//impl<'de, 'a> de::VariantAccess<'de> for Enum {
//    type Error = Error;
//
//    // If the `Visitor` expected this variant to be a unit variant, the input
//    // should have been the plain string case handled in `deserialize_enum`.
//    fn unit_variant(self) -> Result<(), Self::Error> {
//        Err(Error::Cid)
//    }
//
//    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
//    // deserialize the value here.
//    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
//    where
//        T: de::DeserializeSeed<'de>,
//    {
//        //seed.deserialize(self.de)
//        Err(Error::Cid)
//    }
//
//    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
//    // deserialize the sequence of data here.
//    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
//    where
//        V: de::Visitor<'de>,
//    {
//        //de::Deserializer::deserialize_seq(self.de, visitor)
//        Err(Error::Cid)
//    }
//
//    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
//    // deserialize the inner map here.
//    fn struct_variant<V>(
//        self,
//        _fields: &'static [&'static str],
//        visitor: V,
//    ) -> Result<V::Value, Self::Error>
//    where
//        V: de::Visitor<'de>,
//    {
//        //de::Deserializer::deserialize_map(self.de, visitor)
//        Err(Error::Cid)
//    }
//}





/// Deserialize a CID from our custom Serde data model enum serialization
///
/// Deserialize a CID that was serialized as an enum that can be identified as a CID. Its
/// corresponding Rust type would be:
///
/// ```
/// enum $__private__serde__identifier__for__cid {
///     $__private__serde__identifier__for__cid(serde_bytes::BytesBuf)
/// }
/// ```
impl<'de> de::Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        println!("vmx: rust cid: deserialize");
        //deserializer.deserialize_newtype_struct(CID_SERDE_PRIVATE_IDENTIFIER, CidVisitor)
        deserializer.deserialize_enum(CID_SERDE_PRIVATE_IDENTIFIER, &[CID_SERDE_PRIVATE_IDENTIFIER], CidVisitor)
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
