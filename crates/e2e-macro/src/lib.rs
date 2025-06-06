extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, Type, parse_macro_input};

enum DetectedItem {
    Constructor,
    BeforeAll,
    BeforeEach,
    AfterEach,
    AfterAll,
    TestCase(String),
}

#[proc_macro_attribute]
pub fn test_suite(attr: TokenStream, item: TokenStream) -> TokenStream {
    test_suite_impl(attr, item)
}

fn test_suite_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let suite_name = parse_macro_input!(attr as syn::Lit);

    let input = parse_macro_input!(item as ItemImpl);

    let struct_ty = input.self_ty.clone();
    let Type::Path(struct_ty) = *struct_ty else {
        panic!("The test suite must be implemented for a struct type");
    };
    let struct_ty_name = struct_ty.path.get_ident().expect("Expected a struct name");

    let mut constructor = None;
    let mut before_all = None;
    let mut before_each = None;
    let mut after_each = None;
    let mut after_all = None;
    let mut test_cases = vec![];

    let mut cleaned_items = vec![];

    for item in &input.items {
        if let ImplItem::Fn(mut method) = item.clone() {
            let mut detected_item = None;
            let mut ignore = false;
            method.attrs.retain(|attr| {
                let ident = attr.meta.path().get_ident();
                if let Some(ident) = ident {
                    match ident.to_string().as_str() {
                        "constructor" => {
                            detected_item = Some(DetectedItem::Constructor);
                            return false;
                        }
                        "before_all" => {
                            detected_item = Some(DetectedItem::BeforeAll);
                            return false;
                        }
                        "before_each" => {
                            detected_item = Some(DetectedItem::BeforeEach);
                            return false;
                        }
                        "after_each" => {
                            detected_item = Some(DetectedItem::AfterEach);
                            return false;
                        }
                        "after_all" => {
                            detected_item = Some(DetectedItem::AfterAll);
                            return false;
                        }
                        "test_case" => {
                            if let Ok(syn::Lit::Str(lit_str)) = attr
                                .meta
                                .require_list()
                                .expect("`test_case` attribute must contain test name")
                                .parse_args()
                            {
                                detected_item = Some(DetectedItem::TestCase(lit_str.value()));
                            }
                            return false;
                        }
                        "ignore" => {
                            ignore = true;
                            return false;
                        }
                        _ => {}
                    }
                }
                true
            });
            cleaned_items.push(ImplItem::Fn(method.clone()));

            if ignore {
                assert!(
                    matches!(detected_item, Some(DetectedItem::TestCase(_))),
                    "The `ignore` attribute can only be used with `test_case` methods"
                );
            }

            match detected_item {
                Some(DetectedItem::Constructor) => {
                    if constructor.is_some() {
                        panic!("Only one constructor is allowed in a test suite");
                    }
                    constructor = Some(method);
                }
                Some(DetectedItem::BeforeAll) => {
                    if before_all.is_some() {
                        panic!("Only one 'before_all' method is allowed in a test suite");
                    }
                    before_all = Some(method);
                }
                Some(DetectedItem::BeforeEach) => {
                    if before_each.is_some() {
                        panic!("Only one 'before_each' method is allowed in a test suite");
                    }
                    before_each = Some(method);
                }
                Some(DetectedItem::AfterEach) => {
                    if after_each.is_some() {
                        panic!("Only one 'after_each' method is allowed in a test suite");
                    }
                    after_each = Some(method);
                }
                Some(DetectedItem::AfterAll) => {
                    if after_all.is_some() {
                        panic!("Only one 'after_all' method is allowed in a test suite");
                    }
                    after_all = Some(method);
                }
                Some(DetectedItem::TestCase(name)) => {
                    test_cases.push((name, method, ignore));
                }
                None => {}
            }
        } else {
            cleaned_items.push(item.clone());
        }
    }

    let constructor = constructor.unwrap_or_else(|| {
        panic!("A test suite must have a constructor method annotated with #[constructor]");
    });
    // The only argument to the constructor should be config type.
    let config_ty = constructor.sig.inputs.first().cloned().unwrap_or_else(|| {
        panic!("Constructor method must have a single argument for the config type");
    });
    let config_ty = if let syn::FnArg::Typed(pat_type) = config_ty {
        pat_type.ty
    } else {
        panic!("Constructor method must have a single argument for the config type");
    };
    let Type::Reference(config_ty) = *config_ty else {
        panic!("Constructor method must take reference to the config type as an argument");
    };
    let Type::Path(config_ty) = *config_ty.elem else {
        panic!("Constructor method must take reference to the config type as an argument");
    };
    let config_ty_name = config_ty
        .path
        .get_ident()
        .expect("Expected a config type name");
    let constructor_fn_name = &constructor.sig.ident;

    let crate_name = quote::format_ident!("e2e");

    let before_all_code = if let Some(before_all) = before_all {
        let fn_name = &before_all.sig.ident;
        quote! {
            #struct_ty_name::#fn_name(&self).await
        }
    } else {
        quote! {
            Ok(())
        }
    };
    let before_each_code = if let Some(before_each) = before_each {
        let fn_name = &before_each.sig.ident;
        quote! {
            #struct_ty_name::#fn_name(&self).await
        }
    } else {
        quote! {
            Ok(())
        }
    };
    let after_each_code = if let Some(after_each) = after_each {
        let fn_name = &after_each.sig.ident;
        quote! {
            #struct_ty_name::#fn_name(&self).await
        }
    } else {
        quote! {
            Ok(())
        }
    };
    let after_all_code = if let Some(after_all) = after_all {
        let fn_name = &after_all.sig.ident;
        quote! {
            #struct_ty_name::#fn_name(&self).await
        }
    } else {
        quote! {
            Ok(())
        }
    };

    let factory_name = quote::format_ident!("{}Factory", struct_ty_name);

    let mut test_case_code = Vec::new();
    let mut test_case_objects = Vec::new();
    for (id, (name, method, ignore)) in test_cases.into_iter().enumerate() {
        let test_fn_name = &method.sig.ident;

        let test_ty_name = quote::format_ident!("{}Test{}", struct_ty_name, id);
        let test_case = quote! {
            struct #test_ty_name(#struct_ty_name);

            #[async_trait::async_trait]
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

    let factory_fn: syn::ImplItem = syn::parse_quote! {
        pub fn factory() -> Box<dyn #crate_name::TestSuiteFactory<#config_ty_name>> {
            Box::new(#factory_name)
        }
    };
    cleaned_items.push(factory_fn);

    let cleaned_impl = ItemImpl {
        items: cleaned_items,
        ..input
    };

    // Placeholder for generating the test suite logic
    let output = quote! {
        #cleaned_impl

        #[async_trait::async_trait]
        impl #crate_name::TestSuite for #struct_ty_name {
            fn name(&self) -> String {
                #suite_name.to_string()
            }

            fn tests(&self) -> Vec<Box<dyn #crate_name::Test>> {
                vec![
                    #(#test_case_objects),*
                ]
            }

            async fn before_all(&self) -> anyhow::Result<()> {
                #before_all_code
            }

            async fn before_each(&self) -> anyhow::Result<()> {
                #before_each_code
            }

            async fn after_each(&self) -> anyhow::Result<()> {
                #after_each_code
            }

            async fn after_all(&self) -> anyhow::Result<()> {
                #after_all_code
            }
        }

        struct #factory_name;

        #[async_trait::async_trait]
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

        #(#test_case_code)*
    };

    TokenStream::from(output)
}
