use std::collections::HashMap;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[derive(Debug)]
pub(crate) struct Hooks {
    hooks: HashMap<String, syn::ImplItemFn>,
}

impl Hooks {
    pub const BEFORE_ALL: &'static str = "before_all";
    pub const BEFORE_EACH: &'static str = "before_each";
    pub const AFTER_EACH: &'static str = "after_each";
    pub const AFTER_ALL: &'static str = "after_all";

    pub const ALL_HOOKS: &[&'static str] = &[
        Self::BEFORE_ALL,
        Self::BEFORE_EACH,
        Self::AFTER_EACH,
        Self::AFTER_ALL,
    ];

    pub fn new() -> Self {
        Self {
            hooks: Default::default(),
        }
    }

    pub fn is_hook(kind: &str) -> bool {
        Self::ALL_HOOKS.contains(&kind)
    }

    pub fn add_hook(&mut self, kind: &str, method: syn::ImplItemFn) -> syn::Result<()> {
        if !Self::is_hook(kind) {
            return Err(syn::Error::new(
                method.sig.ident.span(),
                format!("Invalid hook kind: {}", kind),
            ));
        }
        if self.hooks.contains_key(kind) {
            return Err(syn::Error::new(
                method.sig.ident.span(),
                format!("Duplicate hook: {}", kind),
            ));
        }
        self.hooks.insert(kind.to_string(), method);

        Ok(())
    }

    pub fn render(&self, struct_ty_name: &syn::Ident) -> TokenStream2 {
        let rendered = self.hooks.iter().map(|(kind, method)| {
            let fn_name = &method.sig.ident;
            let kind = quote::format_ident!("{}", kind);
            quote! {
                async fn #kind(&self) -> anyhow::Result<()> {
                    #struct_ty_name::#fn_name(&self).await
                }
            }
        });
        quote! {
            #(#rendered)*
        }
    }
}
