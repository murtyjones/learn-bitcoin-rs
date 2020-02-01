use std::fmt::{self};
use std::ops;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Denomination {
    // BTC
    Bitcoin,
    // mBTC
    MilliBitcoin,
    // uBTC
    MicroBitcoin,
    // bits
    Bit,
    // msat
    Satoshi,
    // msat
    MilliSatoshi,
}

impl Denomination {
    /// The number of decimal places more than a satoshi
    fn precision(self) -> i32 {
        match self {
            Denomination::Bitcoin => -8,
            Denomination::MilliBitcoin => -5,
            Denomination::MicroBitcoin => -2,
            Denomination::Bit => -2,
            Denomination::Satoshi => 0,
            Denomination::MilliSatoshi => -3,
        }
    }
}

impl fmt::Display for Denomination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Denomination::Bitcoin => "BTC",
            Denomination::MilliBitcoin => "mBTC",
            Denomination::MicroBitcoin => "uBTC",
            Denomination::Bit => "bits",
            Denomination::Satoshi => "satoshi",
            Denomination::MilliSatoshi => "msat",
        })
    }
}

/// E.g. let money: Denomination = "BTC".into();
impl FromStr for Denomination {
    type Err = ParseAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BTC" => Ok(Denomination::Bitcoin),
            "mBTC" => Ok(Denomination::MilliBitcoin),
            "uBTC" => Ok(Denomination::MicroBitcoin),
            "bits" => Ok(Denomination::Bit),
            "satoshi" => Ok(Denomination::Satoshi),
            "sat" => Ok(Denomination::Satoshi),
            "msat" => Ok(Denomination::MilliSatoshi),
            d => Err(ParseAmountError::UnknownDenomination(d.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAmountError {
    /// Amount is negative
    Negative,
    /// Amount is too big to fit inside of the type
    TooBig,
    /// Amount has higher-than-supported decimal precision
    TooPrecise,
    /// Invalid number format
    InvalidFormat,
    /// Input string was too large
    InputTooLarge,
    /// Invalid char in input string
    InvalidCharacters(char),
    /// The denomination didn't match our known ones
    UnknownDenomination(String),
}

/// Can be used to represent Bitcoin amounts. Supports
/// arithmetic operations.
#[derive(Copy, Clone, Hash, PartialEq, SatoshiArithmetic)]
pub struct Amount(u64);

#[derive(Copy, Clone, Hash, PartialEq, SatoshiArithmetic)]
pub struct SignedAmount(i64);

impl SignedAmount {
    /// Subtraction that doesn't allow negative [SignedAmount]s.
    /// Returns [None] if either [self], [rhs] or the result is strictly negative.
    pub fn positive_sub(self, rhs: SignedAmount) -> Option<SignedAmount> {
        if self.is_negative() || rhs.is_negative() || rhs > self {
            None
        } else {
            self.checked_sub(rhs)
        }
    }

    /// Returns `true` if this [SignedAmount] is negative and `false` if
    /// this
    pub fn is_negative(self) -> bool {
        self.0.is_negative()
    }

    /// Parse a decimal string as a value in a given denomination
    ///
    /// Note: This only parses the string value. If you want to parse
    /// a value with denomination, use [FromStr]
    pub fn from_str_in(s: &str, denom: Denomination) -> Result<SignedAmount, ParseAmountError> {
        let (negative, satoshi) = parse_signed_to_satoshi(s, denom)?;
        if satoshi > i64::max_value() as u64 {
            return Err(ParseAmountError::TooBig);
        }
        Ok(match negative {
            true => SignedAmount(-1 * satoshi as i64),
            false => SignedAmount(satoshi as i64),
        })
    }

    /// Convert this [SignedAmount] in a floating-point notatino with a
    /// given denomination.
    /// Can return an error if the amount is too big, too precise, etc.
    ///
    /// TODO figure out what the risk of using this is? From rust-bitcoin:
    /// "Please be aware of the risk of using floating-point numbers."
    pub fn from_float_in(
        value: f64,
        denom: Denomination,
    ) -> Result<SignedAmount, ParseAmountError> {
        SignedAmount::from_str_in(&value.to_string(), denom)
    }
}

/// Parses a decimal string in the given denomination into a satoshi value
/// and a boolean that indicates whether it's a negative amount
fn parse_signed_to_satoshi(
    mut s: &str,
    denom: Denomination,
) -> Result<(bool, u64), ParseAmountError> {
    if s.len() == 0 {
        return Err(ParseAmountError::InvalidFormat);
    }
    // TODO why is 50 the max?
    if s.len() > 50 {
        return Err(ParseAmountError::InputTooLarge);
    }

    let is_negative = s.chars().next().unwrap() == '-';
    // If negative, either return an error (if the `-` is the
    // only character) or remove the `-` and continue parsing
    if is_negative {
        if s.len() == 1 {
            return Err(ParseAmountError::InvalidFormat);
        }
        s = &s[1..];
    }

    let max_decimals = {
        // The difference in precision between native (satoshi)
        // and desired denomation.
        let precision_diff = -denom.precision();
        if precision_diff < 0 {
            // If the precision diff is negative this means we're prasing
            // into a less percise amount, which is only allowed when there
            // arent any decimals, and the last digits are just zeroes as many
            // as is the difference in precision.
            let last_n = precision_diff.abs() as usize;
            if is_too_precise(s, last_n) {
                return Err(ParseAmountError::TooPrecise);
            }
            s = &s[0..s.len() - last_n];
            0
        } else {
            precision_diff
        }
    };

    let mut decimals = None;
    let mut value: u64 = 0; // as satoshis
    for c in s.chars() {
        match c {
            '0'...'9' => {
                match 10_u64.checked_mul(value) {
                    None => return Err(ParseAmountError::TooBig),
                    Some(val) => match val.checked_add((c as u8 - b'0') as u64) {
                        None => return Err(ParseAmountError::TooBig),
                        Some(val) => value = val,
                    },
                }
                decimals = match decimals {
                    None => None,
                    Some(d) if d < max_decimals => Some(d + 1),
                    _ => return Err(ParseAmountError::TooPrecise),
                };
            }
            '.' => match decimals {
                None => decimals = Some(0),
                // double decimal dot
                _ => return Err(ParseAmountError::InvalidFormat),
            },
            c => return Err(ParseAmountError::InvalidCharacters(c)),
        }
    }

    let scale_factor = max_decimals - decimals.unwrap_or(0);
    for _ in 0..scale_factor {
        value = match 10_u64.checked_mul(value) {
            Some(v) => v,
            None => return Err(ParseAmountError::TooBig),
        };
    }
    Ok((is_negative, value))
}

fn is_too_precise(s: &str, precision: usize) -> bool {
    // Returns true if the string has a decimal, the given
    // precision is greater than the length of the string,
    // or any of the last [precision] characters in the string are not `0`
    s.contains(".") || precision > s.len() || s.chars().rev().take(precision).any(|d| d != '0')
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
        let ssat = SignedAmount::from_sat;
        assert_eq!(format!("{:?}", sat(15)), "Amount(15 satoshi)");
        assert_eq!(format!("{:?}", ssat(15)), "SignedAmount(15 satoshi)");
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
        assert_eq!(ssat(-5).checked_div(2), Some(ssat(-2)));

        assert_eq!(ssat(-5).positive_sub(ssat(3)), None);
        assert_eq!(ssat(5).positive_sub(ssat(-3)), None);
        assert_eq!(ssat(3).positive_sub(ssat(5)), None);
        assert_eq!(ssat(3).positive_sub(ssat(3)), Some(ssat(0)));
        assert_eq!(ssat(5).positive_sub(ssat(3)), Some(ssat(2)));
    }

    #[test]
    fn test_parse_signed_to_satoshi() {
        assert_eq!(
            parse_signed_to_satoshi("1", Denomination::Bitcoin).unwrap(),
            (false, 100000000)
        );
        assert_eq!(
            parse_signed_to_satoshi("-1", Denomination::Bitcoin).unwrap(),
            (true, 100000000)
        );
        assert_eq!(
            parse_signed_to_satoshi("-100", Denomination::Bitcoin).unwrap(),
            (true, 10000000000)
        );
        assert_eq!(
            parse_signed_to_satoshi("100", Denomination::MilliSatoshi).unwrap(),
            (false, 100000)
        );
        assert_eq!(
            parse_signed_to_satoshi(".0000100", Denomination::Satoshi).unwrap_err(),
            ParseAmountError::TooPrecise
        );
        assert_eq!(
            parse_signed_to_satoshi("-", Denomination::Satoshi).unwrap_err(),
            ParseAmountError::InvalidFormat
        );
        assert_eq!(
            parse_signed_to_satoshi("", Denomination::Satoshi).unwrap_err(),
            ParseAmountError::InvalidFormat
        );
        assert_eq!(
            parse_signed_to_satoshi(
                "100000000000000000000000000000000000000000000000000000000000000000000000000000000",
                Denomination::Satoshi
            )
            .unwrap_err(),
            ParseAmountError::InputTooLarge
        );
        assert_eq!(
            parse_signed_to_satoshi("1..0", Denomination::Satoshi).unwrap_err(),
            ParseAmountError::InvalidFormat
        );
        assert_eq!(
            parse_signed_to_satoshi("c", Denomination::Satoshi).unwrap_err(),
            ParseAmountError::InvalidCharacters("c".chars().next().unwrap())
        );
    }

    #[test]
    fn floating_point() {
        //        use super::Denomination as D;
        //        let f = Amount::from_float_in;
        //        let sf = SignedAmount::from_float_in;
        //        let sat = Amount::from_sat;
        //        let ssat = SignedAmount::from_sat;
        // TODO fill out these tests
    }
}
