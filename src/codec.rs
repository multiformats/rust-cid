use error::{Error, Result};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Codec {
    Raw,
    DagProtobuf,
    DagCBOR,
    EthereumBlock,
    EthereumTx,
    BitcoinBlock,
    BitcoinTx,
    ZcashBlock,
    ZcashTx,
}

use Codec::*;

impl Codec {
    /// Convert a number to the matching codec
    pub fn from(raw: u64) -> Result<Codec> {
        Ok(match raw {
            0x55 => Raw,
            0x70 => DagProtobuf,
            0x71 => DagCBOR,
            0x90 => EthereumBlock,
            0x91 => EthereumTx,
            0xb0 => BitcoinBlock,
            0xb1 => BitcoinTx,
            0xc0 => ZcashBlock,
            0xc1 => ZcashTx,
            _ => return Err(Error::UnkownCodec),
        })
    }
}

/// Convert to the matching integer code
impl From<Codec> for u64 {
    fn from(codec: Codec) -> u64 {
        match codec {
            Raw => 0x55,
            DagProtobuf => 0x70,
            DagCBOR => 0x71,
            EthereumBlock => 0x90,
            EthereumTx => 0x91,
            BitcoinBlock => 0xb0,
            BitcoinTx => 0xb1,
            ZcashBlock => 0xc0,
            ZcashTx => 0xc1,
        }
    }
}
