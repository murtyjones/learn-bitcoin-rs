// Experimental features that we need

//! # Rust Bitcoin Library
//!
//! This is a library for which supports the Bitcoin network protocol and associated
//! primitives. It is designed for Rust programs built to work with the Bitcoin
//! network.
//!
//! It is also written entirely in Rust to illustrate the benefits of strong type
//! safety, including ownership and lifetime, for financial and/or cryptographic
//! software.

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![cfg_attr(all(test, feature = "unstable"), feature(test))]
// In general, rust is absolutely horrid at supporting users doing things like,
// for example, compiling Rust code for real environments. Disable useless lints
// that don't do anything but annoy us and cant actually ever be resolved.
#![allow(bare_trait_objects)]
#![allow(ellipsis_inclusive_range_patterns)]

pub extern crate bitcoin_hashes as hashes;

#[cfg(feature = "serde")]
extern crate serde;
#[cfg(all(test, feature = "serde"))]
#[macro_use]
extern crate serde_derive; // for 1.22.0 compat
#[cfg(all(test, feature = "serde"))]
extern crate serde_json;
#[cfg(all(test, feature = "serde"))]
extern crate serde_test;
#[cfg(all(test, feature = "unstable"))]
extern crate test;

#[macro_use]
pub mod internal_macros;
pub mod blockdata;
pub mod consensus;
pub mod network;
pub mod util;

pub use util::amount::Amount;
pub use util::amount::SignedAmount;
