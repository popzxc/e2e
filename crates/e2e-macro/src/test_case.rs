use syn::spanned::Spanned as _;

#[derive(Debug)]
pub(crate) struct TestCase {
    pub(crate) name: String,
    pub(crate) method: syn::ImplItemFn,
    pub(crate) ignore: bool,
}

impl TestCase {
    pub const ID: &'static str = "test_case";

    pub fn new(method: syn::ImplItemFn, attr: &syn::Attribute) -> syn::Result<Self> {
        let Ok(syn::Lit::Str(lit_str)) = attr
            .meta
            .require_list()
            .expect("`test_case` attribute must contain test name")
            .parse_args()
        else {
            return Err(syn::Error::new(
                attr.span(),
                "Expected a string literal for the test case name",
            ));
        };
        let name = lit_str.value();
        if name.is_empty() {
            return Err(syn::Error::new(
                lit_str.span(),
                "Test case name cannot be empty",
            ));
        }

        // let ignore = method.attrs.iter().any(|attr| attr.path.is_ident("ignore"));
        let ignore = false;

        Ok(Self {
            name,
            method,
            ignore,
        })
    }
}
