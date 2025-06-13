use syn::{FnArg, ImplItemFn, Type, spanned::Spanned as _};

#[derive(Debug)]
pub(crate) struct Constructor {
    pub(crate) config_ty_name: syn::Ident,
    pub(crate) constructor_fn_name: syn::Ident,
}

impl Constructor {
    pub const ID: &'static str = "constructor";

    pub fn new(method: ImplItemFn) -> syn::Result<Self> {
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
            config_ty_name,
            constructor_fn_name,
        })
    }
}
