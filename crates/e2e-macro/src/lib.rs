extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::test_suite::TestSuite;

mod constructor;
mod hooks;
mod test_case;
mod test_suite;

#[proc_macro_attribute]
pub fn test_suite(attr: TokenStream, item: TokenStream) -> TokenStream {
    let suite_name = parse_macro_input!(attr as syn::Lit);
    let input = parse_macro_input!(item as syn::ItemImpl);
    test_suite_impl(suite_name, input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn test_suite_impl(
    suite_name: syn::Lit,
    input: syn::ItemImpl,
) -> syn::Result<proc_macro2::TokenStream> {
    let suite = TestSuite::from_impl(suite_name, input)?;
    Ok(suite.render())
}
