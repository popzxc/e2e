use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, ExprLit, Token, punctuated::Punctuated, spanned::Spanned as _};

#[derive(Debug)]
pub(crate) struct TestCase {
    pub(crate) name: String,
    pub(crate) method: syn::ImplItemFn,
    pub(crate) ignore: bool,
    pub(crate) only: bool,
}

impl TestCase {
    pub const ID: &'static str = "test_case";

    pub fn new(method: syn::ImplItemFn, attr: &syn::Attribute) -> syn::Result<Self> {
        let arguments: Punctuated<Expr, Token![,]> = attr
            .parse_args_with(Punctuated::parse_terminated)
            .map_err(|_| {
                syn::Error::new(attr.span(), "`test_case` attribute must contain test name")
            })?;
        if arguments.is_empty() {
            return Err(syn::Error::new(
                attr.span(),
                "`test_case` attribute must contain test name",
            ));
        }
        let Expr::Lit(lit) = arguments.first().unwrap() else {
            return Err(syn::Error::new(
                arguments.span(),
                "`test_case` attribute must contain a string literal as the test name",
            ));
        };
        let ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        } = lit
        else {
            return Err(syn::Error::new(
                lit.span(),
                "`test_case` attribute must contain a string literal as the test name",
            ));
        };

        let name = lit_str.value();
        if name.is_empty() {
            return Err(syn::Error::new(
                lit.span(),
                "Test case name cannot be empty",
            ));
        }

        let mut ignore = false;
        let mut only = false;
        for arg in arguments.iter().skip(1) {
            if let Expr::Path(path) = arg {
                if path.path.is_ident("ignore") {
                    ignore = true;
                } else if path.path.is_ident("only") {
                    only = true;
                } else {
                    return Err(syn::Error::new(
                        path.span(),
                        "Unknown argument in `test_case` attribute",
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    arg.span(),
                    "`test_case` attribute arguments must be identifiers",
                ));
            }
        }

        Ok(Self {
            name,
            method,
            ignore,
            only,
        })
    }

    pub fn render(
        &self,
        struct_ty_name: &syn::Ident,
        crate_name: &syn::Ident,
    ) -> (TokenStream2, TokenStream2) {
        let test_fn_name = &self.method.sig.ident;
        let name = &self.name;
        let ignore = self.ignore;
        let only = self.only;

        let test_ty_name = quote::format_ident!(
            "{}_Test_{}",
            struct_ty_name,
            name.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        );
        let test_case = quote! {
            #[allow(non_camel_case_types)]
            struct #test_ty_name(#struct_ty_name);

            #[#crate_name::__private_reexports::async_trait]
            impl #crate_name::Test for #test_ty_name {
                fn name(&self) -> String {
                    #name.to_string()
                }

                async fn run(&self) -> anyhow::Result<()> {
                    self.0.#test_fn_name().await
                }

                fn ignore(&self) -> bool {
                    #ignore
                }

                fn only(&self) -> bool {
                    #only
                }
            }
        };
        let test_case_objects = quote! {
            Box::new(#test_ty_name(self.clone()))
        };
        (test_case, test_case_objects)
    }
}
