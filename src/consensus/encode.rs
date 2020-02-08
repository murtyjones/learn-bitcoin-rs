//! Consensus-encodable types
//!
//! This is basically a replacement of the `Encodable` trait which does
//! normalization for endianness, etc. to ensure that the encoding matches
//! for the network consensus encoding.
