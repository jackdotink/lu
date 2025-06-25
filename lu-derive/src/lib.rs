use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Userdata)]
pub fn userdata_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = syn::parse_macro_input!(input as DeriveInput);

    if !generics.params.is_empty() {
        panic!("Userdata derive macro does not support generics");
    }

    quote! {
        impl ::lu::Userdata for #ident {
            fn tag() -> u32 {
                static TAG: ::std::sync::OnceLock<u32> = ::std::sync::OnceLock::new();
                *TAG.get_or_init(::lu::unique_tag)
            }

            fn name() -> &'static str {
                stringify!(#ident)
            }
        }
    }
    .into()
}
