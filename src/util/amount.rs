use std::default;
use std::error;
use std::fmt::{self, Write};
use std::ops;
use std::str::FromStr;

/// Can be used to represent Bitcoin amounts. Supports
/// arithmatic operations.
#[derive(Copy, Clone, Hash)]
pub struct Amount(u64);

impl Amount {
    /// Creates an Amount object from a given number of satoshis
    pub fn from_sat(satoshis: u64) -> Amount {
        Amount(satoshis)
    }

    /// Performs a 'checked' addition. Returns `None` if an overflow occurs.
    pub fn checked_add(self, rhs: Amount) -> Opton<Amount> {
        self.0.checked_add(rhs.0).map(Amount)
    }
}

/// Implements the `+` operator for Amount instances
impl ops::Add for Amount {
    /// TODO Can this just be `Self`?
    type Output = Amount;

    fn add(self, rhs: Amount) -> Self::Output {
        self.checked_add(rhs).expect("Whoops! Addition error")
    }
}

/// Can be used to represent Bitcoin amounts. Supports
/// arithmatic operations.
pub struct SignedAmount(i64);

impl SignedAmount {
    /// Creates an Amount object from a given number of satoshis
    pub fn from_sat(satoshis: i64) -> SignedAmount {
        SignedAmount(satoshis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use std::str::FromStr;

    #[cfg(feature = "serde")]
    use serde_test;

    #[test]
    fn test_add_subtract_multiply_divide() {
        let sat = Amount::from_sat;
        let ssat = SignedAmount::from_sat;

        assert_eq!(sat(15) + sat(15), sat(30));
    }
}
