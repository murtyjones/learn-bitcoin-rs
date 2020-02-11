use std::error;
use std::fmt;

use blockdata::transaction::Transaction;
use util::psbt::raw;

/// Ways that a partially Signed Transaction might fail.
#[derive(Debug)]
pub enum Error {
    /// Magic bytes for a PSBT must be the ASCII for "psbt" serialized
    /// in most significant byte order
    InvalidMagic,
}
