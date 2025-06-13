use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{FnArg, ImplItemFn, Type, spanned::Spanned as _};

#[derive(Debug)]
pub(crate) struct Constructor {
    pub(crate) name: Option<String>,
    pub(crate) config_ty_name: syn::Ident,
    pub(crate) constructor_fn_name: syn::Ident,
    pub(crate) method: ImplItemFn,
}

impl Constructor {
    pub const ID: &'static str = "constructor";

    pub fn new(method: ImplItemFn, attr: &syn::Attribute) -> syn::Result<Self> {
        let mut name = None;
        if let syn::Meta::List(list) = &attr.meta {
            let lit: syn::Lit = list.parse_args().map_err(|_| {
                syn::Error::new(
                    attr.span(),
                    "Expected a single identifier as the constructor name",
                )
            })?;
            if let syn::Lit::Str(lit_str) = lit {
                name = Some(lit_str.value());
            } else {
                return Err(syn::Error::new(
                    lit.span(),
                    "Expected a string literal for the constructor name",
                ));
            }
        }

        let config_ty = method.sig.inputs.first().cloned().ok_or_else(|| {
            syn::Error::new(
                method.sig.span(),
                "Constructor method must have a single argument for the config type",
            )
        })?;
        let config_ty = if let FnArg::Typed(pat_type) = config_ty {
            pat_type.ty
        } else {
            return Err(syn::Error::new(
                method.sig.span(),
                "Constructor method must have a single argument for the config type",
            ));
        };
        let Type::Reference(config_ty) = *config_ty else {
            return Err(syn::Error::new(
                method.sig.span(),
                "Constructor method must take a reference to the config type as an argument",
            ));
        };
        let Type::Path(config_ty) = *config_ty.elem else {
            return Err(syn::Error::new(
                method.sig.span(),
                "Constructor method must take a reference to the config type as an argument",
            ));
        };
        let config_ty_name = config_ty
            .path
            .get_ident()
            .ok_or_else(|| {
                syn::Error::new(
                    config_ty.span(),
                    "Constructor method must take a reference to a named config type",
                )
            })?
            .clone();
        let constructor_fn_name = method.sig.ident.clone();

        Ok(Self {
            name,
            config_ty_name,
            constructor_fn_name,
            method,
        })
    }

    pub fn render(
        &self,
        suite_name: &syn::Lit,
        crate_name: &syn::Ident,
        struct_ty_name: &syn::Ident,
    ) -> TokenStream2 {
        let config_ty_name = &self.config_ty_name;
        let constructor_fn_name = &self.constructor_fn_name;
        let constructor_fn_name_inner = quote::format_ident!("__{}", self.constructor_fn_name);
        let mut method = self.method.clone();
        method.sig.ident = constructor_fn_name_inner.clone();

        let factory_name =
            quote::format_ident!("{}Factory_{}", struct_ty_name, constructor_fn_name);

        let suite_name_code = if let Some(name) = &self.name {
            quote! {
                format!("{} ({})", #suite_name, #name)
            }
        } else {
            quote! {
                #suite_name.to_string()
            }
        };

        quote! {
            #[allow(non_camel_case_types)]
            struct #factory_name;

            impl #struct_ty_name {
                pub fn #constructor_fn_name() -> Box<dyn #crate_name::TestSuiteFactory<#config_ty_name>> {
                    Box::new(#factory_name)
                }

                #method
            }

            #[#crate_name::__private_reexports::async_trait]
            impl #crate_name::TestSuiteFactory<#config_ty_name> for #factory_name {
                fn name(&self) -> String {
                    #suite_name_code
                }

                async fn create_suite(&self, config: &#config_ty_name) -> anyhow::Result<Box<dyn #crate_name::TestSuite>> {
                    let self_ = #struct_ty_name::#constructor_fn_name_inner(config).await?;
                    Ok(Box::new(self_))
                }
            }

            impl std::fmt::Debug for #factory_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", <Self as #crate_name::TestSuiteFactory<#config_ty_name>>::name(self))
                }
            }
        }
    }
}
