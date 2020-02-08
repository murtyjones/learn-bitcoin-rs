//! Consensus
//!
//! This module defines structures, functions, and traits which are needed to
//! conform to Bitcoin consensus.

pub mod encode;

pub use self::encode::{Decodable, Encodable};
