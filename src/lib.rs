// Experimental features that we need
#![cfg_attr(all(test, feature = "unstable"), feature(test))]

extern crate proc_macro;
#[macro_use]
extern crate learn_bitcoin_rs_macros;

#[cfg(feature = "serde")]
extern crate serde;
#[cfg(all(test, feature = "serde"))]
#[macro_use]
extern crate serde_derive; // for 1.22.0
#[cfg(all(test, feature = "serde"))]
extern crate serde_test;
#[cfg(all(test, feature = "unstable"))]
extern crate test;

pub mod util;
