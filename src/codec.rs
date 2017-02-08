use error::{Error, Result};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Codec {
    Raw,
    DagProtobuf,
    DagCBOR,
    EthereumBlock,
    EthereumBlockList,
    EthereumTxTrie,
    EthereumTx,
    EthereumTxReceiptTrie,
    EthereumTxReceipt,
    EthereumStateTrie,
    EthereumStorageTrie,
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
            0x91 => EthereumBlockList,
            0x92 => EthereumTxTrie,
            0x93 => EthereumTx,
            0x94 => EthereumTxReceiptTrie,
            0x95 => EthereumTxReceipt,
            0x96 => EthereumStateTrie,
            0x98 => EthereumStorageTrie,
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
            EthereumBlockList => 0x91,
            EthereumTxTrie => 0x92,
            EthereumTx => 0x93,
            EthereumTxReceiptTrie => 0x94,
            EthereumTxReceipt => 0x95,
            EthereumStateTrie => 0x96,
            EthereumStorageTrie => 0x98,
            BitcoinBlock => 0xb0,
            BitcoinTx => 0xb1,
            ZcashBlock => 0xc0,
            ZcashTx => 0xc1,
        }
    }
}
