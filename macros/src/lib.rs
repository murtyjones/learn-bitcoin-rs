extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate core;

use proc_macro::TokenStream;
use syn::Type::Path;

#[proc_macro_derive(SharedAmountTraits)]
pub fn arithmetic_derive(input: TokenStream) -> TokenStream {
    impl_formulate(&syn::parse(input).unwrap())
}

fn impl_formulate(ast: &syn::DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let data = match &ast.data {
        syn::Data::Struct(struct_data) => struct_data,
        _ => panic!("not a struct"),
    };
    let fields = match &data.fields {
        syn::Fields::Unnamed(unnamed_fields) => unnamed_fields,
        _ => panic!("this only works for tuple structs"),
    };
    let i64oru64 = match &fields
        .unnamed
        .iter()
        .next()
        .expect("This tuple struct must have at least one field!")
        .ty
    {
        Path(path) => path,
        _ => panic!("not a path"),
    };

    // "i64" or "u64"
    let num_type = &i64oru64.path.segments.iter().next().unwrap().ident;
    assert!(
        &num_type.to_string() == "i64" || &num_type.to_string() == "u64",
        "This macro only works for i64/u64"
    );

    let gen = quote! {
        impl #struct_name {
            /// Creates an Amount/SignedAmount object from a given number of satoshis
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
            /// The max allowed value of a Amount
            pub fn max_value() -> #struct_name {
                #struct_name(#num_type::max_value())
            }

            /// The min allowed value of a Amount
            pub fn min_value() -> #struct_name {
                #struct_name(#num_type::min_value())
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
    };
    gen.into()
}
