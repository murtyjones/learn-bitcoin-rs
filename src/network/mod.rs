//! Network Support
//!
//! This module defines support for serialization/deserialization
//! and network transport of Bitcoin data/network messages.

use std::error;
use std::fmt;
use std::io;

pub mod address;
pub mod constants;
pub mod message_network;
pub use self::address::Address;
pub mod message;

/// Network error
#[derive(Debug)]
pub enum Error {
    /// An I/O error
    Io(io::Error),
    /// Socket mutex was poisoned
    SocketMutexPoisoned,
    /// Not connected to peer
    SocketNotConnectedToPeer,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => fmt::Display::fmt(e, f),
            Error::SocketMutexPoisoned | Error::SocketNotConnectedToPeer => {
                f.write_str(error::Error::description(self))
            }
        }
    }
}

#[doc(hidden)]
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::SocketMutexPoisoned => "socket mutex was poisoned",
            Error::SocketNotConnectedToPeer => "not connected to peer",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref e) => Some(e),
            Error::SocketMutexPoisoned | Error::SocketNotConnectedToPeer => None,
        }
    }
}
