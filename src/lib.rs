//! # cid
//!
//! Implementation of [cid](https://github.com/ipld/cid) in Rust.

#![deny(missing_docs)]

mod cid;
mod codec;
mod error;
mod prefix;
#[cfg(any(feature = "ipld_dag_cbor", feature = "ipld_dag_json"))]
mod serde;
mod to_cid;
mod version;

pub use self::cid::Cid;
pub use self::codec::Codec;
pub use self::error::{Error, Result};
pub use self::prefix::Prefix;
#[cfg(feature = "ipld_dag_cbor")]
pub use self::serde::ipld_dag_cbor;
#[cfg(feature = "ipld_dag_json")]
pub use self::serde::ipld_dag_json;
pub use self::to_cid::ToCid;
pub use self::version::Version;
