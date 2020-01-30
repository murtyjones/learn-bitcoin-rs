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
            /// Performs a 'checked' addition. Returns `None` if an overflow occurs.
            /// @see https://rust-num.github.io/num/num_traits/ops/checked/trait.CheckedAdd.html
            pub fn checked_add(self, rhs: #struct_name) -> Option<#struct_name> {
                self.0.checked_add(rhs.0).map(#struct_name)
            }
        }

        /// Implements the `+` operator using a checked addition for Amount instances.
        impl ops::Add for #struct_name {
            type Output = Self;

            fn add(self, rhs: #struct_name) -> Self::Output {
                self.checked_add(rhs).expect("Whoops! Addition error")
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
    };
    gen.into()
}
