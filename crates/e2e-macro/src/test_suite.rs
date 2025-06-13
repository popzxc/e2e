use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ImplItem, ItemImpl, spanned::Spanned as _};

use crate::{constructor::Constructor, hooks::Hooks, test_case::TestCase};

fn is_special_attr(attr: &syn::Attribute) -> bool {
    attr.meta.path().is_ident(Constructor::ID)
        || attr.meta.path().is_ident(TestCase::ID)
        || Hooks::ALL_HOOKS
            .iter()
            .any(|&hook| attr.meta.path().is_ident(hook))
}

#[derive(Debug)]
pub(crate) struct TestSuite {
    input: syn::ItemImpl,
    crate_name: syn::Ident,
    suite_name: syn::Lit,
    struct_ty_name: syn::Ident,
    constructor: Constructor,
    hooks: Hooks,
    test_cases: Vec<TestCase>,
    cleaned_items: Vec<ImplItem>,
}

impl TestSuite {
    fn struct_ty_name(input: &syn::ItemImpl) -> syn::Result<syn::Ident> {
        let struct_ty = input.self_ty.clone();
        let syn::Type::Path(struct_ty) = *struct_ty else {
            return Err(syn::Error::new(
                struct_ty.span(),
                "The test suite must be implemented for a struct type",
            ));
        };
        struct_ty
            .path
            .get_ident()
            .cloned()
            .ok_or_else(|| syn::Error::new(struct_ty.span(), "Expected a struct name"))
    }

    pub fn from_impl(suite_name: syn::Lit, input: syn::ItemImpl) -> syn::Result<Self> {
        let struct_ty_name = Self::struct_ty_name(&input)?;

        let mut constructor = None;
        let mut hooks = Hooks::new();
        let mut test_cases = vec![];

        let mut cleaned_items = vec![];

        for item in &input.items {
            if let ImplItem::Fn(mut method) = item.clone() {
                let mut attr = None;
                let mut n_special_attrs = 0;
                method.attrs.retain(|a| {
                    if is_special_attr(a) {
                        attr = Some(a.clone());
                        n_special_attrs += 1;
                        false
                    } else {
                        true
                    }
                });
                if n_special_attrs > 1 {
                    return Err(syn::Error::new(
                        method.sig.span(),
                        "A method cannot have multiple test-related attributes",
                    ));
                }

                if let Some(attr) = attr {
                    let ident = attr
                        .meta
                        .path()
                        .get_ident()
                        .expect("Ident must be present")
                        .to_string();
                    if ident == Constructor::ID {
                        let constructor_obj = Constructor::new(method.clone())?;
                        constructor = Some(constructor_obj);
                    } else if ident == TestCase::ID {
                        let test_case = TestCase::new(method.clone(), &attr)?;
                        test_cases.push(test_case);
                    } else if Hooks::is_hook(&ident) {
                        hooks
                            .add_hook(&ident, method.clone())
                            .expect("Failed to add hook");
                    }
                }
                cleaned_items.push(ImplItem::Fn(method.clone()));
            } else {
                cleaned_items.push(item.clone());
            }
        }

        let constructor = constructor.unwrap_or_else(|| {
            panic!("A test suite must have a constructor method annotated with #[constructor]");
        });

        let crate_name = quote::format_ident!("e2e");

        Ok(Self {
            input,
            crate_name,
            suite_name,
            constructor,
            hooks,
            struct_ty_name,
            test_cases,
            cleaned_items,
        })
    }

    fn render_test_cases(&self) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
        let crate_name = &self.crate_name;
        let struct_ty_name = &self.struct_ty_name;
        let test_cases = &self.test_cases;
        let mut test_case_code = Vec::new();
        let mut test_case_objects = Vec::new();
        for (
            id,
            TestCase {
                name,
                method,
                ignore,
            },
        ) in test_cases.iter().enumerate()
        {
            let test_fn_name = &method.sig.ident;

            let test_ty_name = quote::format_ident!("{}Test{}", struct_ty_name, id);
            let test_case = quote! {
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
                }
            };
            test_case_code.push(test_case);
            test_case_objects.push(quote! {
                Box::new(#test_ty_name(self.clone()))
            });
        }

        (test_case_code, test_case_objects)
    }

    fn render_factory(&self) -> TokenStream2 {
        let suite_name = &self.suite_name;
        let crate_name = &self.crate_name;
        let struct_ty_name = &self.struct_ty_name;
        let config_ty_name = &self.constructor.config_ty_name;
        let constructor_fn_name = &self.constructor.constructor_fn_name;

        let factory_name = quote::format_ident!("{}Factory", struct_ty_name);

        quote! {
            struct #factory_name;

            impl #struct_ty_name {
                pub fn factory() -> Box<dyn #crate_name::TestSuiteFactory<#config_ty_name>> {
                    Box::new(#factory_name)
                }
            }

            #[#crate_name::__private_reexports::async_trait]
            impl #crate_name::TestSuiteFactory<#config_ty_name> for #factory_name {
                fn name(&self) -> String {
                    #suite_name.to_string()
                }

                /// Creates a new test suite instance.
                async fn create_suite(&self, config: &#config_ty_name) -> anyhow::Result<Box<dyn #crate_name::TestSuite>> {
                    let self_ = #struct_ty_name::#constructor_fn_name(config).await?;
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

    fn render_test_suite(&self, test_case_objects: Vec<TokenStream2>) -> TokenStream2 {
        let crate_name = &self.crate_name;
        let struct_ty_name = &self.struct_ty_name;
        let suite_name = &self.suite_name;

        let hooks = self.hooks.render(struct_ty_name);

        quote! {
            #[#crate_name::__private_reexports::async_trait]
            impl #crate_name::TestSuite for #struct_ty_name {
                fn name(&self) -> String {
                    #suite_name.to_string()
                }

                fn tests(&self) -> Vec<Box<dyn #crate_name::Test>> {
                    vec![
                        #(#test_case_objects),*
                    ]
                }

                #hooks
            }
        }
    }

    pub fn render(self) -> TokenStream2 {
        let factory = self.render_factory();
        let (test_case_code, test_case_objects) = self.render_test_cases();
        let test_suite = self.render_test_suite(test_case_objects);

        let cleaned_impl = ItemImpl {
            items: self.cleaned_items,
            ..self.input
        };

        quote! {
            #cleaned_impl

            #factory

            #test_suite

            #(#test_case_code)*
        }
    }
}
