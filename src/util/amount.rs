use std::default;
use std::error;
use std::fmt::{self, Write};
use std::ops;
use std::str::FromStr;

/// TODO there should be a way to implement a build script for this and for SignedAmount

/// Can be used to represent Bitcoin amounts. Supports
/// arithmatic operations.
#[derive(Copy, Clone, Hash)]
pub struct Amount(u64);

impl Amount {
    /// Creates an Amount object from a given number of satoshis
    pub fn from_sat(satoshis: u64) -> Amount {
        Amount(satoshis)
    }

    /// Get the number of satoshis
    pub fn as_sat(self) -> u64 {
        self.0
    }

    /// Performs a 'checked' addition. Returns `None` if an overflow occurs.
    /// @see https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedAdd.html
    pub fn checked_add(self, rhs: Amount) -> Option<Amount> {
        self.0.checked_add(rhs.0).map(Amount)
    }

    /// Performs a 'checked' substrction. Returns `None` if an overflow occurs.
    /// @see https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedSub.html
    pub fn checked_sub(self, rhs: Amount) -> Option<Amount> {
        self.0.checked_sub(rhs.0).map(Amount)
    }

    /// Performs a checked multiplication, returning None if an overflow occurs.
    /// https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedMul.html
    pub fn checked_mul(self, rhs: u64) -> Option<Amount> {
        self.0.checked_mul(rhs).map(Amount)
    }

    /// Performs a checked division, returning None if an overflow occurs.
    /// https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedDiv.html
    /// NOTE: The remainder will be lost if no exact division takes place.
    pub fn checked_div(self, rhs: u64) -> Option<Amount> {
        self.0.checked_div(rhs).map(Amount)
    }

    /// Performs a checked remainder, returning None if an overflow occurs.
    /// https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedDiv.html
    pub fn checked_rem(self, rhs: u64) -> Option<Amount> {
        self.0.checked_rem(rhs).map(Amount)
    }
}

/// Implements the `+` operator using a checked addition for Amount instances.
impl ops::Add for Amount {
    /// bitcoin-rs uses `type Output = Amount;` here for some reason:
    type Output = Self;

    fn add(self, rhs: Amount) -> Self::Output {
        self.checked_add(rhs).expect("Whoops! Addition error")
    }
}

/// Allows us to subtract one Amount from another using `-`
impl ops::Sub for Amount {
    type Output = Self;

    fn sub(self, rhs: Amount) -> Self::Output {
        self.checked_sub(rhs).expect("Whoops! Subtraction error")
    }
}

/// Allows us to compare Amounts using `==`
impl PartialEq for Amount {
    fn eq(&self, other: &Amount) -> bool {
        PartialEq::eq(&self.0, &other.0)
    }
}

/// Allows us to display amounts for Satoshis and compare them in tests
impl fmt::Debug for Amount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Amount({} satoshi)", self.as_sat())
    }
}

/// Allows us to use `*` to multiply Amounts
impl ops::Mul<u64> for Amount {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self {
        self.checked_mul(rhs).expect("Uh oh! Multiplcation error")
    }
}

/// Allows us to use `/` to divide Amounts
impl ops::Div<u64> for Amount {
    type Output = Self;

    fn div(self, rhs: u64) -> Self {
        self.checked_div(rhs).expect("Uh oh! Division error")
    }
}

/// Allows us to use `%` to find remainders from dividing Amounts
impl ops::Rem<u64> for Amount {
    type Output = Self;

    fn rem(self, modulus: u64) -> Self {
        self.checked_rem(modulus).expect("Uh oh! Remainder error")
    }
}

#[derive(Copy, Clone, Hash)]
pub struct SignedAmount(i64);

impl SignedAmount {
    /// Creates an Amount object from a given number of satoshis
    pub fn from_sat(satoshis: i64) -> SignedAmount {
        SignedAmount(satoshis)
    }

    /// Get the number of satoshis
    pub fn as_sat(self) -> i64 {
        self.0
    }

    /// Performs a 'checked' addition. Returns `None` if an overflow occurs.
    /// @see https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedAdd.html
    pub fn checked_add(self, rhs: SignedAmount) -> Option<SignedAmount> {
        self.0.checked_add(rhs.0).map(SignedAmount)
    }

    /// Performs a 'checked' substrction. Returns `None` if an overflow occurs.
    /// @see https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedSub.html
    pub fn checked_sub(self, rhs: SignedAmount) -> Option<SignedAmount> {
        self.0.checked_sub(rhs.0).map(SignedAmount)
    }

    /// Performs a checked multiplication, returning None if an overflow occurs.
    /// https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedMul.html
    pub fn checked_mul(self, rhs: i64) -> Option<SignedAmount> {
        self.0.checked_mul(rhs).map(SignedAmount)
    }

    /// Performs a checked division, returning None if an overflow occurs.
    /// https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedDiv.html
    /// NOTE: The remainder will be lost if no exact division takes place.
    pub fn checked_div(self, rhs: i64) -> Option<SignedAmount> {
        self.0.checked_div(rhs).map(SignedAmount)
    }

    /// Performs a checked remainder, returning None if an overflow occurs.
    /// https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedDiv.html
    pub fn checked_rem(self, rhs: i64) -> Option<SignedAmount> {
        self.0.checked_rem(rhs).map(SignedAmount)
    }
}

/// Implements the `+` operator using a checked addition for SignedAmount instances.
impl ops::Add for SignedAmount {
    /// bitcoin-rs uses `type Output = SignedAmount;` here for some reason:
    type Output = Self;

    fn add(self, rhs: SignedAmount) -> Self::Output {
        self.checked_add(rhs).expect("Whoops! Addition error")
    }
}

/// Allows us to subtract one SignedAmount from another using `-`
impl ops::Sub for SignedAmount {
    type Output = Self;

    fn sub(self, rhs: SignedAmount) -> Self::Output {
        self.checked_sub(rhs).expect("Whoops! Subtraction error")
    }
}

/// Allows us to compare SignedAmounts using `==`
impl PartialEq for SignedAmount {
    fn eq(&self, other: &SignedAmount) -> bool {
        PartialEq::eq(&self.0, &other.0)
    }
}

/// Allows us to display amounts for Satoshis and compare them in tests
impl fmt::Debug for SignedAmount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SignedAmount({} satoshi)", self.as_sat())
    }
}

/// Allows us to use `*` to multiply SignedAmounts
impl ops::Mul<i64> for SignedAmount {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self {
        self.checked_mul(rhs).expect("Uh oh! Multiplcation error")
    }
}

/// Allows us to use `/` to divide SignedAmounts
impl ops::Div<i64> for SignedAmount {
    type Output = Self;

    fn div(self, rhs: i64) -> Self {
        self.checked_div(rhs).expect("Uh oh! Division error")
    }
}

/// Allows us to use `%` to find remainders from dividing SignedAmounts
impl ops::Rem<i64> for SignedAmount {
    type Output = Self;

    fn rem(self, modulus: i64) -> Self {
        self.checked_rem(modulus).expect("Uh oh! Remainder error")
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

        assert_eq!(sat(15) + sat(15), sat(30));
        assert_eq!(sat(15) - sat(15), sat(0));
        assert_eq!(sat(15) * 10, sat(150));
        assert_eq!(sat(15) / 5, sat(3));
        assert_eq!(sat(14) % 5, sat(4));
        assert_eq!(ssat(15) - ssat(20), ssat(-5));
    }
}
