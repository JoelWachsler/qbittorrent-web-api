mod method_with_params;
mod method_without_params;
mod return_type;
mod send_method_builder;

use crate::{generate::util, parser};
use case::CaseExt;
use quote::quote;

use self::{
    method_with_params::create_method_with_params,
    method_without_params::create_method_without_params,
};

pub fn generate_methods(
    group: &parser::ApiGroup,
    auth: &syn::Ident,
    group_name_camel: &syn::Ident,
) -> proc_macro2::TokenStream {
    let methods_and_extra = group
        .methods
        .iter()
        .map(|method| generate_method(group, method));

    let methods = methods_and_extra
        .clone()
        .map(|MethodsAndExtra { methods, .. }| methods);

    let extra = methods_and_extra.flat_map(|MethodsAndExtra { extra: structs, .. }| structs);

    quote! {
        impl <'a> #group_name_camel<'a> {
            pub fn new(auth: &'a #auth) -> Self {
                Self { auth }
            }

            #(#methods)*
        }

        #(#extra)*
    }
}

#[derive(Debug)]
pub struct MethodsAndExtra {
    methods: proc_macro2::TokenStream,
    extra: Option<proc_macro2::TokenStream>,
}

impl MethodsAndExtra {
    pub fn new(methods: proc_macro2::TokenStream) -> Self {
        Self {
            methods,
            extra: None,
        }
    }

    pub fn with_structs(mut self, structs: proc_macro2::TokenStream) -> Self {
        self.extra = Some(structs);
        self
    }
}

fn generate_method(group: &parser::ApiGroup, method: &parser::ApiMethod) -> MethodsAndExtra {
    let method_name = util::to_ident(&method.name.to_snake());
    let url = format!("/api/v2/{}/{}", group.url, method.url);

    match &method.parameters {
        Some(params) => create_method_with_params(group, method, params, &method_name, &url),
        None => create_method_without_params(group, method, method_name, &url),
    }
}
