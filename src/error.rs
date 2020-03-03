use std::{error, fmt};

pub type Result<T> = std::result::Result<T, Error>;

/// Error types
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Error {
    UnknownCodec,
    InputTooShort,
    ParsingError,
    InvalidCidVersion,
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

impl From<multihash::EncodeError> for Error {
    fn from(_: multihash::EncodeError) -> Error {
        Error::ParsingError
    }
}

impl From<multihash::DecodeError> for Error {
    fn from(_: multihash::DecodeError) -> Error {
        Error::ParsingError
    }
}

impl From<multihash::DecodeOwnedError> for Error {
    fn from(_: multihash::DecodeOwnedError) -> Error {
        Error::ParsingError
    }
}

impl From<unsigned_varint::decode::Error> for Error {
    fn from(_: unsigned_varint::decode::Error) -> Self {
        Error::VarIntDecodeError
    }
}
