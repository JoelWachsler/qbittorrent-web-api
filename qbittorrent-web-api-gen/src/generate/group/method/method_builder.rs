use quote::quote;

pub struct MethodBuilder {
    method_name: syn::Ident,
    url: String,
    return_type: Option<proc_macro2::TokenStream>,
}

impl MethodBuilder {
    pub fn new(method_name: &syn::Ident, url: &str) -> Self {
        Self {
            method_name: method_name.clone(),
            url: url.to_string(),
            return_type: None,
        }
    }

    pub fn return_type(mut self, value: &proc_macro2::TokenStream) -> Self {
        self.return_type = Some(value.clone());
        self
    }

    pub fn build(&self) -> proc_macro2::TokenStream {
        let method_name = &self.method_name;
        let (return_type, parse_type) = match &self.return_type {
            Some(t) => (t.clone(), quote! { .json::<#t>() }),
            None => (quote! { String }, quote! { .text() }),
        };
        let url = &self.url;

        quote! {
            pub async fn #method_name(&self) -> Result<#return_type> {
                let res = self.auth
                    .authenticated_client(#url)
                    .send()
                    .await?
                    #parse_type
                    .await?;

                Ok(res)
            }
        }
    }
}
