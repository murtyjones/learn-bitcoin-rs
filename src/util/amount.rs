use std::fmt::{self};
use std::ops;

/// TODO there should be a way to implement a build script for this and for SignedAmount

/// Can be used to represent Bitcoin amounts. Supports
/// arithmetic operations.
#[derive(Copy, Clone, Hash, PartialEq, SatoshiArithmetic)]
pub struct Amount(u64);

/// Allows us to display amounts for Satoshis and compare them in tests
impl fmt::Debug for Amount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Amount({} satoshi)", self.as_sat())
    }
}

#[derive(Copy, Clone, Hash, PartialEq, SatoshiArithmetic)]
pub struct SignedAmount(i64);

/// Allows us to display amounts for Satoshis and compare them in tests
impl fmt::Debug for SignedAmount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SignedAmount({} satoshi)", self.as_sat())
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
    fn test_fmt() {
        let sat = Amount::from_sat;
        assert_eq!(format!("{:?}", sat(15)), "Amount(15 satoshi)");
    }

    #[test]
    fn test_add_subtract_multiply_divide() {
        let sat = Amount::from_sat;
        let ssat = SignedAmount::from_sat;

        // Basic arithmetic
        assert_eq!(sat(15) + sat(15), sat(30));
        assert_eq!(sat(15) - sat(15), sat(0));
        assert_eq!(sat(15) * 10, sat(150));
        assert_eq!(sat(15) / 5, sat(3));
        assert_eq!(sat(14) % 5, sat(4));
        assert_eq!(ssat(15) - ssat(20), ssat(-5));
        assert_eq!(ssat(-14) * 3, ssat(-42));
        assert_eq!(ssat(-14) % 3, ssat(-2));

        // test -Assign traits
        let mut bs = ssat(-5);
        bs += ssat(16);
        assert_eq!(ssat(11), bs);
        bs -= ssat(3);
        assert_eq!(ssat(8), bs);
        bs /= 2;
        assert_eq!(ssat(4), bs);
        bs %= 3;
        assert_eq!(ssat(1), bs);
        bs *= 14;
        assert_eq!(ssat(14), bs);
        let mut b = sat(5);
        b += sat(16);
        assert_eq!(sat(21), b);
        b -= sat(3);
        assert_eq!(sat(18), b);
        b /= 2;
        assert_eq!(sat(9), b);
        b %= 5;
        assert_eq!(sat(4), b);
        b *= 2;
        assert_eq!(sat(8), b);

        // panic when overflow occurs
        let result = panic::catch_unwind(|| Amount::max_value() + Amount::from_sat(1));
        assert!(result.is_err());
        let result = panic::catch_unwind(|| Amount::from_sat(8446744073709551615) * 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_checked_arithmetic() {
        let sat = Amount::from_sat;
        let ssat = SignedAmount::from_sat;

        assert_eq!(sat(42).checked_add(sat(1)), Some(sat(43)));
        assert_eq!(SignedAmount::max_value().checked_add(ssat(1)), None);
        assert_eq!(SignedAmount::min_value().checked_sub(ssat(1)), None);
        assert_eq!(Amount::max_value().checked_add(sat(1)), None);
        assert_eq!(Amount::min_value().checked_sub(sat(1)), None);

        assert_eq!(sat(5).checked_sub(sat(3)), Some(sat(2)));
        assert_eq!(sat(5).checked_sub(sat(6)), None);
        assert_eq!(ssat(5).checked_sub(ssat(6)), Some(ssat(-1)));
        assert_eq!(sat(5).checked_rem(2), Some(sat(1)));

        assert_eq!(sat(5).checked_div(2), Some(sat(2)));
    }
}
