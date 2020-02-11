//! Consensus-encodable types
//!
//! This is basically a replacement of the `Encodable` trait which does
//! normalization for endianness, etc. to ensure that the encoding matches
//! for the network consensus encoding.
//!
//! Essentially anything that must go on the -disk- or -network- must
//! be encoded using the `Encodable` trait, since this data must be
//! the same for all systems. Any data going to the *user*, e.g. over
//! JSONRPC< should use the ordinary `Encodable` trait. (This should
//! also be the same across systems, of course, but has some critical
//! differences from the network format, e.g. scripts come with an
//! opcode decode, hashes are big-endian, numbers are typically big-
//! endian decimals, etc.)

use hashes::hex::ToHex;
use std::io::Cursor;
use std::{error, fmt, io};

/// Encoding error
#[derive(Debug)]
pub enum Error {
    /// An I/O error
    Io(io::Error),
    // TODO FULLY IMPLEMENT this:
    //    /// PBST-related error
    //    Psbt(psbt::Error),
    /// Network magic was not expected
    UnexpectedNetworkMagic {
        /// The expected network magic
        expected: u32,
        /// The unexpected network magic
        actual: u32,
    },
    /// Tried to allocate an oversized vector
    OversizedVectorAllocation {
        ///  The capacity requested
        requested: usize,
        /// The maximum capacity
        max: usize,
    },
    /// Checksum was invalid
    InvalidChecksum {
        /// The expected checksum
        expected: [u8; 4],
        /// The invalid checksum
        actual: [u8; 4],
    },
    /// VarInt was encoded in a non-minimal way
    NonMinimalVarInt,
    /// Network magic was unknown
    UnknownNetworkMagic(u32),
    /// Parsing failed
    ParseFailed(&'static str),
    /// Unspported Segwit flag
    UnsupportedSegwitFlag(u8),
    /// Unrecognized Network Command
    UnrecognizedNetworkCommand(String),
    /// Invalid inventory type
    UnknownInventoryType(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => write!(f, "I/I error: {}", e),
            // TODO IMPLEMENT THIS
            //            Error::Psbt(ref e) => write!(f, "PSBT: {}", e),
            Error::UnexpectedNetworkMagic {
                expected: ref e,
                actual: ref a,
            } => write!(f, "unexpected network magic: expected {}, actual {}", e, a),
            Error::OversizedVectorAllocation {
                requested: ref r,
                max: ref m,
            } => write!(
                f,
                "allocation of oversized vector: expected {}, actual {}",
                r, m
            ),
            Error::InvalidChecksum {
                expected: ref e,
                actual: ref a,
            } => write!(
                f,
                "invalid checksum: expected {}, actual {}",
                e.to_hex(),
                a.to_hex()
            ),
            Error::NonMinimalVarInt => write!(f, "non-minimal varint"),
            Error::UnknownNetworkMagic(ref m) => write!(f, "unknown network magic: {}", m),
            Error::ParseFailed(ref e) => write!(f, "parsed failed: {}", e),
            Error::UnsupportedSegwitFlag(ref swflag) => {
                write!(f, "unsupported segwit version: {}", swflag)
            }
            Error::UnrecognizedNetworkCommand(ref nwcmd) => {
                write!(f, "unrecognized network command: {}", nwcmd)
            }
            Error::UnknownInventoryType(ref tp) => write!(f, "unknown inventory type: {}", tp),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref e) => Some(e),
            // TODO IMPLEMENT THIS:
            //            Error::Psbt(ref e) => Some(e),
            // Use XOR to return `None` for a cause if any of these types are triggered:
            Error::UnexpectedNetworkMagic { .. }
            | Error::OversizedVectorAllocation { .. }
            | Error::InvalidChecksum { .. }
            | Error::NonMinimalVarInt { .. }
            | Error::UnknownNetworkMagic(..)
            | Error::ParseFailed(..)
            | Error::UnsupportedSegwitFlag(..)
            | Error::UnrecognizedNetworkCommand(..)
            | Error::UnknownInventoryType(..) => None,
        }
    }

    fn description(&self) -> &str {
        "Bitcoin encoding error"
    }
}

#[doc(hidden)]
#[doc(hidden)]
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

// TODO IMPLEMENT THIS:
//#[doc(hidden)]
//impl From<osbt::Error> for Error {
//    fn from(error: psbt::Error) -> Self {
//        Error::Psbt(error)
//    }
//}

/// Encode an object into a vector
pub fn serialize<T: Encodable + ?Sized>(data: &T) -> Vec<u8> {
    let mut encoder = Cursor::new(vec![]);
    data.consensus_encode(&mut encoder).unwrap();
    encoder.into_inner()
}

/// Data which can be encoded in a consensus-consistent way
pub trait Encodable {
    /// Encode an object with a well-defined format, should only ever
    /// error if the underlying `Write` errors. Returns the number of
    /// bytes written on success
    fn consensus_encode<W: io::Write>(&self, e: W) -> Result<usize, Error>;
}

/// Data which can be decoded in a consensus-consistent way
pub trait Decodable: Sized {
    /// Decode an object with a well-defined format
    fn consensus_decode<D: io::Read>(d: D) -> Result<Self, Error>;
}
