//! Consensus
//!
//! This module defines structures, functions, and traits which are needed to
//! conform to Bitcoin consensus.

pub mod encode;

pub use self::encode::{deserialize, deserialize_partial, serialize};
pub use self::encode::{Decodable, Encodable, ReadExt, WriteExt};
//pub use self::params::Params;
