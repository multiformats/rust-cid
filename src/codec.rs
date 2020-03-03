use crate::error::{Error, Result};

macro_rules! build_codec_enum {
    {$( #[$attr:meta] $code:expr => $codec:ident, )*} => {
        /// List of types currently supported in the multicodec spec.
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
        pub enum Codec {
            $( #[$attr] $codec, )*
        }

        impl Codec {
            /// Convert a number to the matching codec, or `Error` if unknown codec is matching.
            pub fn from(raw: u64) -> Result<Codec> {
                match raw {
                    $( $code => Ok(Self::$codec), )*
                    _ => Err(Error::UnknownCodec),
                }
            }
        }

        impl From<Codec> for u64 {
            /// Convert to the matching integer code
            fn from(codec: Codec) -> u64 {
                match codec {
                    $( Codec::$codec => $code, )*
                }
            }
        }
    }
}

build_codec_enum! {
    /// Raw binary
    0x55 => Raw,
    /// MerkleDAG protobuf
    0x70 => DagProtobuf,
    /// MerkleDAG cbor
    0x71 => DagCBOR,
    /// Raw Git object
    0x78 => GitRaw,
    /// Ethereum Block (RLP)
    0x90 => EthereumBlock,
    /// Ethereum Block List (RLP)
    0x91 => EthereumBlockList,
    /// Ethereum Transaction Trie (Eth-Trie)
    0x92 => EthereumTxTrie,
    /// Ethereum Transaction (RLP)
    0x93 => EthereumTx,
    /// Ethereum Transaction Receipt Trie (Eth-Trie)
    0x94 => EthereumTxReceiptTrie,
    /// Ethereum Transaction Receipt (RLP)
    0x95 => EthereumTxReceipt,
    /// Ethereum State Trie (Eth-Secure-Trie)
    0x96 => EthereumStateTrie,
    /// Ethereum Account Snapshot (RLP)
    0x97 => EthereumAccountSnapshot,
    /// Ethereum Contract Storage Trie (Eth-Secure-Trie)
    0x98 => EthereumStorageTrie,
    /// Bitcoin Block
    0xb0 => BitcoinBlock,
    /// Bitcoin Transaction
    0xb1 => BitcoinTx,
    /// Zcash Block
    0xc0 => ZcashBlock,
    /// Zcash Transaction
    0xc1 => ZcashTx,
    /// MerkleDAG json
    0x0129 => DagJSON,
}
