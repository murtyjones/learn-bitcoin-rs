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
use std::io::{Cursor, Read, Write};
use std::{error, fmt, io, mem, u32};

use util::endian;

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

/// Extensions of `Write` to encode data as per Bitcoin consensus
pub trait WriteExt {
    /// Output a 64-bit uint
    fn emit_u64(&mut self, v: u64) -> Result<(), Error>;
    /// Output a 32-bit uint
    fn emit_u32(&mut self, v: u32) -> Result<(), Error>;
    /// Output a 16-bit uint
    fn emit_u16(&mut self, v: u16) -> Result<(), Error>;
    /// Output a 8-bit uint
    fn emit_u8(&mut self, v: u8) -> Result<(), Error>;

    /// Output a 64-bit int
    fn emit_i64(&mut self, v: i64) -> Result<(), Error>;
    /// Output a 32-bit int
    fn emit_i32(&mut self, v: i32) -> Result<(), Error>;
    /// Output a 16-bit int
    fn emit_i16(&mut self, v: i16) -> Result<(), Error>;
    /// Output a 8-bit int
    fn emit_i8(&mut self, v: i8) -> Result<(), Error>;

    /// Output a boolean
    fn emit_bool(&mut self, v: bool) -> Result<(), Error>;

    /// Output a byte slice
    fn emit_slice(&mut self, v: &[u8]) -> Result<(), Error>;
}

/// Extensions of `Read` to decode data as per Bitcoin consensus
pub trait ReadExt {
    /// Read a 64-bit uint
    fn read_u64(&mut self) -> Result<u64, Error>;
    /// Read a 32-bit uint
    fn read_u32(&mut self) -> Result<u32, Error>;
    /// Read a 16-bit uint
    fn read_u16(&mut self) -> Result<u16, Error>;
    /// Read a 8-bit uint
    fn read_u8(&mut self) -> Result<u8, Error>;

    /// Read a 64-bit int
    fn read_i64(&mut self) -> Result<i64, Error>;
    /// Read a 32-bit int
    fn read_i32(&mut self) -> Result<i32, Error>;
    /// Read a 16-bit int
    fn read_i16(&mut self) -> Result<i16, Error>;
    /// Read a 8-bit int
    fn read_i8(&mut self) -> Result<i8, Error>;

    /// Read a boolean
    fn read_bool(&mut self) -> Result<bool, Error>;

    /// Read a byte slice
    fn read_slice(&mut self, slice: &mut [u8]) -> Result<(), Error>;
}

macro_rules! encoder_fn {
    ($name:ident, $val_type:ty, $writefn:ident) => {
        #[inline]
        fn $name(&mut self, v: $val_type) -> Result<(), Error> {
            self.write_all(&endian::$writefn(v)).map_err(Error::Io)
        }
    }
}

macro_rules! decoder_fn {
    ($name:ident, $val_type:ty, $readfn:ident, $byte_len: expr) => {
        #[inline]
        fn $name(&mut self) -> Result<$val_type, Error> {
            assert_eq!(::std::mem::size_of::<$val_type>(), $byte_len); // size_of isn't a constfn in 1.22
            let mut val = [0; $byte_len];
            self.read_exact(&mut val[..]).map_err(Error::Io)?;
            Ok(endian::$readfn(&val))
        }
    }
}

impl<W: Write> WriteExt for W {
    encoder_fn!(emit_u64, u64, u64_to_array_le);
    encoder_fn!(emit_u32, u32, u32_to_array_le);
    encoder_fn!(emit_u16, u16, u16_to_array_le);
    encoder_fn!(emit_i64, i64, i64_to_array_le);
    encoder_fn!(emit_i32, i32, i32_to_array_le);
    encoder_fn!(emit_i16, i16, i16_to_array_le);

    #[inline]
    fn emit_i8(&mut self, v: i8) -> Result<(), Error> {
        self.write_all(&[v as u8]).map_err(Error::Io)
    }
    #[inline]
    fn emit_u8(&mut self, v: u8) -> Result<(), Error> {
        self.write_all(&[v]).map_err(Error::Io)
    }
    #[inline]
    fn emit_bool(&mut self, v: bool) -> Result<(), Error> {
        self.write_all(&[if v { 1 } else { 0 }]).map_err(Error::Io)
    }
    #[inline]
    fn emit_slice(&mut self, v: &[u8]) -> Result<(), Error> {
        self.write_all(v).map_err(Error::Io)
    }
}

impl<R: Read> ReadExt for R {
    decoder_fn!(read_u64, u64, slice_to_u64_le, 8);
    decoder_fn!(read_u32, u32, slice_to_u32_le, 4);
    decoder_fn!(read_u16, u16, slice_to_u16_le, 2);
    decoder_fn!(read_i64, i64, slice_to_i64_le, 8);
    decoder_fn!(read_i32, i32, slice_to_i32_le, 4);
    decoder_fn!(read_i16, i16, slice_to_i16_le, 2);

    #[inline]
    fn read_u8(&mut self) -> Result<u8, Error> {
        let mut slice = [0u8; 1];
        self.read_exact(&mut slice)?;
        Ok(slice[0])
    }
    #[inline]
    fn read_i8(&mut self) -> Result<i8, Error> {
        let mut slice = [0u8; 1];
        self.read_exact(&mut slice)?;
        Ok(slice[0] as i8)
    }
    #[inline]
    fn read_bool(&mut self) -> Result<bool, Error> {
        ReadExt::read_i8(self).map(|bit| bit != 0)
    }
    #[inline]
    fn read_slice(&mut self, slice: &mut [u8]) -> Result<(), Error> {
        self.read_exact(slice).map_err(Error::Io)
    }
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

// Primitive types
macro_rules! impl_int_encodable {
    ($ty:ident, $meth_dec:ident, $meth_enc:ident) => {
        impl Decodable for $ty {
            #[inline]
            fn consensus_decode<D: io::Read>(mut d: D) -> Result<Self, Error> {
                ReadExt::$meth_dec(&mut d).map($ty::from_le)
            }
        }

        impl Encodable for $ty {
            #[inline]
            fn consensus_encode<S: WriteExt>(&self, mut s: S) -> Result<usize, self::Error> {
                s.$meth_enc(self.to_le())?;
                Ok(mem::size_of::<$ty>())
            }
        }
    };
}

impl_int_encodable!(u8, read_u8, emit_u8);
impl_int_encodable!(u16, read_u16, emit_u16);
impl_int_encodable!(u32, read_u32, emit_u32);
impl_int_encodable!(u64, read_u64, emit_u64);
impl_int_encodable!(i8, read_i8, emit_i8);
impl_int_encodable!(i16, read_i16, emit_i16);
impl_int_encodable!(i32, read_i32, emit_i32);
impl_int_encodable!(i64, read_i64, emit_i64);

impl Encodable for bool {
    #[inline]
    fn consensus_encode<S: WriteExt>(&self, mut s: S) -> Result<usize, Error> {
        s.emit_u8(if *self { 1 } else { 0 })?;
        Ok(1)
    }
}

impl Decodable for bool {
    #[inline]
    fn consensus_decode<D: io::Read>(mut d: D) -> Result<bool, Error> {
        ReadExt::read_u8(&mut d).map(|n| n != 0)
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::serialize;

    #[test]
    fn serialize_int_test() {
        assert_eq!(serialize(&false), vec![0u8]);
        assert_eq!(serialize(&true), vec![1u8]);
    }
}
