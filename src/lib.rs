/// ! # cid
/// !
/// ! Implementation of [cid](https://github.com/ipld/cid) in Rust.

extern crate multihash;
extern crate try_from;
extern crate varmint;

use multihash::Multihash;
use try_from::TryFrom;
use varmint::{WriteVarInt, ReadVarInt};

#[derive(PartialEq, Eq, Clone, Debug)]
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

impl Codec {
    /// Convert to the matching integer code
    pub fn to_code(&self) -> u64 {
        use Codec::*;

        match *self {
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

    // Convert a number to the matching codec
    pub fn from(raw: u64) -> Result<Codec, Error> {
        use Codec::*;

        match raw {
            0x55 => Ok(Raw),
            0x70 => Ok(DagProtobuf),
            0x71 => Ok(DagCBOR),
            0x90 => Ok(EthereumBlock),
            0x91 => Ok(EthereumTx),
            0xb0 => Ok(BitcoinBlock),
            0xb1 => Ok(BitcoinTx),
            0xc0 => Ok(ZcashBlock),
            0xc1 => Ok(ZcashTx),
            _ => Err(Error::UnkownCodec),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cid<'a> {
    version: u64,
    codec: Codec,
    hash: &'a [u8],
}

/// Error types
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Error {
    Fail,
    UnkownCodec,
}

impl<'a> TryFrom<&'a [u8]> for Cid<'a> {
    type Err = Error;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Err> {
        // legacy multihash
        let is_legacy = data.len() == 34 && data[0] == 18 && data[1] == 32;

        println!("{:} {:} {:}", data.len(), data[0], data[1]);
        if is_legacy {
            multihash::decode(data).unwrap();
            Ok(Cid::new(Codec::DagProtobuf, 0, data))
        } else {
            let mut data = data;
            let version = data.read_u64_varint().unwrap();
            let codec = data.read_u64_varint()
                .map(|raw| Codec::from(raw).unwrap())
                .unwrap();
            println!("{:?}", data);
            multihash::decode(&data).unwrap();
            Ok(Cid::new(codec, version, data))
        }
    }
}

impl<'a> Cid<'a> {
    pub fn new(codec: Codec, version: u64, hash: &'a [u8]) -> Cid {
        Cid {
            version: version,
            codec: codec,
            hash: hash,
        }
    }

    fn as_bytes_v0(&self) -> Vec<u8> {
        self.hash.iter().cloned().collect()
    }

    fn as_bytes_v1(&self) -> Vec<u8> {
        let mut res = vec![];
        res.write_u64_varint(self.version).unwrap();
        res.write_u64_varint(self.codec.to_code()).unwrap();
        res.extend(self.hash.iter().cloned().collect::<Vec<u8>>());

        res
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self.version {
            0 => self.as_bytes_v0(),
            1 => self.as_bytes_v1(),
            _ => panic!("should not happen"),
        }
    }
}

#[cfg(test)]
mod tests {
    use ::{Cid, Codec};
    use try_from::TryFrom;
    use multihash;

    #[test]
    fn basic_marshalling() {
        let h = multihash::encode(multihash::HashTypes::SHA2256, "beep boop".as_bytes()).unwrap();

        let cid = Cid::new(Codec::DagProtobuf, 1, &h);

        let data = cid.as_bytes();
        let out = Cid::try_from(data.as_slice()).unwrap();

        assert_eq!(cid, out);
    }
}
