use quote::quote;
use quote::ToTokens;

pub fn to_ident(name: &str) -> proc_macro2::Ident {
    syn::Ident::new(name, proc_macro2::Span::call_site())
}

pub fn add_docs<T: ToTokens>(docs: &Option<String>, stream: T) -> proc_macro2::TokenStream {
    if let Some(docs) = docs {
        quote! {
            #[doc = #docs]
            #stream
        }
    } else {
        quote! {
            #stream
        }
    }
}
