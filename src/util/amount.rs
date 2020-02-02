use std::fmt::{self, Display, Formatter, Write};
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
            Denomination::MilliSatoshi => 3,
        }
    }
}

impl Display for Denomination {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
    /// Amount is negative (only an error if using [Amount])
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
    InvalidCharacter(char),
    /// The denomination didn't match our known ones
    UnknownDenomination(String),
}

/// Can be used to represent Bitcoin amounts. Supports
/// arithmetic operations.
#[derive(Copy, Clone, Hash, PartialEq, SatoshiArithmetic)]
pub struct Amount(u64);

impl Amount {
    /// Parse a decimal string as a value in a given denomination
    pub fn from_str_in(s: &str, denom: Denomination) -> Result<Amount, ParseAmountError> {
        let (negative, satoshi) = parse_signed_to_satoshi(s, denom)?;
        // Gotta use [SignedAmount] for negative amounts
        if negative {
            return Err(ParseAmountError::Negative);
        }
        if satoshi > i64::max_value() as u64 {
            return Err(ParseAmountError::TooBig);
        }
        Ok(Amount::from_sat(satoshi))
    }

    /// Turns a float into an [Amount]
    pub fn from_float_in(value: f64, denom: Denomination) -> Result<Amount, ParseAmountError> {
        if value < 0.0 {
            // gotta use [SignedAmount] for negative values
            return Err(ParseAmountError::Negative);
        }
        // Relying on string parsing is the safest way to parse a float.
        // apparently float parsing is tricky due to `halfway cases`
        Amount::from_str_in(&value.to_string(), denom)
    }

    /// Format the value of this [Amount] in the given denomination.
    pub fn fmt_value_in(&self, f: &mut dyn Write, denom: Denomination) -> fmt::Result {
        fmt_satoshi_in(self.as_sat(), false, f, denom)
    }
}

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

    /// Format the value of this [SignedAmount] in the given denomination.
    pub fn fmt_value_in(&self, f: &mut dyn Write, denom: Denomination) -> fmt::Result {
        fmt_satoshi_in(self.as_sat().abs() as u64, self.is_negative(), f, denom)
    }
}

/// Parses a decimal string in the given denomination into a satoshi value
/// and a boolean that indicates whether it's a negative amount
/// Parse decimal string in the given denomination into a satoshi value and a
/// bool indicator for a negative amount.
fn parse_signed_to_satoshi(
    mut s: &str,
    denom: Denomination,
) -> Result<(bool, u64), ParseAmountError> {
    if s.len() == 0 {
        return Err(ParseAmountError::InvalidFormat);
    }
    if s.len() > 50 {
        return Err(ParseAmountError::InputTooLarge);
    }

    let is_negative = s.chars().next().unwrap() == '-';
    if is_negative {
        if s.len() == 1 {
            return Err(ParseAmountError::InvalidFormat);
        }
        s = &s[1..];
    }

    let max_decimals = {
        // The difference in precision between native (satoshi)
        // and desired denomination.
        let precision_diff = -denom.precision();
        if precision_diff < 0 {
            // If precision diff is negative, this means we are parsing
            // into a less precise amount. That is not allowed unless
            // there are no decimals and the last digits are zeroes as
            // many as the difference in precision.
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
            '0'..='9' => {
                // Do `value = 10 * value + digit`, catching overflows.
                match 10_u64.checked_mul(value) {
                    None => return Err(ParseAmountError::TooBig),
                    Some(val) => match val.checked_add((c as u8 - b'0') as u64) {
                        None => return Err(ParseAmountError::TooBig),
                        Some(val) => value = val,
                    },
                }
                // Increment the decimal digit counter if past decimal.
                decimals = match decimals {
                    None => None,
                    Some(d) if d < max_decimals => Some(d + 1),
                    _ => return Err(ParseAmountError::TooPrecise),
                };
            }
            '.' => match decimals {
                None => decimals = Some(0),
                // Double decimal dot.
                _ => return Err(ParseAmountError::InvalidFormat),
            },
            c => return Err(ParseAmountError::InvalidCharacter(c)),
        }
    }

    // Decimally shift left by `max_decimals - decimals`.
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

/// Format the given satoshi amount in the given denomination.
fn fmt_satoshi_in(
    satoshi: u64,
    negative: bool,
    f: &mut dyn Write,
    denom: Denomination,
) -> fmt::Result {
    if negative {
        f.write_str("-")?;
    }

    let sat_precision = Denomination::Satoshi.precision();

    if denom.precision() > sat_precision {
        // add zeroes in the end
        let width = denom.precision() as usize;
        write!(f, "{}{:0width$}", satoshi, 0, width = width)?;
    } else if denom.precision() < 0 {
        // need to inject a comma in the number
        let nb_decimals = denom.precision().abs() as usize;
        let real = format!("{:0width$}", satoshi, width = nb_decimals);
        if real.len() == nb_decimals {
            write!(f, "0.{}", &real[real.len() - nb_decimals..])?;
        } else {
            write!(
                f,
                "{}.{}",
                &real[0..(real.len() - nb_decimals)],
                &real[real.len() - nb_decimals..]
            )?;
        }
    } else {
        write!(f, "{}", satoshi)?;
    }
    Ok(())
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
            parse_signed_to_satoshi("-900", Denomination::Bitcoin).unwrap(),
            (true, 90000000000)
        );
        assert_eq!(
            parse_signed_to_satoshi("10000", Denomination::MilliSatoshi).unwrap(),
            (false, 10)
        );
        assert_eq!(
            // 100 millisatoshis would be like .0001 satoshis or something like that. can't
            // have fractional satoshis.
            parse_signed_to_satoshi("100", Denomination::MilliSatoshi).unwrap_err(),
            ParseAmountError::TooPrecise
        );
        assert_eq!(
            // 100 millisatoshis would be like .0000001 satoshis or something like that. can't
            // have fractional satoshis.
            parse_signed_to_satoshi(".001", Denomination::MilliSatoshi).unwrap_err(),
            ParseAmountError::TooPrecise
        );
        assert_eq!(
            parse_signed_to_satoshi(".0000100", Denomination::Satoshi).unwrap_err(),
            ParseAmountError::TooPrecise
        );
        assert_eq!(
            parse_signed_to_satoshi(".0000100", Denomination::Bitcoin).unwrap(),
            (false, 1000)
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
            ParseAmountError::InvalidCharacter("c".chars().next().unwrap())
        );
        assert_eq!(
            parse_signed_to_satoshi(&*format!("{}", i64::max_value()), Denomination::Bitcoin)
                .unwrap_err(),
            ParseAmountError::TooBig
        );
    }

    #[test]
    fn floating_point() {
        use super::Denomination as D;
        let f = Amount::from_float_in;
        let sf = SignedAmount::from_float_in;
        let sat = Amount::from_sat;
        let ssat = SignedAmount::from_sat;

        // Successful parsing:
        assert_eq!(f(11.22, D::Bitcoin), Ok(sat(1122000000)));
        assert_eq!(sf(-11.22, D::MilliBitcoin), Ok(ssat(-1122000)));
        assert_eq!(f(11.22, D::Bit), Ok(sat(1122)));
        assert_eq!(sf(-1000.0, D::MilliSatoshi), Ok(ssat(-1)));
        assert_eq!(f(0.0001234, D::Bitcoin), Ok(sat(12340)));
        assert_eq!(sf(-0.00012345, D::Bitcoin), Ok(ssat(-12345)));

        // Failed parsing:
        assert_eq!(f(-100.0, D::MilliSatoshi), Err(ParseAmountError::Negative));
        assert_eq!(f(11.22, D::Satoshi), Err(ParseAmountError::TooPrecise));
        assert_eq!(
            sf(-100.0, D::MilliSatoshi),
            Err(ParseAmountError::TooPrecise)
        );
        assert_eq!(
            f(42.123456781, D::Bitcoin),
            Err(ParseAmountError::TooPrecise)
        );
        assert_eq!(
            sf(-184467440738.0, D::Bitcoin),
            Err(ParseAmountError::TooBig)
        );
        assert_eq!(
            f(18446744073709551617.0, D::Satoshi),
            Err(ParseAmountError::TooBig)
        );
        assert_eq!(
            f(
                SignedAmount::max_value().to_float_in(D::Satoshi) + 1.0,
                D::Satoshi
            ),
            Err(ParseAmountError::TooBig)
        );
        assert_eq!(
            f(
                Amount::max_value().to_float_in(D::Satoshi) + 1.0,
                D::Satoshi
            ),
            Err(ParseAmountError::TooBig)
        );

        let btc = move |f| SignedAmount::from_btc(f).unwrap();
        assert_eq!(btc(2.5).to_float_in(D::Bitcoin), 2.5);
        assert_eq!(btc(-2.5).to_float_in(D::MilliBitcoin), -2500.0);
        assert_eq!(btc(2.5).to_float_in(D::Satoshi), 250000000.0);
        assert_eq!(btc(-2.5).to_float_in(D::MilliSatoshi), -250000000000.0);

        let btc = move |f| Amount::from_btc(f).unwrap();
        assert_eq!(&btc(0.0012).to_float_in(D::Bitcoin).to_string(), "0.0012");
    }

    #[test]
    fn test_fmt_satoshi_in() {
        let mut buf = String::new();
        fmt_satoshi_in(100, false, &mut buf, Denomination::Satoshi);
        assert_eq!(buf, "100");
        let mut buf = String::new();
        fmt_satoshi_in(1000, true, &mut buf, Denomination::Satoshi);
        assert_eq!(buf, "-1000");
        let mut buf = String::new();
        fmt_satoshi_in(1000, true, &mut buf, Denomination::MilliSatoshi);
        assert_eq!(buf, "-1000000");
        let mut buf = String::new();
        fmt_satoshi_in(1000, true, &mut buf, Denomination::Bitcoin);
        assert_eq!(buf, "-0.00001000");
    }

    #[test]
    fn test_parsing() {
        use super::ParseAmountError as E;
        let btc = Denomination::Bitcoin;
        let p = Amount::from_str_in;
        let sp = SignedAmount::from_str_in;

        assert_eq!(p("x", btc), Err(E::InvalidCharacter('x')));
        assert_eq!(p("-", btc), Err(E::InvalidFormat));
        assert_eq!(sp("-", btc), Err(E::InvalidFormat));
        assert_eq!(p("-1.0x", btc), Err(E::InvalidCharacter('x')));
        assert_eq!(p("0.0 ", btc), Err(E::InvalidCharacter(' ')));
        assert_eq!(p("0.000.000 ", btc), Err(E::InvalidFormat));
        let max = format!("{}", i64::max_value());
        assert_eq!(p(&max, btc), Err(E::TooBig));
        let more_than_max = format!("1{}", Amount::max_value());
        assert_eq!(p(&more_than_max, btc), Err(E::TooBig));
        assert_eq!(p("0.000000042", btc), Err(E::TooPrecise));

        assert_eq!(p("1", btc), Ok(Amount::from_sat(100_000_000)));
        assert_eq!(sp("-.5", btc), Ok(SignedAmount::from_sat(-50_000_000)));
        assert_eq!(p("1.1", btc), Ok(Amount::from_sat(110_000_000)));
        assert_eq!(
            p("12345678901.12345678", btc),
            Ok(Amount::from_sat(12_345_678_901__123_456_78))
        );
        assert_eq!(p("12", Denomination::MilliSatoshi), Err(E::TooPrecise));
    }

    #[test]
    fn test_to_string() {
        use super::Denomination as D;

        assert_eq!(Amount::ONE_BTC.to_string_in(D::Bitcoin), "1.00000000");
        assert_eq!(Amount::ONE_BTC.to_string_in(D::Satoshi), "100000000");
        assert_eq!(Amount::ONE_SAT.to_string_in(D::Bitcoin), "0.00000001");
        assert_eq!(
            SignedAmount::from_sat(-42).to_string_in(D::Bitcoin),
            "-0.00000042"
        );

        assert_eq!(
            Amount::ONE_BTC.to_string_with_denomination(D::Bitcoin),
            "1.00000000 BTC"
        );
        assert_eq!(
            Amount::ONE_SAT.to_string_with_denomination(D::MilliSatoshi),
            "1000 msat"
        );
        assert_eq!(
            Amount::ONE_BTC.to_string_with_denomination(D::Satoshi),
            "100000000 satoshi"
        );
        assert_eq!(
            Amount::ONE_SAT.to_string_with_denomination(D::Bitcoin),
            "0.00000001 BTC"
        );
        assert_eq!(
            SignedAmount::from_sat(-42).to_string_with_denomination(D::Bitcoin),
            "-0.00000042 BTC"
        );
    }

    #[test]
    fn test_from_string() {
        use super::ParseAmountError as E;
        let p = Amount::from_str;
        let sp = SignedAmount::from_str;

        assert_eq!(p("x BTC"), Err(E::InvalidCharacter('x')));
        assert_eq!(p("5 BTC BTC"), Err(E::InvalidFormat));
        assert_eq!(p("5 5 BTC"), Err(E::InvalidFormat));

        assert_eq!(p("5 BCH"), Err(E::UnknownDenomination("BCH".to_owned())));

        assert_eq!(p("-1 BTC"), Err(E::Negative));
        assert_eq!(p("-0.0 BTC"), Err(E::Negative));
        assert_eq!(p("0.123456789 BTC"), Err(E::TooPrecise));
        assert_eq!(sp("-0.1 satoshi"), Err(E::TooPrecise));
        assert_eq!(p("0.123456 mBTC"), Err(E::TooPrecise));
        assert_eq!(sp("-1.001 bits"), Err(E::TooPrecise));
        assert_eq!(sp("-200000000000 BTC"), Err(E::TooBig));
        assert_eq!(p("18446744073709551616 BTC"), Err(E::TooBig));

        assert_eq!(sp("0 msat"), Err(E::TooPrecise));
        assert_eq!(sp("-0 msat"), Err(E::TooPrecise));
        // TODO THESE SHOULD FAIL:
        //        assert_eq!(sp("000 msat"), Err(E::TooPrecise));
        //        assert_eq!(sp("-000 msat"), Err(E::TooPrecise));
        assert_eq!(p("0 msat"), Err(E::TooPrecise));
        assert_eq!(p("-0 msat"), Err(E::TooPrecise));
        // TODO THESE SHOULD FAIL:
        //        assert_eq!(p("000 msat"), Err(E::TooPrecise));
        //        assert_eq!(p("-000 msat"), Err(E::TooPrecise));
    }
}
