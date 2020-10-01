use std::{error, fmt};

/// Type alias to use this library's [`Error`] type in a `Result`.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Error {
    /// Unknown CID codec.
    UnknownCodec,
    /// Input data is too short.
    InputTooShort,
    /// Multibase or multihash codec failure
    ParsingError,
    /// Invalid CID version.
    InvalidCidVersion,
    /// Invalid CIDv0 codec.
    InvalidCidV0Codec,
    /// Invalid CIDv0 multihash.
    InvalidCidV0Multihash,
    /// Invalid CIDv0 base encoding.
    InvalidCidV0Base,
    /// Varint decode failure.
    VarIntDecodeError,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        let error = match *self {
            UnknownCodec => "Unknown codec",
            InputTooShort => "Input too short",
            ParsingError => "Failed to parse multihash",
            InvalidCidVersion => "Unrecognized CID version",
            InvalidCidV0Codec => "CIDv0 requires a DagPB codec",
            InvalidCidV0Multihash => "CIDv0 requires a Sha-256 multihash",
            InvalidCidV0Base => "CIDv0 requires a Base58 base",
            VarIntDecodeError => "Failed to decode unsigned varint format",
        };

        f.write_str(error)
    }
}

impl From<multibase::Error> for Error {
    fn from(_: multibase::Error) -> Error {
        Error::ParsingError
    }
}

impl From<multihash::Error> for Error {
    fn from(_: multihash::Error) -> Error {
        Error::ParsingError
    }
}

impl From<unsigned_varint::decode::Error> for Error {
    fn from(_: unsigned_varint::decode::Error) -> Self {
        Error::VarIntDecodeError
    }
}
