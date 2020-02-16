//! Network-related network messages
//!
//! This modules defines network messages which describe peers and
//! their capabilities

use std::borrow::Cow;
use std::io;

use consensus::encode;
use consensus::{Decodable, Encodable, ReadExt};
use hashes::sha256d;
use network::address::Address;
use network::constants::{self, ServiceFlags};
use network::message::CommandString;

/// Some simple messages

/// The `version` message
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct VersionMessage {
    /// The P2P network protocl version
    pub version: u32,
    /// A bitmask describing the services supported by this node
    pub services: ServiceFlags,
    /// The time at which this `version` message was sent
    pub timestamp: i64,
    /// The network address of the peer receiving the message
    pub receiver: Address,
    /// The network address of the peer sending the message
    pub sender: Address,
    /// A random nonce used to detect loops in the network
    pub nonce: u64,
    /// A string describing the peer's software
    pub user_agent: String,
    /// The height of the maximum-work blockchain that the peer is aware of
    pub start_height: i32,
    /// Whether the receiving per should relay messages to the sender; used
    /// if the sender is badnwidth limited and would to support
    /// bloom-filtering. Defaults to false.
    pub relay: bool,
}
