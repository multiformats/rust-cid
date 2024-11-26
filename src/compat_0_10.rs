//! This module implements [`TryFrom`] trait for converting between [`crate::Cid`]
//! and [`cid_0_10::Cid`]

impl TryFrom<crate::Cid> for cid_0_10::Cid {
    type Error = cid_0_10::Error;

    fn try_from(value: crate::Cid) -> Result<Self, Self::Error> {
        let bytes = value.to_bytes();
        Self::read_bytes(bytes.as_slice())
    }
}

impl TryFrom<cid_0_10::Cid> for crate::Cid {
    type Error = crate::Error;

    fn try_from(value: cid_0_10::Cid) -> Result<Self, Self::Error> {
        let bytes = value.to_bytes();
        Self::read_bytes(bytes.as_slice())
    }
}

#[cfg(all(test, feature = "arb"))]
mod tests {
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn to_old_cid(cid: crate::Cid) {
        let other: cid_0_10::Cid = cid.try_into().unwrap();
        assert_eq!(cid.to_bytes(), other.to_bytes());
    }

    #[quickcheck]
    fn from_old_cid(cid: cid_0_10::Cid) {
        let other: crate::Cid = cid.try_into().unwrap();
        assert_eq!(cid.to_bytes(), other.to_bytes());
    }
}
