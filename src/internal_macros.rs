macro_rules! display_from_debug {
    ($thing:ident) => {
        impl fmt::Display for $thing {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(self, f)
            }
        }
    };
}
