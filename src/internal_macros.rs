//! Macros for internal use in this library

macro_rules! impl_consensus_encoding {
        ($thing:ident, $($field:ident),+) => (
            impl ::consensus::Encodable for $thing {
                #[inline]
                fn consensus_encode<S: ::std::io::Write>(
                    &self,
                    mut s: S,
                ) -> Result<usize, ::consensus::encode::Error> {
                    let mut len = 0;
                    $(len += self.$field.consensus_encode(&mut s)?;)+
                    Ok(len)
                }
            }

            impl ::consensus::Decodable for $thing {
                #[inline]
                fn consensus_decode<D: ::std::io::Read>(
                    mut d: D
                ) -> Result<$thing, ::consensus::encode::Error> {
                    Ok($thing {
                        $($field: ::consensus::Decodable::consensus_decode(&mut d)?),+
                    })
                }
            }
        )
}

macro_rules! display_from_debug {
    ($thing:ident) => {
        impl fmt::Display for $thing {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(self, f)
            }
        }
    };
}

macro_rules! user_enum {
    (
        $(#[$attr:meta])*
        pub enum $name:ident {
            $(#[$doc:meta]
              $elem:ident <-> $txt:expr),*
        }
    ) => (
        $(#[$attr])*
        pub enum $name {
            $(#[$doc] $elem),*
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.pad(match *self {
                    $($name::$elem => $txt),*
                })
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.pad(match *self {
                    $($name::$elem => $txt),*
                })
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = ::std::io::Error;
            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($txt => Ok($name::$elem)),*,
                    _ => Err(::std::io::Error::new(
                        ::std::io::ErrorKind::InvalidInput,
                        format!("Unknown network (type {})", s),
                    )),
                }
            }
        }

         #[cfg(feature = "serde")]
        impl<'de> $crate::serde::Deserialize<'de> for $name {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: $crate::serde::Deserializer<'de>,
            {
                // TODO implement this:
                unimplemented!();
            }
        }

        #[cfg(feature = "serde")]
        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                // TODO implement this:
                unimplemented!();
            }
        }
    );
}
