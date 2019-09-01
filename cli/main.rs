use async_std::{
    io::{self, Read},
    task,
};
use core::{convert::TryFrom, fmt, str::FromStr};
use cid::Cid;
use exitfailure::ExitFailure;
use failure::{format_err, Error};
use multibase::Base;
use multihash::Multihash;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opts {
    /// The mode
    #[structopt(subcommand)]
    mode: Mode,
}

#[derive(StructOpt, Debug)]
enum Mode {
    #[structopt(name = "encode")]
    Encode {
        #[structopt(short = "v", long = "version", default_value = "v0")]
        version: Version,
        #[structopt(short = "c", long = "codec", default_value = "dag-pb")]
        codec: Codec,
    },
    #[structopt(name = "decode")]
    Decode,
}

fn main() -> Result<(), ExitFailure> {
    env_logger::init();
    task::block_on(async {
        let opts = Opts::from_args();
        match opts.mode {
            Mode::Encode { version, codec } => encode(version, codec).await,
            Mode::Decode => decode().await,
        }
    })
}

#[derive(Debug)]
struct Version(cid::Version);

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version_str = match self.0 {
            cid::Version::V0 => "v0",
            cid::Version::V1 => "v1",
        };
        write!(f, "{}", version_str)
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(version_str: &str) -> Result<Self, Self::Err> {
        let version = match version_str {
            "v0" => Ok(cid::Version::V0),
            "v1" => Ok(cid::Version::V1),
            _ => Err(format_err!("Unknown version {:?}", version_str)),
        };
        version.map(Self)
    }
}

#[derive(Debug)]
struct Codec(cid::Codec);

impl fmt::Display for Codec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use cid::Codec::*;
        let codec_str = match self.0 {
            Raw => "raw",
            DagProtobuf => "dag-pb",
            DagCBOR => "dag-cbor",
            DagJSON => "dag-json",
            GitRaw => "git-raw",
            EthereumBlock => "eth-block",
            EthereumBlockList => "eth-block-list",
            EthereumTxTrie => "eth-tx-trie",
            EthereumTx => "eth-tx",
            EthereumTxReceiptTrie => "eth-tx-receipt-trie",
            EthereumTxReceipt => "eth-tx-receipt",
            EthereumStateTrie => "eth-state-trie",
            EthereumAccountSnapshot => "eth-account-snapshot",
            EthereumStorageTrie => "eth-storage-trie",
            BitcoinBlock => "btc-block",
            BitcoinTx => "btc-tx",
            ZcashBlock => "zec-block",
            ZcashTx => "zec-tx",
        };
        write!(f, "{}", codec_str)
    }
}

impl FromStr for Codec {
    type Err = Error;

    fn from_str(codec_str: &str) -> Result<Self, Self::Err> {
        use cid::Codec::*;
        let codec = match codec_str {
            "raw" => Ok(Raw),
            "dag-pb" => Ok(DagProtobuf),
            "dag-cbor" => Ok(DagCBOR),
            "dag-json" => Ok(DagJSON),
            "git-raw" => Ok(GitRaw),
            "eth-block" => Ok(EthereumBlock),
            "eth-block-list" => Ok(EthereumBlockList),
            "eth-tx-trie" => Ok(EthereumTxTrie),
            "eth-tx" => Ok(EthereumTx),
            "eth-tx-receipt-trie" => Ok(EthereumTxReceiptTrie),
            "eth-tx-receipt" => Ok(EthereumTxReceipt),
            "eth-state-trie" => Ok(EthereumStateTrie),
            "eth-account-snapshot" => Ok(EthereumAccountSnapshot),
            "eth-storage-trie" => Ok(EthereumStorageTrie),
            "btc-block" => Ok(BitcoinBlock),
            "btc-tx" => Ok(BitcoinTx),
            "zec-block" => Ok(ZcashBlock),
            "zec-tx" => Ok(ZcashTx),
            _ => Err(format_err!("Unknown codec {:?}", codec_str)),
        };
        codec.map(Self)
    }
}

async fn encode(version: Version, codec: Codec) -> Result<(), ExitFailure> {
    let mut stdin = io::stdin();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer).await?;
    let hash = Multihash::from_bytes(buffer)?;
    let cid = Cid::new(codec.0, version.0, hash);
    print!("{}", cid);
    Ok(())
}

async fn decode() -> Result<(), ExitFailure> {
    let mut stdin = io::stdin();
    let mut buffer = String::new();
    stdin.read_to_string(&mut buffer).await?;
    let cid = Cid::try_from(buffer)?;
    println!("version: {}", Version(cid.version));
    println!("codec: {}", Codec(cid.codec));
    println!("hash: {}", multibase::encode(Base::Base58btc, &cid.hash.as_ref()));
    Ok(())
}
