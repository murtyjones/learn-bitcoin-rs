//! Network message
//!
//! this module defines the `Message` traits whciha re used
//! for (de)serializing Bitcoin objects for transmissin on
//! the netowrk. It also defines (de)serialization routes for
//! many primitives.

use std::borrow::Cow;
use std::{io, iter, mem, fmt};
use consensus::{encode, serialize};
use consensus::encode::{Decodable, Encodable};

/// Serializer for a command string
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CommandString(Cow<'static, str>);

impl fmt::Display for CommandString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.as_ref())
    }
}

impl From<&'static str> for CommandString {
    fn from(f: &'static str) -> Self {
        CommandString(f.into())
    }
}

impl From<String> for CommandString {
    fn from(f: String) -> Self {
        CommandString(f.into())
    }
}

impl AsRef<str> for CommandString {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Encodable for CommandString {
    #[inline]
    fn consensus_encode<S: io::Write>(
        &self,
        s: S,
    ) -> Result<usize, encode::Error> {
        let mut rawbytes = [0u8; 12];
        let strbytes = self.0.as_bytes();
        if strbytes.len() > 12 {
            return Err(encode::Error::UnrecognizedNetworkCommand(self.0.clone().into_owned()));
        }
        for x in 0..strbytes.len() {
            rawbytes[x] = strbytes[x];
        }
        rawbytes.consensus_encode(s)
    }
}

impl Decodable for CommandString {
    #[inline]
    fn consensus_decode<D: io::Read>(d: D) -> Result<Self, encode::Error> {
        let rawbytes: [u8; 12] = Decodable::consensus_decode(d)?;
        let rv = iter::FromIterator::from_iter(
            rawbytes
            .iter()
            .filter_map(|&u| if u > 0 { Some(u as char) } else { None })
        );
        Ok(CommandString(rv))
    }
}