//! Network constants
//!
//! This module provides various constants relating to the Bitcoin network
//! protocol, such as protocol versioning and magic header bytes.
//!
//! The [`Network`][1] type implements the [`Decodable`][2] and
//! [`Encodable`][3] traits and encodes the magic bytes of the
//! given network.
//!
//! [1]: enum.Network.html
//! [2]: ../../consensus/encode/trait.Decodable.html
//! [3]: ../../consensus/encode/trait.Encodable.html
//!
//! # Example: encoding a network's magic bytes
//!
//! ```rust
//! use bitcoin::network::constants::Network;
//! use bitcoin::consensus::encode::serialize;
//!
//! let network = Network::Bitcoin;
//! let bytes = serialize(&network.magic());
//!
//! assert_eq!(&bytes[..], &[0xF9, 0xBE, 0xB4, 0xD9]);
//! ```

//use std::{fmt, io, ops};
//
//use consensus::encode::{self, Decodable, Encodable};

/// Version of the protocol as appearing in network message
pub const PROTOCOL_VERSION: u32 = 70001;

user_enum! {
    /// The cryptocurrency to act on
    #[derive(Copy, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
    pub enum Network {
        /// Classic Bitcoin
        Bitcoin <-> "bitcoin",
        /// Bitcoin's testnet
        Testnet <-> "testnet",
        /// Bitcoin's regtest
        Regtest <-> "regtest"
    }
}

impl Network {
    /// Creates a `Network` from the magic bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitcoin::network::constants::Network;
    ///
    /// assert_eq!(Some(Network::Bitcoin), Network::from_magic(0xD9B4BEF9));
    /// assert_eq!(None, Network::from_magic(0xFFFFFFFF));
    /// ```
    pub fn from_magic(magic: u32) -> Option<Network> {
        // Note: any new entries here must be added to `magic` below
        match magic {
            0xD9B4BEF9 => Some(Network::Bitcoin),
            0x0709110B => Some(Network::Testnet),
            0xDAB5BFFA => Some(Network::Regtest),
            _ => None,
        }
    }

    /// Return the network magic bytes, which should be encoded little-endian
    /// at the start of every message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitcoin::network::constants::Network;
    ///
    /// let network = Network::Bitcoin;
    /// assert_eq!(network.magic(), 0xD9B4BEF9);
    /// ```
    pub fn magic(&self) -> u32 {
        // Note: any new entries here must be added to `magic` below
        match *self {
            Network::Bitcoin => 0xD9B4BEF9,
            Network::Testnet => 0x0709110B,
            Network::Regtest => 0xDAB5BFFA,
        }
    }
}
