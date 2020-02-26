use std::{error, fmt};

/// Type alias to use this library's [`Error`] type in a `Result`.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Error {
    /// Unknown CID codec.
    UnknownCodec(u64),
    /// Invalid CID version.
    InvalidCidVersion(u64),
    /// Input data is too short.
    InputTooShort,
    /// Multibase parsing failure.
    ParseBaseError,
    /// Multihash parseing failure.
    ParseHashError,
    /// Unsigned varint decode failure.
    UnsignedVarIntDecodeError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnknownCodec(codec) => write!(f, "Unknown codec: {}", codec),
            Error::InvalidCidVersion(version) => write!(f, "Unrecognized CID version: {}", version),
            Error::InputTooShort => f.write_str("Input too short"),
            Error::ParseBaseError => f.write_str("Failed to parse multibase"),
            Error::ParseHashError => f.write_str("Failed to parse multihase"),
            Error::UnsignedVarIntDecodeError => {
                f.write_str("Failed to decode unsigned varint format")
            }
        }
    }
}

impl error::Error for Error {}

impl From<multibase::Error> for Error {
    fn from(_: multibase::Error) -> Error {
        Error::ParseBaseError
    }
}

impl From<multihash::DecodeError> for Error {
    fn from(_: multihash::DecodeError) -> Error {
        Error::ParseHashError
    }
}

impl From<unsigned_varint::decode::Error> for Error {
    fn from(_: unsigned_varint::decode::Error) -> Self {
        Error::UnsignedVarIntDecodeError
    }
}
