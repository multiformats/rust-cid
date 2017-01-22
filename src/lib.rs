/// ! # cid
/// !
/// ! Implementation of [cid](https://github.com/ipld/cid) in Rust.

extern crate multihash;
extern crate multibase;
extern crate try_from;
extern crate varmint;

use try_from::TryFrom;
use varmint::{WriteVarInt, ReadVarInt};
use std::fmt;

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
pub struct Cid {
    version: u64,
    codec: Codec,
    hash: Vec<u8>,
}

/// Error types
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Error {
    Fail,
    UnkownCodec,
    InputTooShort,
    ParsingError,
}

impl TryFrom<Vec<u8>> for Cid {
    type Err = Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Err> {
        Cid::try_from(data.as_slice())
    }
}

impl TryFrom<String> for Cid {
    type Err = Error;

    fn try_from(data: String) -> Result<Self, Self::Err> {
        Cid::try_from(&data[..])
    }
}

impl<'a> TryFrom<&'a str> for Cid {
    type Err = Error;

    fn try_from(data: &str) -> Result<Self, Self::Err> {
        if data.len() < 2 {
            return Err(Error::InputTooShort);
        }

        let mut data = data.to_string();

        if data.len() == 46 && &data[0..2] == "Qm" {
            data = multibase::Base::Base58btc.code().to_string() + &data;
        }

        println!("decoding {:?}", data);
        multibase::decode(data)
            .map_err(|_| Error::ParsingError)
            .and_then(|res| {
                println!("trying with {:?}\n{:?}", res.1, res.1);
                Cid::try_from(res.1)
            })
    }
}

impl<'a> TryFrom<&'a [u8]> for Cid {
    type Err = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Err> {
        // legacy multihash
        let is_legacy = data.len() == 34 && data[0] == 18 && data[1] == 32;

        if is_legacy {
            multihash::decode(data).unwrap();
            Ok(Cid::new(Codec::DagProtobuf, 0, data.to_vec()))
        } else {
            let mut data = data;
            let version = data.read_u64_varint().unwrap();
            let codec = data.read_u64_varint()
                .map(|raw| Codec::from(raw).unwrap())
                .unwrap();
            println!("{:?} {:}", data, data.len());
            // multihash::decode(&data).unwrap();
            Ok(Cid::new(codec, version, (*data).to_vec()))
        }
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.version {
            0 => self.to_string_v0(f),
            1 => self.to_string_v1(f),
            _ => panic!("should not happen"),
        }
    }
}

impl Cid {
    pub fn new(codec: Codec, version: u64, hash: Vec<u8>) -> Cid {
        Cid {
            version: version,
            codec: codec,
            hash: hash,
        }
    }

    fn to_string_v0(&self, f: &mut fmt::Formatter) -> fmt::Result {
        multibase::encode(multibase::Base::Base58btc, self.hash.as_slice())
            .map_err(|_| fmt::Error {})
            .and_then(|enc| {
                // Drop the first character as v0 does not know                 // about multibase
                f.write_str(&enc[1..])
            })
    }

    fn to_string_v1(&self, f: &mut fmt::Formatter) -> fmt::Result {
        multibase::encode(multibase::Base::Base58btc, self.as_bytes().as_slice())
            .map_err(|_| fmt::Error {})
            .and_then(|enc| {
                println!("data {:}", enc);
                f.write_str(&enc)
            })
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
    use ::{Cid, Codec, Error};
    use try_from::TryFrom;
    use multihash;

    #[test]
    fn basic_marshalling() {
        let h = multihash::encode(multihash::HashTypes::SHA2256, "beep boop".as_bytes()).unwrap();

        let cid = Cid::new(Codec::DagProtobuf, 1, h.to_vec());

        let data = cid.as_bytes();
        let out = Cid::try_from(data).unwrap();

        println!("first {:?}", h);
        assert_eq!(cid, out);

        let s = cid.to_string();
        let out2 = Cid::try_from(&s[..]).unwrap();
        println!("second {:?} {:?} {:?}", cid, out2, &s);
        assert_eq!(cid, out2);
    }

    #[test]
    fn empty_string() {
        assert_eq!(Cid::try_from(""), Err(Error::InputTooShort));
    }

    #[test]
    fn v0_handling() {
        let old = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";
        let cid = Cid::try_from(old).unwrap();

        assert_eq!(cid.version, 0);
        assert_eq!(cid.to_string(), old);
    }
}
