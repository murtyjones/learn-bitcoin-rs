extern crate proc_macro;

extern crate syn;
#[macro_use]
extern crate quote;
extern crate core;

use proc_macro::TokenStream;
use syn::Error;
use syn::Type::Path;

#[proc_macro_derive(SatoshiArithmetic)]
pub fn arithmetic_derive(input: TokenStream) -> TokenStream {
    impl_formulate(&syn::parse(input).unwrap())
}

fn impl_formulate(ast: &syn::DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let data = match &ast.data {
        syn::Data::Struct(struct_data) => Some(struct_data),
        _ => None,
    };
    if data.is_none() {
        return Error::new(ast.ident.span(), "Only works for structs!")
            .to_compile_error()
            .into();
    }
    let fields = match &data.unwrap().fields {
        syn::Fields::Unnamed(unnamed_fields) => Some(unnamed_fields),
        _ => None,
    };
    if fields.is_none() {
        return Error::new(ast.ident.span(), "Only work for tuple structs!")
            .to_compile_error()
            .into();
    }

    let field = &fields.unwrap().unnamed.iter().next();
    if field.is_none() {
        return Error::new(
            ast.ident.span(),
            "This tuple struct must have at least one field!",
        )
        .to_compile_error()
        .into();
    }

    let field_type = &field.unwrap().ty;

    let i64oru64 = match field_type {
        Path(path) => Some(path),
        _ => None,
    };

    if i64oru64.is_none() {
        return Error::new(ast.ident.span(), "expected a path in this struct!")
            .to_compile_error()
            .into();
    }

    let i64oru64 = i64oru64.unwrap();

    // "i64" or "u64"
    let num_type = &i64oru64.path.segments.iter().next().unwrap().ident;
    assert!(
        &num_type.to_string() == "i64" || &num_type.to_string() == "u64",
        "This macro only works for i64/u64"
    );
    if &num_type.to_string() != "i64" && &num_type.to_string() != "u64" {
        return Error::new(num_type.span(), "expected u64 or i64")
            .to_compile_error()
            .into();
    }

    let struct_name_string = struct_name.to_string();

    let gen = quote! {
        impl #struct_name {
            /// Creates an [Amount]|[SignedAmount] object from a given number of satoshis
            pub fn from_sat(satoshis: #num_type) -> #struct_name {
                #struct_name(satoshis)
            }
            /// Get the number of satoshis
            pub fn as_sat(self) -> #num_type {
                self.0
            }
            /// Performs a 'checked' addition. Returns `None` if an overflow occurs.
            /// @see https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedAdd.html
            pub fn checked_add(self, rhs: #struct_name) -> Option<#struct_name> {
                self.0.checked_add(rhs.0).map(#struct_name)
            }
            /// Performs a 'checked' subtraction. Returns `None` if an overflow occurs.
            /// @see https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedSub.html
            pub fn checked_sub(self, rhs: #struct_name) -> Option<#struct_name> {
                self.0.checked_sub(rhs.0).map(#struct_name)
            }
            /// Performs a checked multiplication, returning None if an overflow occurs.
            /// https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedMul.html
            pub fn checked_mul(self, rhs: #num_type) -> Option<#struct_name> {
                self.0.checked_mul(rhs).map(#struct_name)
            }
            /// Performs a checked division, returning None if an overflow occurs.
            /// https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedDiv.html
            /// NOTE: The remainder will be lost if no exact division takes place.
            pub fn checked_div(self, rhs: #num_type) -> Option<#struct_name> {
                self.0.checked_div(rhs).map(#struct_name)
            }
            /// Performs a checked remainder, returning None if an overflow occurs.
            /// https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedDiv.html
            pub fn checked_rem(self, rhs: #num_type) -> Option<#struct_name> {
                self.0.checked_rem(rhs).map(#struct_name)
            }
            /// The max allowed value of a [Amount]|[SignedAmount]
            pub fn max_value() -> #struct_name {
                #struct_name(#num_type::max_value())
            }

            /// The min allowed value of a [Amount]|[SignedAmount]
            pub fn min_value() -> #struct_name {
                #struct_name(#num_type::min_value())
            }

            /// Express this [Amount]|[SignedAmount] as a floating-point value in the given denomination.
            ///
            /// Be aware of the risk of using floating point numbers
            pub fn to_float_in(&self, denom: Denomination) -> f64 {
                f64::from_str(&self.to_string_in(denom)).unwrap()
            }

            /// Get a string number of this [Amount]|[SignedAmount] in the given denomination.
            pub fn to_string_in(&self, denom: Denomination) -> String {
                let mut buf = String::new();
                self.fmt_value_in(&mut buf, denom).unwrap();
                buf
            }
        }

        /// Allows us to display amounts for Satoshis and compare them in tests
        impl fmt::Debug for #struct_name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({} satoshi)", #struct_name_string, self.as_sat())
            }
        }

        /// Implements the `+` operator using a checked addition for Amount instances.
        impl ops::Add for #struct_name {
            type Output = Self;

            fn add(self, rhs: #struct_name) -> Self::Output {
                self.checked_add(rhs).expect("Whoops! Addition error")
            }
        }

        /// Allows us to use `*` to multiply Amounts
        impl ops::Mul<#num_type> for #struct_name {
            type Output = Self;

            fn mul(self, rhs: #num_type) -> Self {
                self.checked_mul(rhs).expect("Uh oh! Multiplcation error")
            }
        }

        /// Allows `*=`
        impl ops::MulAssign<#num_type> for #struct_name {
            fn mul_assign(&mut self, rhs: #num_type) {
                *self = *self * rhs
            }
        }

        /// Allows `+=`
        impl ops::AddAssign for #struct_name {
            fn add_assign(&mut self, other: #struct_name) {
                *self = *self + other
            }
        }

        /// Allows us to subtract one Amount from another using `-`
        impl ops::Sub for #struct_name {
            type Output = Self;

            fn sub(self, rhs: #struct_name) -> Self::Output {
                self.checked_sub(rhs).expect("Whoops! Subtraction error")
            }
        }

        /// Allows `-=`
        impl ops::SubAssign for #struct_name {
            fn sub_assign(&mut self, other: #struct_name) {
                *self = *self - other
            }
        }

        /// Allows us to use `/` to divide Amounts
        impl ops::Div<#num_type> for #struct_name {
            type Output = Self;

            fn div(self, rhs: #num_type) -> Self {
                self.checked_div(rhs).expect("Uh oh! Division error")
            }
        }

        /// Allows `/=`
        impl ops::DivAssign<#num_type> for #struct_name {
            fn div_assign(&mut self, rhs: #num_type) {
                *self = *self / rhs
            }
        }

        /// Allows us to use `%` to find remainders from dividing Amounts
        impl ops::Rem<#num_type> for #struct_name {
            type Output = Self;

            fn rem(self, modulus: #num_type) -> Self {
                self.checked_rem(modulus).expect("Uh oh! Remainder error")
            }
        }

        /// Allows `%=`
        impl ops::RemAssign<#num_type> for #struct_name {
            fn rem_assign(&mut self, other: #num_type) {
                *self = *self % other
            }
        }

        /// Allows comparison of [Amount]|[SignedAmount] using `<` `>`
        impl PartialOrd for #struct_name {
            fn partial_cmp(&self, other: &#struct_name) -> Option<::std::cmp::Ordering> {
                PartialOrd::partial_cmp(&self.0, &other.0)
            }
        }

        impl Ord for #struct_name {
            fn cmp(&self, other: &#struct_name) -> ::std::cmp::Ordering {
                Ord::cmp(&self.0, &other.0)
            }
        }

        impl Eq for #struct_name {}
    };
    gen.into()
}
