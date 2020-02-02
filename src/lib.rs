extern crate proc_macro;
#[macro_use]
extern crate learn_bitcoin_rs_macros;
#[cfg(all(test, feature = "mutation_testing"))]
extern crate mutagen;

pub mod util;

#[cfg(all(
    any(feature = "mutation_testing", feature = "fuzztarget"),
    not(any(test, debug_assertions))
))]
const ERR: () = "You should never be building with feature = mutation_testing or feature = fuzztarget! They are used to compile with broken code for testing only!";
